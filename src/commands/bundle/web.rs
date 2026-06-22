use std::env;

use crate::Result;
use crate::commands::build;
use crate::pkg::manifest::Manifest;

/// Assembles the deployable web site into `bundle/web/`: the guest compiled by
/// `build --target web` (`<output>/web/guest`) plus the embedded host runtime,
/// harness, page, and WASI shim. The web analogue of producing a macOS `.app`.
///
/// Assumes `build --target web` has already run (see the dispatch in `app.rs`).
pub async fn bundle_project(_release: &bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let manifest = Manifest::load()?;

    let output = manifest.primary_build()?.output.clone().unwrap_or_else(|| "bin".into());
    let guest_dir = current_dir.join(&output).join("web").join("guest");
    let bundle_dir = current_dir.join("bundle").join("web");

    if bundle_dir.exists() {
        std::fs::remove_dir_all(&bundle_dir)?;
    }

    build::assemble_web_site(&guest_dir, &bundle_dir)?;

    println!("Web bundle emitted to {}", bundle_dir.display());
    println!(
        "Deploy its contents to any static host, or serve locally with `python3 -m http.server` from that dir."
    );
    Ok(())
}
