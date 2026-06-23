//! Typed view of a `jumpjet.toml` manifest.
//!
//! `[project].type` is the project kind, either `game` or `lib`. A `game` declares
//! `[client.build]` (the client / singleplayer entrypoint) and optionally
//! `[server.build]` (a headless multiplayer server); a `lib` declares `[lib]`
//! (its `namespace:name` identifier) + `[lib.build]`. [`Manifest::primary_build`]
//! selects the entrypoint by type
//! (`[client.build]` for games, `[lib.build]` for libs); [`Manifest::server_build`]
//! returns the server component when present.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use color_eyre::eyre::{Result, WrapErr, eyre};
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
    let valid = label.split('-').all(|word| {
        !word.is_empty()
            && word
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    if !valid {
        return Err(eyre!(
            "`{label}` must be lowercase alphanumeric words separated by single dashes"
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    #[default]
    Game,
    Lib,
}

/// `[project]` — the first section of every `jumpjet.toml`. Holds the project's
/// kind and metadata, plus the remote project `id` once `jumpjet project link`/
/// `create` has linked this directory to a project in the user's account.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Project {
    #[serde(rename = "type", default)]
    pub kind: ProjectKind,
    pub version: Option<Version>,
    pub author: Option<String>,
    /// Remote project id (added by `jumpjet project link`/`create`).
    pub id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Runtime {
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Build {
    pub pre: Option<String>,
    /// Path (relative to the project root) of the built wasm component to package.
    pub entrypoint: Option<String>,
    /// Optional directory of data files shipped with the game and mounted as its
    /// local storage at runtime.
    pub assets: Option<String>,
    /// For `type = "lib"`: dir (or file) holding the package's own exported WIT.
    pub wit: Option<String>,
    pub output: Option<String>,
}

/// A buildable component, declared as `[<component>.build]` (e.g. `[client.build]`,
/// `[server.build]`). The outer table (`[client]`, `[server]`) is the component's
/// identity; `build` is how it's compiled. Component-level config (e.g. future
/// server runtime settings) would live as sibling fields here.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Component {
    #[serde(default)]
    pub build: Build,
}

/// `[lib]` — a library package component. Unlike a game, a library has a wasm-pkg
/// `identifier` (`namespace:name`) so it can be published and depended on.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Lib {
    /// wasm-pkg identity `namespace:name`.
    pub identifier: Option<String>,
    #[serde(default)]
    pub build: Build,
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
    /// `[project]` — kind/version/author and the linked project `id`. The first
    /// section of every manifest.
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub runtime: Runtime,
    /// The client (and singleplayer) component: `[client.build]`.
    pub client: Option<Component>,
    /// The optional headless server component (multiplayer): `[server.build]`.
    pub server: Option<Component>,
    /// The library component for `type = "lib"` packages: `[lib]` + `[lib.build]`.
    pub lib: Option<Lib>,
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

    /// The build config for the project's primary component, selected by
    /// `[project].type`: `[client.build]` for games, `[lib.build]` for libs.
    pub fn primary_build(&self) -> Result<&Build> {
        match self.project.kind {
            ProjectKind::Game => self
                .client
                .as_ref()
                .map(|c| &c.build)
                .ok_or_else(|| eyre!("missing [client.build] section in jumpjet.toml")),
            ProjectKind::Lib => self
                .lib
                .as_ref()
                .map(|c| &c.build)
                .ok_or_else(|| eyre!("missing [lib.build] section in jumpjet.toml")),
        }
    }

    /// The headless server component's build config, present only for multiplayer
    /// games that declare `[server.build]`.
    pub fn server_build(&self) -> Option<&Build> {
        self.server.as_ref().map(|c| &c.build)
    }

    /// The linked remote project id (`[project].id`), if this directory is linked.
    pub fn project_id(&self) -> Option<&str> {
        self.project.id.as_deref()
    }

    /// The library's wasm-pkg identity (`[lib].identifier`, `namespace:name`).
    /// Only libraries have one — games are identified by their `[project]`.
    pub fn package_name(&self) -> Result<PackageName> {
        self.lib
            .as_ref()
            .and_then(|l| l.identifier.as_deref())
            .ok_or_else(|| eyre!("[lib].identifier (\"namespace:name\") is required"))?
            .parse()
    }

    pub fn is_lib(&self) -> bool {
        self.project.kind == ProjectKind::Lib
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
            [project]
            type = "lib"
            version = "0.1.0"

            [lib]
            identifier = "acme:demo"

            [lib.build]
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
    fn multiplayer_game_manifest_resolves_components() {
        let m = Manifest::parse(
            r#"
            [project]
            type = "game"
            version = "0.1.0"
            author = ""
            id = "7203549128374"

            [runtime]
            version = "0.1.0"

            [client.build]
            pre = "cargo build -p client --target wasm32-wasip2"
            entrypoint = "./target/wasm32-wasip2/debug/client.wasm"
            output = "./bin"

            [server.build]
            pre = "cargo build -p server --target wasm32-wasip2"
            entrypoint = "./target/wasm32-wasip2/debug/server.wasm"

            [bundle]
            name = "My Game"
        "#,
        )
        .unwrap();
        assert!(!m.is_lib());
        assert_eq!(m.project_id(), Some("7203549128374"));
        assert!(m.extra.contains_key("bundle"));

        // `[client.build]` is the primary component.
        let primary = m.primary_build().unwrap();
        assert_eq!(
            primary.entrypoint.as_deref(),
            Some("./target/wasm32-wasip2/debug/client.wasm")
        );
        // `[server.build]` is exposed separately.
        let server = m.server_build().expect("server build present");
        assert_eq!(
            server.entrypoint.as_deref(),
            Some("./target/wasm32-wasip2/debug/server.wasm")
        );
    }

    #[test]
    fn singleplayer_game_has_no_server() {
        let m = Manifest::parse(
            r#"
            [project]
            type = "game"

            [client.build]
            entrypoint = "./solo.wasm"
            output = "./bin"
        "#,
        )
        .unwrap();
        assert!(m.primary_build().is_ok());
        assert!(m.server_build().is_none());
        assert!(m.project_id().is_none());
    }

    #[test]
    fn lib_manifest_resolves_primary() {
        let m = Manifest::parse(
            r#"
            [project]
            type = "lib"
            version = "0.1.0"

            [lib]
            identifier = "acme:physics"

            [lib.build]
            entrypoint = "./physics.wasm"
            wit = "./wit"
            output = "./bin"
        "#,
        )
        .unwrap();
        assert!(m.is_lib());
        assert_eq!(m.package_name().unwrap().to_string(), "acme:physics");
        assert_eq!(m.primary_build().unwrap().wit.as_deref(), Some("./wit"));
        assert!(m.server_build().is_none());
    }

    #[test]
    fn missing_primary_build_errors() {
        let m = Manifest::parse(
            r#"
            [project]
            type = "game"
        "#,
        )
        .unwrap();
        assert!(m.primary_build().is_err());
    }
}
