use color_eyre::eyre::eyre;
use subprocess::{Exec, Redirection};

use crate::pkg::manifest::Manifest;
use crate::Result;

/// Builds this package and publishes its component to a registry via the `wkg`
/// CLI (wasm-pkg-tools), which resolves the target registry from the shared
/// `~/.config/wasm-pkg/config.toml`.
pub async fn publish() -> Result<()> {
    let dir = std::env::current_dir()?;
    let manifest = Manifest::load_from(&dir)?;

    if !manifest.is_lib() {
        return Err(eyre!("only packages (`type = \"lib\"`) can be published"));
    }
    let name = manifest.package_name()?;
    let version = manifest
        .package
        .version
        .clone()
        .ok_or_else(|| eyre!("[package].version is required to publish"))?;

    // Build to make sure the published component is current.
    crate::commands::build::build(&false).await?;

    let output = manifest.build.output.clone().unwrap_or_else(|| "bin".into());
    let entrypoint = manifest
        .build
        .entrypoint
        .clone()
        .ok_or_else(|| eyre!("[build].entrypoint is required to publish"))?;
    let component = dir.join(&output).join(&entrypoint);

    let cap = Exec::cmd("wkg")
        .arg("publish")
        .arg(&component)
        .args(&["--package", &format!("{name}@{version}")])
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture();
    let cap = match cap {
        Ok(c) => c,
        Err(_) => {
            return Err(eyre!(
                "`wkg` was not found on your PATH. Install wasm-pkg-tools to publish: https://github.com/bytecodealliance/wasm-pkg-tools"
            ))
        }
    };
    if !cap.success() {
        return Err(eyre!("`wkg publish` failed:\n{}", cap.stdout_str()));
    }

    println!("Published {name}@{version}");
    Ok(())
}
