//! Typed view of a `jumpjet.toml` manifest.
//!
//! The existing build pipeline (`src/commands/build.rs`) still reads the manifest
//! as a raw [`toml::Table`] for the fields it already understood; this module adds
//! a typed layer used by the package manager for package identity and the
//! `[dependencies]` table. The two coexist — nothing here changes how games build.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use color_eyre::eyre::{eyre, Result, WrapErr};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// A wasm-pkg style package name: `namespace:name` (e.g. `acme:physics`).
///
/// This mirrors `wasm_pkg_common::package::PackageRef` so identities map cleanly
/// onto wasm-pkg-tools / WIT package names when the registry source lands.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageName {
    pub namespace: String,
    pub name: String,
}

impl PackageName {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Result<Self> {
        let namespace = namespace.into();
        let name = name.into();
        validate_label(&namespace).wrap_err("invalid package namespace")?;
        validate_label(&name).wrap_err("invalid package name")?;
        Ok(Self { namespace, name })
    }
}

impl FromStr for PackageName {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (namespace, name) = s
            .split_once(':')
            .ok_or_else(|| eyre!("package name must be `namespace:name`, got `{s}`"))?;
        PackageName::new(namespace, name)
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

/// A fully resolved package identity: `namespace:name@version`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageId {
    pub name: PackageName,
    pub version: Version,
}

impl PackageId {
    pub fn new(name: PackageName, version: Version) -> Self {
        Self { name, version }
    }
}

impl FromStr for PackageId {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (name, version) = s
            .split_once('@')
            .ok_or_else(|| eyre!("package id must be `namespace:name@version`, got `{s}`"))?;
        Ok(PackageId {
            name: name.parse()?,
            version: Version::parse(version)
                .wrap_err_with(|| format!("invalid version in `{s}`"))?,
        })
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Labels (namespace / name segments) follow the WIT identifier rules wasm-pkg
/// enforces: lowercase alphanumeric words separated by single dashes.
fn validate_label(label: &str) -> Result<()> {
    if label.is_empty() {
        return Err(eyre!("label must not be empty"));
    }
    let valid = label
        .split('-')
        .all(|word| !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    if !valid {
        return Err(eyre!(
            "`{label}` must be lowercase alphanumeric words separated by single dashes"
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    #[default]
    Game,
    Lib,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Package {
    /// wasm-pkg identity `namespace:name`. Required for `type = "lib"`.
    pub name: Option<String>,
    /// Legacy flat identifier used by game templates.
    pub identifier: Option<String>,
    pub version: Option<Version>,
    #[serde(rename = "type", default)]
    pub kind: PackageKind,
    pub author: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Runtime {
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Build {
    pub pre: Option<String>,
    pub input: Option<String>,
    pub entrypoint: Option<String>,
    /// For `type = "lib"`: dir (or file) holding the package's own exported WIT.
    pub wit: Option<String>,
    pub output: Option<String>,
}

/// A `[dependencies]` entry, either a bare version requirement or a detailed table.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    /// `"acme:math" = "0.2.0"`
    Version(VersionReq),
    /// `"acme:math" = { version = "0.2.0", path = "../math" }`
    Detailed(DetailedDependency),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DetailedDependency {
    pub version: Option<VersionReq>,
    /// Registry override (namespace→registry mappings otherwise come from the
    /// shared `~/.config/wasm-pkg/config.toml`).
    pub registry: Option<String>,
    pub path: Option<PathBuf>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub url: Option<String>,
}

impl Dependency {
    pub fn version_req(&self) -> Option<&VersionReq> {
        match self {
            Dependency::Version(req) => Some(req),
            Dependency::Detailed(d) => d.version.as_ref(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub runtime: Runtime,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
    /// `[bundle]` (and any other sections) are preserved opaquely so a typed
    /// round-trip doesn't drop fields the package manager doesn't care about.
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl Manifest {
    pub const FILE_NAME: &'static str = "jumpjet.toml";

    /// Loads and parses `jumpjet.toml` from `dir`.
    pub fn load_from(dir: &Path) -> Result<Self> {
        let path = dir.join(Self::FILE_NAME);
        let text = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("reading {}", path.display()))?;
        Self::parse(&text)
    }

    /// Loads `jumpjet.toml` from the current working directory.
    pub fn load() -> Result<Self> {
        Self::load_from(&std::env::current_dir()?)
    }

    pub fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).wrap_err("parsing jumpjet.toml")
    }

    /// The package's wasm-pkg identity. Prefers `name = "namespace:name"`, falling
    /// back to the legacy `identifier` under a `game:` namespace for games.
    pub fn package_name(&self) -> Result<PackageName> {
        if let Some(name) = &self.package.name {
            return name.parse();
        }
        if let Some(id) = &self.package.identifier {
            return PackageName::new("game", id.replace('_', "-"));
        }
        Err(eyre!(
            "[package] must set `name = \"namespace:name\"` (or legacy `identifier`)"
        ))
    }

    pub fn is_lib(&self) -> bool {
        self.package.kind == PackageKind::Lib
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_roundtrip() {
        let n: PackageName = "acme:physics".parse().unwrap();
        assert_eq!(n.namespace, "acme");
        assert_eq!(n.name, "physics");
        assert_eq!(n.to_string(), "acme:physics");
    }

    #[test]
    fn package_id_roundtrip() {
        let id: PackageId = "acme:physics@0.1.0".parse().unwrap();
        assert_eq!(id.to_string(), "acme:physics@0.1.0");
        assert_eq!(id.version, Version::new(0, 1, 0));
    }

    #[test]
    fn rejects_bad_names() {
        assert!("nocolon".parse::<PackageName>().is_err());
        assert!("Acme:physics".parse::<PackageName>().is_err());
        assert!("acme:".parse::<PackageName>().is_err());
        assert!("acme:phys_ics".parse::<PackageName>().is_err());
    }

    #[test]
    fn parses_dependencies_both_shapes() {
        let m = Manifest::parse(
            r#"
            [package]
            name = "acme:demo"
            version = "0.1.0"
            type = "lib"

            [build]
            entrypoint = "demo.wasm"
            wit = "wit"

            [dependencies]
            "acme:math" = "0.2.0"
            "acme:physics" = { version = "0.1.0", path = "../physics" }
        "#,
        )
        .unwrap();
        assert!(m.is_lib());
        assert_eq!(m.package_name().unwrap().to_string(), "acme:demo");
        assert_eq!(m.dependencies.len(), 2);
        assert!(m.dependencies["acme:math"].version_req().is_some());
        match &m.dependencies["acme:physics"] {
            Dependency::Detailed(d) => assert!(d.path.is_some()),
            _ => panic!("expected detailed dependency"),
        }
    }

    #[test]
    fn legacy_game_manifest_still_parses() {
        let m = Manifest::parse(
            r#"
            [package]
            identifier = "my-game"
            version = "0.1.0"
            type = "game"
            author = ""

            [runtime]
            version = "0.1.0"

            [build]
            pre = "cargo build --target wasm32-wasip2"
            input = "./target/wasm32-wasip2/debug"
            entrypoint = "my_game.wasm"
            output = "./bin"

            [bundle]
            name = "My Game"
        "#,
        )
        .unwrap();
        assert!(!m.is_lib());
        assert_eq!(m.package_name().unwrap().to_string(), "game:my-game");
        assert!(m.dependencies.is_empty());
        assert!(m.extra.contains_key("bundle"));
    }
}
