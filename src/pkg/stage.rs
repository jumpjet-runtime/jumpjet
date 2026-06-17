//! Stages resolved dependencies' WIT into the consumer's source tree so guest
//! bindgen (`wit-bindgen`'s `generate!`, `jco`, etc.) can resolve `import
//! <namespace>:<name>/...`.
//!
//! The staged WIT is derived from each dependency's *component* rather than its
//! source WIT: the component encodes only the package's public interfaces, so the
//! staged package is self-contained and carries no back-reference to
//! `jumpjet:runtime` (which the consumer already provides, and which a flat
//! `deps/` resolution can't satisfy from a dependency's WIT).

use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

use crate::pkg::manifest::Manifest;
use crate::pkg::resolve::Resolution;
use crate::pkg::source::interface_wit;

/// The directory a consumer's WIT dependencies belong in, determined by which WIT
/// package holds the consumer's world:
/// - games: the world lives in `jumpjet:runtime` (`.jumpjet/wit/runtime`), so deps
///   go in `.jumpjet/wit/runtime/deps`.
/// - libs: the world lives in the package's own `wit/`, so deps go in `wit/deps`.
pub fn deps_dir(dir: &Path, manifest: &Manifest) -> PathBuf {
    if manifest.is_lib() {
        let wit = manifest.build.wit.clone().unwrap_or_else(|| "wit".into());
        dir.join(wit).join("deps")
    } else {
        dir.join(".jumpjet").join("wit").join("runtime").join("deps")
    }
}

/// Writes each resolved package's public interface WIT into the consumer's deps
/// directory, one self-contained package per dependency.
pub fn stage_wit(dir: &Path, manifest: &Manifest, resolution: &Resolution) -> Result<()> {
    let base = deps_dir(dir, manifest);
    for pkg in &resolution.packages {
        let component = pkg.stored.read_component()?;
        let wit = interface_wit(&component, &pkg.id.name)?;

        let dest = base.join(format!("{}-{}", pkg.id.name.namespace, pkg.id.name.name));
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        std::fs::create_dir_all(&dest)?;
        std::fs::write(dest.join(format!("{}.wit", pkg.id.name.name)), wit)?;
    }
    Ok(())
}
