use std::env;
use std::path::Path;

use color_eyre::eyre::{Result, eyre};
use toml_edit::{DocumentMut, Item, Table, value};

use crate::pkg::manifest::Manifest;
use crate::pkg::stage;

/// `jumpjet wit` — re-sync the project's staged WIT to the runtime WIT embedded in
/// *this* CLI build, then re-stage dependency WIT and regenerate the world.
///
/// The runtime WIT is only copied into a project at `jumpjet new` time, so after a
/// `jumpjet upgrade` an existing project's staged runtime WIT
/// (`.jumpjet/wit/deps/runtime` for games, `wit/deps/jumpjet-runtime` for libs)
/// drifts from the host the CLI actually implements. This command refreshes it from
/// the embedded definitions and pins `[runtime] version` to match, so guest bindgen
/// generates against the runtime the installed CLI will run.
pub async fn wit() -> Result<()> {
    let dir = env::current_dir()?;
    let manifest = Manifest::load_from(&dir)?;

    // 1. Refresh the embedded runtime WIT into the consumer's deps dir. Games key it
    //    as `runtime`, libs as `jumpjet-runtime` — matching `jumpjet new`.
    let wit_root = stage::wit_root(&dir, &manifest);
    let runtime_dest = if manifest.is_lib() {
        wit_root.join("deps").join("jumpjet-runtime")
    } else {
        wit_root.join("deps").join("runtime")
    };
    crate::commands::new::package::copy_runtime_wit(&runtime_dest)?;

    // 2. Re-resolve dependencies and re-stage their WIT, regenerating the game world.
    let resolution = crate::pkg::resolve::resolve(&dir, false).await?;
    stage::stage_wit(&dir, &manifest, &resolution)?;

    // 3. Pin `[runtime] version` to this CLI, since the staged WIT now matches it.
    let cli_version = env!("CARGO_PKG_VERSION");
    set_runtime_version(&dir, cli_version)?;

    println!("Synced WIT to runtime v{cli_version}");
    Ok(())
}

/// Sets `[runtime] version` in `jumpjet.toml` to `version`, preserving the rest of
/// the file's formatting and comments.
fn set_runtime_version(dir: &Path, version: &str) -> Result<()> {
    let toml_path = dir.join(Manifest::FILE_NAME);
    let text = std::fs::read_to_string(&toml_path)?;
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| eyre!("parsing jumpjet.toml: {e}"))?;

    if doc.get("runtime").is_none() {
        doc["runtime"] = Item::Table(Table::new());
    }
    doc["runtime"]["version"] = value(version);

    std::fs::write(&toml_path, doc.to_string())?;
    Ok(())
}
