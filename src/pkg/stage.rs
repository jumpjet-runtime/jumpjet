//! Stages resolved dependencies' WIT into the consumer's source tree so guest
//! bindgen (`wit-bindgen`'s `generate!`, `jco`, etc.) can resolve `import
//! <namespace>:<name>/...`.
//!
//! Two things have to happen for a dependency to actually generate bindings:
//!  1. The dependency's WIT package is written under the consumer's `deps/` so the
//!     WIT parser can *resolve* the package (see [`stage_wit`]). The staged WIT is
//!     derived from each dependency's *component* rather than its source WIT: the
//!     component encodes only the package's public interfaces, so the staged
//!     package is self-contained and carries no back-reference to `jumpjet:runtime`
//!     (which the consumer already provides, and which a flat `deps/` resolution
//!     can't satisfy from a dependency's WIT).
//!  2. The consumer's *world* imports each interface the dependency exports — a
//!     package present under `deps/` but referenced by no world is on the search
//!     path but unused, and `wit-bindgen` only generates bindings for what the
//!     world graph touches.
//!
//! A game gets its own WIT package (`jumpjet:game`) under `.jumpjet/wit/`, whose
//! `game` world `include`s the host `jumpjet:runtime/runtime` world (pulling in
//! every host import plus the `guest` export) and adds the dependency imports; the
//! guest targets `game`. The runtime itself is staged as an ordinary dependency at
//! `.jumpjet/wit/deps/runtime/` (alongside other packages), so a single-directory
//! resolve of `.jumpjet/wit/` sees everything. Lib packages author their own world
//! and add the `import`s to it themselves.

use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

use crate::pkg::manifest::Manifest;
use crate::pkg::resolve::Resolution;
use crate::pkg::source::{interface_wit, package_world_imports};

/// Name of the generated world a game guest targets (`world: "game"`).
const GAME_WORLD: &str = "game";
/// File the generated world is written to. Wholly owned by jumpjet and rewritten on
/// every build, so the `.gen.` infix signals "do not hand-edit".
const GENERATED_WORLD_FILE: &str = "world.gen.wit";

/// The WIT package directory that holds the consumer's world:
/// - games: the game's own `jumpjet:game` package at `.jumpjet/wit`.
/// - libs: the world lives in the package's own `wit/`.
pub fn wit_root(dir: &Path, manifest: &Manifest) -> PathBuf {
    if manifest.is_lib() {
        let wit = manifest.build.wit.clone().unwrap_or_else(|| "wit".into());
        dir.join(wit)
    } else {
        dir.join(".jumpjet").join("wit")
    }
}

/// The directory a consumer's WIT dependencies belong in (`<wit-root>/deps`).
pub fn deps_dir(dir: &Path, manifest: &Manifest) -> PathBuf {
    wit_root(dir, manifest).join("deps")
}

/// Writes each resolved package's public interface WIT into the consumer's deps
/// directory (one self-contained package per dependency) and, for games,
/// regenerates the `game` world that imports them.
pub fn stage_wit(dir: &Path, manifest: &Manifest, resolution: &Resolution) -> Result<()> {
    let root = wit_root(dir, manifest);
    let base = root.join("deps");
    let mut imports = Vec::new();

    for pkg in &resolution.packages {
        let component = pkg.stored.read_component()?;
        let wit = interface_wit(&component, &pkg.id.name)?;

        let dest = base.join(format!("{}-{}", pkg.id.name.namespace, pkg.id.name.name));
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        std::fs::create_dir_all(&dest)?;
        std::fs::write(dest.join(format!("{}.wit", pkg.id.name.name)), wit)?;

        imports.extend(package_world_imports(&component, &pkg.id.name)?);
    }

    imports.sort();
    imports.dedup();

    // Games bind the generated `game` world; it must always exist (even with no
    // dependencies) for `world: "game"` to resolve. Libs own their world.
    if !manifest.is_lib() {
        write_game_world(&root, &imports)?;
    }
    Ok(())
}

/// Writes the generated `game` world (`include jumpjet:runtime/runtime;` plus an
/// `import` per dependency interface) into the game's WIT root (`.jumpjet/wit`).
/// Called by [`stage_wit`] on every build and by `jumpjet new game` at scaffold
/// time so a freshly created game resolves `world: "game"` before it has any
/// dependencies — the same file later dependency imports are injected into.
pub fn write_game_world(wit_root: &Path, imports: &[String]) -> Result<()> {
    std::fs::create_dir_all(wit_root)?;
    std::fs::write(wit_root.join(GENERATED_WORLD_FILE), game_world(imports))?;
    Ok(())
}

/// The generated `game` world: its own `jumpjet:game` package that `include`s the
/// host `jumpjet:runtime/runtime` world and adds an `import` per dependency
/// interface.
fn game_world(imports: &[String]) -> String {
    let mut out = String::new();
    out.push_str("package jumpjet:game;\n\n");
    out.push_str(&format!("world {GAME_WORLD} {{\n"));
    out.push_str("  include jumpjet:runtime/runtime;\n");
    for imp in imports {
        out.push_str("  ");
        out.push_str(imp);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_world_includes_runtime() {
        let w = game_world(&[]);
        assert!(w.contains("package jumpjet:game;"));
        assert!(w.contains(&format!("world {GAME_WORLD} {{")));
        assert!(w.contains("include jumpjet:runtime/runtime;"));
    }

    #[test]
    fn game_world_adds_each_import() {
        let w = game_world(&[
            "import jumpjet:threejs/three@0.1.0;".to_string(),
            "import jumpjet:physics/world@2.0.0;".to_string(),
        ]);
        assert!(w.contains("include jumpjet:runtime/runtime;"));
        assert!(w.contains("import jumpjet:threejs/three@0.1.0;"));
        assert!(w.contains("import jumpjet:physics/world@2.0.0;"));
    }
}
