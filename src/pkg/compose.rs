//! Composes dependency components into a consumer component.
//!
//! Uses `wac-graph`'s `plug` model: the consumer is the *socket* and each
//! dependency is a *plug* whose exports satisfy the socket's matching imports. The
//! consumer's `jumpjet:runtime/*` and `wasi:*` imports have no plug and are left
//! unsatisfied — the runtime `Linker` provides them at load time.

use color_eyre::eyre::{Result, eyre};
use wac_graph::types::Package;
use wac_graph::{CompositionGraph, EncodeOptions, PlugError, plug};

use crate::pkg::manifest::PackageId;

pub struct ComposeDep {
    pub id: PackageId,
    pub component: Vec<u8>,
}

/// Composes `deps` into the `consumer` component, returning the new component
/// bytes. With no deps (or none the consumer actually imports) the consumer is
/// returned unchanged.
pub fn compose(consumer: Vec<u8>, deps: &[ComposeDep]) -> Result<Vec<u8>> {
    if deps.is_empty() {
        return Ok(consumer);
    }

    let mut graph = CompositionGraph::new();

    let socket = Package::from_bytes("jumpjet:root", None, consumer.clone(), graph.types_mut())
        .map_err(|e| eyre!("reading consumer component: {e:#}"))?;
    let socket_id = graph
        .register_package(socket)
        .map_err(|e| eyre!("registering consumer component: {e:#}"))?;

    let mut plug_ids = Vec::new();
    for dep in deps {
        let pkg = Package::from_bytes(
            &dep.id.name.to_string(),
            Some(&dep.id.version),
            dep.component.clone(),
            graph.types_mut(),
        )
        .map_err(|e| eyre!("reading dependency `{}`: {e:#}", dep.id))?;
        let id = graph
            .register_package(pkg)
            .map_err(|e| eyre!("registering dependency `{}`: {e:#}", dep.id))?;
        plug_ids.push(id);
    }

    match plug(&mut graph, plug_ids, socket_id) {
        Ok(()) => {}
        // The consumer imports none of its declared dependencies — leave it as-is.
        Err(PlugError::NoPlugHappened) => return Ok(consumer),
        Err(e) => return Err(eyre!("composing dependencies: {e:#}")),
    }

    graph
        .encode(EncodeOptions::default())
        .map_err(|e| eyre!("encoding composed component: {e:#}"))
}
