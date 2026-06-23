//! `jumpjet auth signin` / `jumpjet auth logout`.
//!
//! Signin runs a browser-based OAuth flow with PKCE (RFC 7636). We never hold a
//! client secret: a one-shot loopback server on a random port receives the
//! authorization code, which we exchange (proving possession of the PKCE verifier)
//! for an access token. The token is stored at `~/.jumpjet/credentials` (0600) and
//! sent as `Authorization: Bearer <token>` on authenticated API calls.

use std::{io::ErrorKind, path::PathBuf, time::Duration};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use color_eyre::eyre::{Result, bail, eyre};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// How long we wait for the user to complete the browser flow before giving up.
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(120);

/// Base URL of the Jumpjet web app. Overridable via `JUMPJET_BASE_URL` so the flow
/// can be pointed at a local/staging deployment during development.
pub(crate) fn base_url() -> String {
    std::env::var("JUMPJET_BASE_URL")
        .ok()
        .map(|s| s.trim_end_matches('/').to_string())
        // .unwrap_or_else(|| "https://jumpjet.dev".to_string())
        .unwrap_or_else(|| "http://localhost:5173".to_string())
}

/// Stored credentials. Kept deliberately small — just the bearer token plus
/// whatever account descriptor the server handed back (for display / `whoami`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<Value>,
}

/// Token-exchange response from `POST /api/auth/cli/token`.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    account: Option<Value>,
}

/// Run the interactive sign-in flow.
pub async fn signin() -> Result<()> {
    // 1. CSRF state + PKCE pair.
    let state = random_token();
    let code_verifier = random_token();
    let code_challenge = pkce_challenge(&code_verifier);

    // 2. Loopback server on a random free port.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    // 3. Build the authorize URL and open the browser (printing it as a fallback).
    let enc = urlencoding::encode;
    let auth_url = format!(
        "{base}/auth/cli?redirect_uri={redirect}&state={state}&code_challenge={challenge}&code_challenge_method=S256",
        base = base_url(),
        redirect = enc(&redirect_uri),
        state = enc(&state),
        challenge = enc(&code_challenge),
    );

    println!("Opening your browser to sign in…");
    println!("If it doesn't open automatically, visit:\n\n    {auth_url}\n");
    crate::commands::serve::open_browser(&auth_url);

    // 4. Wait for the callback (with a timeout), verifying the CSRF state.
    let code = match tokio::time::timeout(CALLBACK_TIMEOUT, wait_for_callback(&listener, &state))
        .await
    {
        Ok(result) => result?,
        Err(_) => bail!("timed out after {}s waiting for the browser callback", CALLBACK_TIMEOUT.as_secs()),
    };

    // 5. Exchange the authorization code for an access token.
    let token = exchange_code(&code, &code_verifier, &redirect_uri).await?;

    // 6. Persist the token (0600) for use on subsequent API calls.
    let creds = Credentials {
        access_token: token.access_token,
        account: token.account,
    };
    save_credentials(&creds)?;

    println!("\n✓ Signed in{}.", account_suffix(creds.account.as_ref()));
    Ok(())
}

/// Remove stored credentials. Idempotent — succeeds even if not signed in.
pub async fn logout() -> Result<()> {
    let path = credentials_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => println!("✓ Signed out."),
        Err(e) if e.kind() == ErrorKind::NotFound => println!("Not signed in."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

/// Load the stored bearer token, if any. Used by authenticated API calls.
pub fn load_credentials() -> Result<Option<Credentials>> {
    let path = credentials_path()?;
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

// ---- PKCE helpers ----------------------------------------------------------

/// 32 bytes of CSPRNG randomness, base64url-encoded (no padding). Serves as both
/// the CSRF `state` and the PKCE `code_verifier` (43 chars, within RFC 7636's
/// 43–128 range).
fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// `code_challenge = base64url(SHA256(code_verifier))`.
fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

// ---- Callback server -------------------------------------------------------

/// Accept connections until one hits `/callback`, returning the `code`. Other
/// paths (e.g. the browser's favicon probe) get a 404 and are ignored. Any
/// `error` param, or a `state` mismatch, aborts the flow.
async fn wait_for_callback(listener: &TcpListener, expected_state: &str) -> Result<String> {
    loop {
        let (mut stream, _) = listener.accept().await?;
        let Some(target) = read_request_target(&mut stream).await? else {
            respond(&mut stream, 400, "Bad Request", "Bad request.").await?;
            continue;
        };

        let (path, query) = split_target(&target);
        if path != "/callback" {
            respond(&mut stream, 404, "Not Found", "Not found.").await?;
            continue;
        }

        let params = parse_query(query);

        if let Some(err) = params.iter().find(|(k, _)| k == "error").map(|(_, v)| v) {
            respond(
                &mut stream,
                400,
                "Bad Request",
                "Sign-in failed. You can close this tab and return to the terminal.",
            )
            .await?;
            bail!("authorization failed: {err}");
        }

        let state = params
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone());
        let code = params
            .iter()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.clone());

        if state.as_deref() != Some(expected_state) {
            respond(
                &mut stream,
                400,
                "Bad Request",
                "Sign-in failed (state mismatch). You can close this tab.",
            )
            .await?;
            bail!("state mismatch — possible CSRF, aborting");
        }

        let Some(code) = code else {
            respond(&mut stream, 400, "Bad Request", "Missing authorization code.").await?;
            bail!("callback did not include an authorization code");
        };

        respond(
            &mut stream,
            200,
            "OK",
            "You're signed in! You can close this tab and return to the terminal.",
        )
        .await?;
        return Ok(code);
    }
}

/// Read just enough of the request to get the path from the request line
/// (`GET /callback?... HTTP/1.1`). Returns `None` if the line is malformed.
async fn read_request_target(stream: &mut TcpStream) -> Result<Option<String>> {
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        // The request line ends at the first CRLF; headers/body don't matter to us.
        if let Some(pos) = buf.windows(2).position(|w| w == b"\r\n") {
            let line = String::from_utf8_lossy(&buf[..pos]).into_owned();
            return Ok(line.split_whitespace().nth(1).map(|s| s.to_string()));
        }
        if buf.len() > 8192 {
            break;
        }
    }
    Ok(None)
}

/// Write a minimal HTML response and close the connection.
async fn respond(stream: &mut TcpStream, status: u16, reason: &str, message: &str) -> Result<()> {
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Jumpjet</title>\
<style>body{{font-family:system-ui,sans-serif;display:grid;place-items:center;\
height:100vh;margin:0;color:#111}}div{{text-align:center}}\
h1{{color:#fa5400}}</style></head>\
<body><div><h1>Jumpjet</h1><p>{message}</p></div></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\n\
Content-Type: text/html; charset=utf-8\r\n\
Content-Length: {len}\r\n\
Connection: close\r\n\r\n{body}",
        len = body.len(),
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Split a request target into `(path, query)` (query excludes the `?`).
fn split_target(target: &str) -> (&str, &str) {
    match target.split_once('?') {
        Some((path, query)) => (path, query),
        None => (target, ""),
    }
}

/// Parse a URL query string into decoded key/value pairs.
fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn decode(s: &str) -> String {
    urlencoding::decode(s)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

// ---- Token exchange --------------------------------------------------------

async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let url = format!("{}/api/auth/cli/token", base_url());
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": code_verifier,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("token exchange failed ({status}): {body}");
    }
    resp.json::<TokenResponse>()
        .await
        .map_err(|e| eyre!("could not parse token response: {e}"))
}

// ---- Credential storage ----------------------------------------------------

/// `~/.jumpjet/credentials`.
fn credentials_path() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .ok_or_else(|| eyre!("could not determine home directory"))?;
    Ok(home.join(".jumpjet").join("credentials"))
}

/// Write credentials as JSON with owner-only (0600) permissions.
fn save_credentials(creds: &Credentials) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Render a " as <name>" suffix for the success message, best-effort.
fn account_suffix(account: Option<&Value>) -> String {
    let label = account.and_then(|a| {
        a.get("email")
            .or_else(|| a.get("name"))
            .or_else(|| a.get("username"))
            .or_else(|| a.get("handle"))
            .and_then(|v| v.as_str())
    });
    match label {
        Some(name) => format!(" as {name}"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_rfc7636_vector() {
        // RFC 7636, Appendix B.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            pkce_challenge(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_token_is_urlsafe_and_long_enough() {
        let t = random_token();
        // 32 bytes base64url (no pad) -> 43 chars, within RFC 7636's 43..=128.
        assert_eq!(t.len(), 43);
        assert!(t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert_ne!(random_token(), random_token());
    }

    #[test]
    fn split_and_parse_query() {
        let (path, query) = split_target("/callback?code=abc%20123&state=xyz");
        assert_eq!(path, "/callback");
        let params = parse_query(query);
        assert_eq!(params, vec![
            ("code".to_string(), "abc 123".to_string()),
            ("state".to_string(), "xyz".to_string()),
        ]);

        let (path, query) = split_target("/favicon.ico");
        assert_eq!(path, "/favicon.ico");
        assert!(parse_query(query).is_empty());
    }

    /// Drive the loopback server with raw HTTP: a favicon probe (ignored), then a
    /// real callback. Verifies code extraction and that the browser gets a 200.
    #[tokio::test]
    async fn callback_server_returns_code() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move { wait_for_callback(&listener, "st8").await });

        // Stray request on another path — should be 404'd and skipped.
        let mut probe = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        probe
            .write_all(b"GET /favicon.ico HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        let mut buf = String::new();
        probe.read_to_string(&mut buf).await.unwrap();
        assert!(buf.starts_with("HTTP/1.1 404"));

        // The real callback.
        let mut cb = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        cb.write_all(b"GET /callback?code=the_code&state=st8 HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        let mut buf = String::new();
        cb.read_to_string(&mut buf).await.unwrap();
        assert!(buf.starts_with("HTTP/1.1 200"));

        assert_eq!(server.await.unwrap().unwrap(), "the_code");
    }

    /// A mismatched CSRF state must abort and return an error.
    #[tokio::test]
    async fn callback_server_rejects_state_mismatch() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move { wait_for_callback(&listener, "expected").await });

        let mut cb = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        cb.write_all(b"GET /callback?code=x&state=attacker HTTP/1.1\r\n\r\n")
            .await
            .unwrap();
        let mut buf = String::new();
        cb.read_to_string(&mut buf).await.unwrap();

        assert!(server.await.unwrap().is_err());
    }
}
