//! Anonymous usage analytics via the Firebase / GA4 Measurement Protocol.
//!
//! There is no native Firebase SDK for Rust, so we POST events directly to the
//! Measurement Protocol HTTP endpoint. The design goals here are, in order:
//! privacy-respecting, never blocks or slows a command, never fails a command.
//!
//! Consent model: opt-out. Analytics is on by default but a one-time notice is
//! printed, and it can be disabled via `DO_NOT_TRACK=1`, `JUMPJET_NO_ANALYTICS=1`,
//! or `"enabled": false` in `<data_dir>/analytics.json`.
//!
//! Credentials (`JUMPJET_FIREBASE_APP_ID` / `JUMPJET_FIREBASE_API_SECRET`) are
//! baked in at build time via dotenvx + `build.rs`. When either is empty (e.g. a
//! local dev build with no `.env`), analytics silently no-ops.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task::JoinHandle;
use tracing::debug;
use uuid::Uuid;

/// Firebase app id, injected at build time. Empty in builds without credentials.
const FIREBASE_APP_ID: &str = env!("JUMPJET_FIREBASE_APP_ID");
/// Measurement Protocol API secret, injected at build time. Empty when absent.
const FIREBASE_API_SECRET: &str = env!("JUMPJET_FIREBASE_API_SECRET");

const COLLECT_URL: &str = "https://www.google-analytics.com/mp/collect";
/// Upper bound on how long we'll wait for the analytics request before giving up.
const FLUSH_TIMEOUT: Duration = Duration::from_millis(1500);

/// Persisted analytics state, stored at `<data_dir>/analytics.json`.
#[derive(Debug, Serialize, Deserialize)]
struct AnalyticsState {
    /// Stable, anonymous per-install id (32 hex chars, as Firebase expects).
    app_instance_id: String,
    /// Whether analytics is enabled. Defaults to true on first run.
    enabled: bool,
    /// Whether the first-run consent notice has been shown.
    notice_shown: bool,
}

impl Default for AnalyticsState {
    fn default() -> Self {
        Self {
            app_instance_id: Uuid::new_v4().simple().to_string(),
            enabled: true,
            notice_shown: false,
        }
    }
}

fn state_path() -> std::path::PathBuf {
    crate::utils::get_data_dir().join("analytics.json")
}

fn load_state() -> AnalyticsState {
    std::fs::read_to_string(state_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &AnalyticsState) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}

/// True when the user has globally opted out via environment variable.
/// Honors the cross-tool `DO_NOT_TRACK` convention plus our own override.
fn env_opted_out() -> bool {
    let truthy = |v: String| !v.is_empty() && v != "0" && v.to_lowercase() != "false";
    std::env::var("JUMPJET_NO_ANALYTICS").map(truthy).unwrap_or(false)
        || std::env::var("DO_NOT_TRACK").map(truthy).unwrap_or(false)
}

/// True when credentials were baked into this build.
fn has_credentials() -> bool {
    !FIREBASE_APP_ID.is_empty() && !FIREBASE_API_SECRET.is_empty()
}

/// Report the one-time `install` event on a brand-new install, returning a handle
/// to await before exit (see [`track`]). The state file is only ever written by
/// analytics, so its absence is our first-run signal — this means existing users
/// (who already have `analytics.json`) never retroactively fire `install`.
///
/// Call this before [`track`] so the shared first-run setup (consent notice,
/// `app_instance_id`) is persisted once and reused by the command event.
#[must_use]
pub fn track_install() -> Option<JoinHandle<()>> {
    if !has_credentials() || env_opted_out() {
        return None;
    }

    // Genuine first run: no state has been persisted yet.
    if state_path().exists() {
        return None;
    }

    track("install", json!({ "cli_version": env!("CARGO_PKG_VERSION") }))
}

/// Report an event, returning a handle that should be awaited before exit so the
/// in-flight request can flush. Returns `None` when analytics is disabled (no
/// credentials, opted out, or first-run notice declined).
///
/// `params` are GA4 event params (string/number values). Never include PII.
#[must_use]
pub fn track(event_name: &str, params: Value) -> Option<JoinHandle<()>> {
    if !has_credentials() || env_opted_out() {
        return None;
    }

    let mut state = load_state();

    // First run with analytics active: tell the user before sending anything,
    // and persist so we only ever say it once.
    if !state.notice_shown {
        print_notice();
        state.notice_shown = true;
        save_state(&state);
    }

    if !state.enabled {
        return None;
    }

    Some(spawn_send(&state.app_instance_id, event_name, params))
}

/// Spawn the fire-and-forget Measurement Protocol request for a single event.
fn spawn_send(app_instance_id: &str, event_name: &str, mut params: Value) -> JoinHandle<()> {
    // GA4 wants engagement_time_msec on events for them to surface in reports.
    if let Value::Object(map) = &mut params {
        map.entry("engagement_time_msec").or_insert(json!("1"));
    }

    let event_name = event_name.to_string();
    let body = json!({
        "app_instance_id": app_instance_id,
        "events": [{ "name": event_name, "params": params }],
    });

    let url = format!(
        "{COLLECT_URL}?firebase_app_id={FIREBASE_APP_ID}&api_secret={FIREBASE_API_SECRET}"
    );

    tokio::spawn(async move {
        let client = match reqwest::Client::builder().timeout(FLUSH_TIMEOUT).build() {
            Ok(c) => c,
            Err(e) => {
                debug!("analytics: client build failed: {e}");
                return;
            }
        };
        // Fire-and-forget: log at debug, never surface to the user.
        match client.post(&url).json(&body).send().await {
            Ok(resp) => debug!("analytics: sent {} -> {}", event_name, resp.status()),
            Err(e) => debug!("analytics: send failed: {e}"),
        }
    })
}

/// Await an in-flight analytics request (from [`track`]) with a hard timeout so a
/// slow or hung network never delays process exit.
pub async fn flush(handle: Option<JoinHandle<()>>) {
    if let Some(handle) = handle {
        let _ = tokio::time::timeout(FLUSH_TIMEOUT, handle).await;
    }
}

fn print_notice() {
    eprintln!(
        "\x1b[2mJumpjet collects anonymous usage analytics to help improve the CLI.\n\
         No personal data or project contents are sent. Opt out any time with\n\
         DO_NOT_TRACK=1 or JUMPJET_NO_ANALYTICS=1.\x1b[0m"
    );
}
