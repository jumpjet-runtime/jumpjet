//! `jumpjet project list | create <name> | link [id]`.
//!
//! Links the current directory to a project in the user's Jumpjet account (a
//! remote record on the backend, see `jumpjet-web/backend` `systems/projects`).
//! `create` makes a new project via the API and links it; `link` links an existing
//! one (interactively if no id is given). The link is persisted as `[project].id`
//! in `jumpjet.toml`. All calls authenticate with the CLI bearer token from
//! `jumpjet auth signin`.

use std::io::{self, Write};

use color_eyre::eyre::{Result, bail, eyre};
use serde::Deserialize;
use serde_json::json;
use toml_edit::{DocumentMut, Item, Table, value};

use crate::cli::ProjectSubcommand;
use crate::commands::auth;
use crate::pkg::manifest::Manifest;

/// A project as returned by `GET`/`POST /api/projects`. The backend serializes the
/// snowflake `id` as a string.
#[derive(Debug, Deserialize)]
struct ApiProject {
    id: String,
    name: String,
    #[serde(default)]
    created_at: Option<String>,
}

pub async fn project(sub: &ProjectSubcommand) -> Result<()> {
    match sub {
        ProjectSubcommand::List => list().await,
        ProjectSubcommand::Create { name } => create(name).await,
        ProjectSubcommand::Link { id } => link(id.clone()).await,
    }
}

async fn list() -> Result<()> {
    let projects = list_projects().await?;
    if projects.is_empty() {
        println!("No projects yet. Create one with `jumpjet project create <name>`.");
        return Ok(());
    }
    let linked = current_link();
    println!("Your projects:");
    for p in &projects {
        let mark = if Some(p.id.as_str()) == linked.as_deref() {
            "  (linked)"
        } else {
            ""
        };
        println!("  {}  {}{}{}", p.id, p.name, created_suffix(p), mark);
    }
    Ok(())
}

async fn create(name: &str) -> Result<()> {
    let p = create_project(name).await?;
    write_project_id(&p.id)?;
    println!("✓ Created project '{}' ({}) and linked it.", p.name, p.id);
    Ok(())
}

async fn link(id: Option<String>) -> Result<()> {
    let id = match id {
        Some(id) => id,
        None => select_interactively().await?,
    };
    write_project_id(&id)?;
    println!("✓ Linked to project {id}");
    Ok(())
}

/// Lists existing projects and prompts the user to pick one (or create a new one).
async fn select_interactively() -> Result<String> {
    let projects = list_projects().await?;
    if projects.is_empty() {
        println!("You have no projects yet.");
        return create_via_prompt().await;
    }

    println!("Your projects:");
    for (i, p) in projects.iter().enumerate() {
        println!("  {}) {}{}", i + 1, p.name, created_suffix(p));
    }
    let choice = prompt(&format!(
        "Select a project [1-{}], or 'n' to create: ",
        projects.len()
    ))?;
    let choice = choice.trim();

    if choice.eq_ignore_ascii_case("n") {
        return create_via_prompt().await;
    }

    let idx: usize = choice
        .parse()
        .ok()
        .and_then(|n: usize| n.checked_sub(1))
        .filter(|&i| i < projects.len())
        .ok_or_else(|| eyre!("invalid selection: {choice}"))?;
    Ok(projects[idx].id.clone())
}

async fn create_via_prompt() -> Result<String> {
    let name = prompt("New project name: ")?;
    let name = name.trim();
    if name.is_empty() {
        bail!("project name cannot be empty");
    }
    let p = create_project(name).await?;
    println!("✓ Created project '{}' ({}).", p.name, p.id);
    Ok(p.id)
}

// ---- API -------------------------------------------------------------------

async fn list_projects() -> Result<Vec<ApiProject>> {
    let token = require_token()?;
    let url = format!("{}/api/projects", auth::base_url());
    let resp = reqwest::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;
    parse_json(resp, "listing projects").await
}

async fn create_project(name: &str) -> Result<ApiProject> {
    let token = require_token()?;
    let url = format!("{}/api/projects", auth::base_url());
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(token)
        .json(&json!({ "name": name }))
        .send()
        .await?;
    parse_json(resp, "creating project").await
}

/// Loads the stored bearer token, or a friendly "sign in first" error.
fn require_token() -> Result<String> {
    match auth::load_credentials()? {
        Some(c) => Ok(c.access_token),
        None => bail!("not signed in — run `jumpjet auth signin` first"),
    }
}

/// Checks the response status (with a clear message for `401`) and decodes JSON.
async fn parse_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    action: &str,
) -> Result<T> {
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        bail!("authentication failed — run `jumpjet auth signin` again");
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("{action} failed ({status}): {body}");
    }
    resp.json::<T>()
        .await
        .map_err(|e| eyre!("could not parse {action} response: {e}"))
}

// ---- jumpjet.toml ----------------------------------------------------------

/// Writes `[project].id = "<id>"` into `jumpjet.toml`, preserving formatting and
/// comments (same `toml_edit` approach as `jumpjet add`).
fn write_project_id(id: &str) -> Result<()> {
    let path = std::env::current_dir()?.join(Manifest::FILE_NAME);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| eyre!("reading {}: {e}", path.display()))?;
    std::fs::write(&path, set_project_id(&text, id)?)?;
    Ok(())
}

/// Returns `toml` with `[project].id` set to `id`, adding the section if absent and
/// otherwise preserving formatting and comments.
fn set_project_id(toml: &str, id: &str) -> Result<String> {
    let mut doc: DocumentMut = toml.parse().map_err(|e| eyre!("parsing jumpjet.toml: {e}"))?;
    if doc.get("project").is_none() {
        doc["project"] = Item::Table(Table::new());
    }
    doc["project"]["id"] = value(id);
    Ok(doc.to_string())
}

/// The project id this directory is currently linked to, if any (best effort).
fn current_link() -> Option<String> {
    Manifest::load()
        .ok()
        .and_then(|m| m.project_id().map(str::to_string))
}

// ---- prompts ---------------------------------------------------------------

/// Prints `label` (no newline) and reads a line from stdin.
fn prompt(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line)
}

/// `"  (created 2026-06-20)"` from an RFC3339 timestamp, or empty.
fn created_suffix(p: &ApiProject) -> String {
    match p.created_at.as_deref() {
        Some(ts) if ts.len() >= 10 => format!("  (created {})", &ts[..10]),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::set_project_id;
    use crate::pkg::manifest::Manifest;

    #[test]
    fn adds_project_section_preserving_content() {
        let toml = "\
# my game
[project]
type = \"game\"  # singleplayer
version = \"0.1.0\"

[client.build]
entrypoint = \"./bin/client.wasm\"
";
        let out = set_project_id(toml, "7203549128374").unwrap();
        // Comments + existing sections preserved.
        assert!(out.contains("# my game"));
        assert!(out.contains("type = \"game\"  # singleplayer"));
        assert!(out.contains("[client.build]"));
        // id added to the existing [project] section, parses back correctly.
        let m = Manifest::parse(&out).unwrap();
        assert_eq!(m.project_id(), Some("7203549128374"));
    }

    #[test]
    fn updates_existing_project_id() {
        let toml = "[project]\ntype = \"game\"\nid = \"old\"\n";
        let out = set_project_id(toml, "new").unwrap();
        assert_eq!(Manifest::parse(&out).unwrap().project_id(), Some("new"));
        assert!(!out.contains("\"old\""));
    }
}
