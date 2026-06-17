use color_eyre::eyre::{eyre, Result};
use toml_edit::{value, DocumentMut, InlineTable, Item, Table};

use crate::pkg::manifest::{Manifest, PackageName};
use crate::Result as CrateResult;

#[derive(Default)]
pub struct AddOptions {
    pub path: Option<String>,
    pub git: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,
}

/// Adds a dependency to `jumpjet.toml` (preserving formatting/comments) and
/// resolves it. `spec` is `namespace:name[@version]`.
pub async fn add(spec: &str, opts: AddOptions) -> CrateResult<()> {
    let (name_str, version) = match spec.split_once('@') {
        Some((n, v)) => (n, Some(v.to_string())),
        None => (spec, None),
    };
    let name: PackageName = name_str.parse()?;
    let key = name.to_string();

    let dir = std::env::current_dir()?;
    let toml_path = dir.join(Manifest::FILE_NAME);
    let text = std::fs::read_to_string(&toml_path)?;
    let mut doc: DocumentMut = text.parse().map_err(|e| eyre!("parsing jumpjet.toml: {e}"))?;

    if doc.get("dependencies").is_none() {
        doc["dependencies"] = Item::Table(Table::new());
    }
    let deps = doc["dependencies"]
        .as_table_mut()
        .ok_or_else(|| eyre!("[dependencies] in jumpjet.toml is not a table"))?;

    deps[&key] = dependency_value(&name, version, &opts)?;
    std::fs::write(&toml_path, doc.to_string())?;
    println!("Added `{key}` to jumpjet.toml");

    crate::pkg::resolve::resolve(&dir, false).await?;
    Ok(())
}

fn dependency_value(
    name: &PackageName,
    version: Option<String>,
    opts: &AddOptions,
) -> Result<Item> {
    if let Some(path) = &opts.path {
        let mut t = InlineTable::new();
        t.insert("path", path.clone().into());
        return Ok(value(t));
    }
    if let Some(git) = &opts.git {
        let mut t = InlineTable::new();
        t.insert("git", git.clone().into());
        if let Some(tag) = &opts.tag {
            t.insert("tag", tag.clone().into());
        }
        if let Some(branch) = &opts.branch {
            t.insert("branch", branch.clone().into());
        }
        if let Some(rev) = &opts.rev {
            t.insert("rev", rev.clone().into());
        }
        return Ok(value(t));
    }
    if let Some(url) = &opts.url {
        let mut t = InlineTable::new();
        t.insert("url", url.clone().into());
        return Ok(value(t));
    }

    // Registry dependency — needs a version.
    let version = version.ok_or_else(|| {
        eyre!("registry dependency `{name}` requires a version (`{name}@<version>`)")
    })?;
    Ok(value(version))
}
