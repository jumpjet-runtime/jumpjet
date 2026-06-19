//! Minimal static-file dev server for `jumpjet run --target web`.
//!
//! Hand-rolled on `tokio` (no extra deps): serves the assembled web site with the
//! COOP/COEP/CORP headers the WASI shim's SharedArrayBuffer path needs, and exposes
//! an SSE endpoint (`/__jumpjet_reload`) used for live reload. The `run` watch loop
//! broadcasts on `reload_tx` after a rebuild; an injected client script in
//! `index.html` reloads the tab.

use std::path::{Path, PathBuf};

use color_eyre::eyre;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

use crate::Result;

/// SSE path the injected client script subscribes to.
const RELOAD_PATH: &str = "/__jumpjet_reload";

/// Serves `site_dir` on `127.0.0.1:port` until the process exits. Each rebuild
/// should `reload_tx.send(())` to reload connected browsers.
pub async fn serve(site_dir: PathBuf, port: u16, reload_tx: broadcast::Sender<()>) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| eyre::eyre!("failed to bind 127.0.0.1:{port}: {e}"))?;

    loop {
        let (stream, _) = listener.accept().await?;
        let site = site_dir.clone();
        let rx = reload_tx.subscribe();
        tokio::spawn(async move {
            let _ = handle_conn(stream, site, rx).await;
        });
    }
}

async fn handle_conn(
    mut stream: TcpStream,
    site_dir: PathBuf,
    reload_rx: broadcast::Receiver<()>,
) -> Result<()> {
    // Read request headers (GET only; no body to consume).
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") || buf.len() > 65536 {
            break;
        }
    }

    let req = String::from_utf8_lossy(&buf);
    let raw_path = req
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");
    let path = raw_path.split(['?', '#']).next().unwrap_or("/");

    if path == RELOAD_PATH {
        return serve_sse(stream, reload_rx).await;
    }

    let rel = path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    if rel.contains("..") {
        return write_response(
            &mut stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            b"forbidden",
        )
        .await;
    }

    let file_path = site_dir.join(rel);
    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let ct = content_type(rel);
            if rel == "index.html" {
                let injected = inject_reload(&String::from_utf8_lossy(&bytes));
                write_response(&mut stream, "200 OK", ct, injected.as_bytes()).await
            } else {
                write_response(&mut stream, "200 OK", ct, &bytes).await
            }
        }
        Err(_) => {
            write_response(
                &mut stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"not found",
            )
            .await
        }
    }
}

/// Holds the connection open and pushes a `reload` event on each broadcast.
async fn serve_sse(mut stream: TcpStream, mut reload_rx: broadcast::Receiver<()>) -> Result<()> {
    let header = "HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache\r\n\
        Cross-Origin-Opener-Policy: same-origin\r\n\
        Cross-Origin-Embedder-Policy: require-corp\r\n\
        Cross-Origin-Resource-Policy: same-origin\r\n\
        Connection: keep-alive\r\n\r\n";
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(b": connected\n\n").await?;
    stream.flush().await?;

    loop {
        match reload_rx.recv().await {
            Ok(()) => {
                if stream.write_all(b"data: reload\n\n").await.is_err()
                    || stream.flush().await.is_err()
                {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
    Ok(())
}

async fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-cache\r\n\
         Cross-Origin-Opener-Policy: same-origin\r\n\
         Cross-Origin-Embedder-Policy: require-corp\r\n\
         Cross-Origin-Resource-Policy: same-origin\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "json" | "map" => "application/json; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Injects the live-reload client just before `</body>` (run-only; never written to
/// a `bundle`).
fn inject_reload(html: &str) -> String {
    let script = format!(
        "<script>new EventSource('{RELOAD_PATH}').onmessage=()=>location.reload();</script>"
    );
    match html.rfind("</body>") {
        Some(idx) => format!("{}{}{}", &html[..idx], script, &html[idx..]),
        None => format!("{html}{script}"),
    }
}

/// Opens `url` in the default browser (best effort).
pub fn open_browser(url: &str) {
    let _ = url;
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
}

/// Returns the last-modified time of `path`, if available — used by the watch loop.
pub fn mtime(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}
