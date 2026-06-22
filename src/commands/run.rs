use std::env;
use std::path::PathBuf;
use std::time::Duration;

use color_eyre::eyre::eyre;
use tokio::sync::broadcast;

use crate::Result;
use crate::commands::{build, serve};
use crate::pkg::manifest::Manifest;

/// Native run: build the project and execute it in the in-process Jumpjet runtime.
/// With `server`, runs the headless `[server.build]` component instead of the
/// game/client.
pub async fn run(release: &bool, server: bool) -> Result<()> {
    crate::commands::build::build(release).await?;

    let current_dir = env::current_dir()?;
    let manifest = Manifest::load()?;

    if server {
        let server_build = manifest
            .server_build()
            .ok_or_else(|| eyre!("`--server` requires a [server.build] section in jumpjet.toml"))?;
        let output_path = current_dir.join(server_build.output.as_deref().unwrap_or("bin"));
        let binary = std::fs::read(output_path.join(build::SERVER_FILE))?;
        jumpjet::runtime::run_server(output_path, binary);
    } else {
        let output_path =
            current_dir.join(manifest.primary_build()?.output.as_deref().unwrap_or("bin"));
        let binary = std::fs::read(output_path.join(build::ENTRYPOINT_FILE))?;
        jumpjet::runtime::run(output_path, binary, !release);
    }

    Ok(())
}

/// Web run: compile + assemble a runnable site, serve it locally, open the browser,
/// and live-reload when the build input artifact changes.
///
/// Watch model (per design): jumpjet does NOT recompile the guest — the developer
/// runs their own compiler in watch mode (e.g. `cargo watch -- cargo build --target
/// wasm32-wasip2`). When the resulting wasm changes, we re-run only the post-compile
/// pipeline (componentize + jco transpile) and reload the tab.
pub async fn run_web(release: &bool, port: u16) -> Result<()> {
    let current_dir = env::current_dir()?;
    let manifest = Manifest::load()?;
    let primary = manifest.primary_build()?;
    let output_path = current_dir.join(primary.output.as_deref().unwrap_or("bin"));
    let site_dir = output_path.join("web");
    let guest_dir = site_dir.join("guest");
    let input_artifact = build::source_entrypoint(primary)?;

    // Initial full build (runs `pre`) + assemble the servable site at bin/web.
    build::build_web(release).await?;
    build::assemble_web_site(&guest_dir, &site_dir)?;

    let (reload_tx, _) = broadcast::channel::<()>(16);

    // Serve in the background.
    let server = {
        let site = site_dir.clone();
        let tx = reload_tx.clone();
        tokio::spawn(async move { serve::serve(site, port, tx).await })
    };

    let url = format!("http://localhost:{port}");
    println!("Serving {url} (COOP/COEP enabled)");
    println!(
        "Watching {} — rebuild your guest to live-reload. Ctrl-C to stop.",
        input_artifact.display()
    );
    serve::open_browser(&url);

    // Foreground watch loop.
    let watch = watch_and_reload(input_artifact, *release, reload_tx);

    tokio::select! {
        r = server => r??,
        r = watch => r?,
    }
    Ok(())
}

/// Polls the build input artifact's mtime; on change re-runs the incremental web
/// build (which rewrites `bin/web/guest`) and broadcasts a reload.
async fn watch_and_reload(
    artifact: PathBuf,
    release: bool,
    reload_tx: broadcast::Sender<()>,
) -> Result<()> {
    let mut last = serve::mtime(&artifact);
    loop {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let current = serve::mtime(&artifact);
        if current.is_some() && current != last {
            // Small debounce so we read the artifact after the writer finishes.
            tokio::time::sleep(Duration::from_millis(120)).await;
            last = serve::mtime(&artifact);

            match build::build_web_incremental(&release).await {
                Ok(()) => {
                    println!("Rebuilt — reloading browser.");
                    let _ = reload_tx.send(());
                }
                Err(e) => eprintln!("Rebuild failed: {e}"),
            }
        }
    }
}
