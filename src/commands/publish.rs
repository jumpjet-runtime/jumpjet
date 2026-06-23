use color_eyre::eyre::eyre;
use wasm_pkg_client::{Client, PackageRef, PublishOpts};

use crate::Result;
use crate::pkg::manifest::Manifest;
use crate::pkg::source::registry_config;

/// Builds this package and publishes its component to the Jumpjet registry
/// (`packages.jumpjet.dev`) using the `wasm-pkg-client` library.
pub async fn publish() -> Result<()> {
    let dir = std::env::current_dir()?;
    let manifest = Manifest::load_from(&dir)?;

    if !manifest.is_lib() {
        return Err(eyre!("only packages (`type = \"lib\"`) can be published"));
    }
    let name = manifest.package_name()?;
    let version = manifest
        .project
        .version
        .clone()
        .ok_or_else(|| eyre!("[project].version is required to publish"))?;

    // Build to make sure the published component is current.
    crate::commands::build::build(&false).await?;

    let build = manifest.primary_build()?;
    let output = build.output.clone().unwrap_or_else(|| "bin".into());
    let entrypoint = build
        .entrypoint
        .clone()
        .ok_or_else(|| eyre!("[lib.build].entrypoint is required to publish"))?;
    let component = dir.join(&output).join(&entrypoint);

    let package: PackageRef = name
        .to_string()
        .parse()
        .map_err(|e| eyre!("invalid package name `{name}`: {e}"))?;

    let client = Client::new(registry_config()?);

    let opts = PublishOpts {
        package: Some((package, version.clone())),
        ..Default::default()
    };
    let (published, version) = client
        .publish_release_file(&component, opts)
        .await
        .map_err(|e| eyre!("publishing `{name}@{version}`: {e}"))?;

    println!("Published {published}@{version}");
    Ok(())
}
