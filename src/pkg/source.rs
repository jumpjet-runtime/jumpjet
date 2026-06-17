//! Where a dependency's component + WIT come from.
//!
//! Every source resolves to a [`FetchedPackage`] — the package's componentized
//! wasm, its exported WIT, and its concrete version. Path/git sources point at a
//! Jumpjet project (built on demand by re-invoking the CLI); http/registry sources
//! deliver a normalized bundle whose version is read back out of the component's
//! exported package name.

use std::path::{Path, PathBuf};

use color_eyre::eyre::{eyre, Result, WrapErr};
use semver::{Version, VersionReq};
use subprocess::{Exec, Redirection};

use crate::pkg::manifest::{Dependency, Manifest, PackageName};

/// A package's WIT as a flat list of `(relative path, bytes)` entries.
pub type WitTree = Vec<(PathBuf, Vec<u8>)>;

pub struct FetchedPackage {
    pub version: Version,
    pub component: Vec<u8>,
    pub wit: WitTree,
}

/// A concrete, resolved location for a single dependency.
pub enum DepSource {
    Path(PathBuf),
    Git {
        url: String,
        reference: Option<String>,
    },
    Http(String),
    Registry {
        req: VersionReq,
        registry: Option<String>,
    },
}

/// Picks the source for a `[dependencies]` entry. `base_dir` is the directory of
/// the manifest the dependency was declared in (so relative paths resolve).
pub fn resolve_source(dep: &Dependency, base_dir: &Path) -> Result<DepSource> {
    match dep {
        Dependency::Version(req) => Ok(DepSource::Registry {
            req: req.clone(),
            registry: None,
        }),
        Dependency::Detailed(d) => {
            if let Some(p) = &d.path {
                return Ok(DepSource::Path(base_dir.join(p)));
            }
            if let Some(url) = &d.git {
                let reference = d.rev.clone().or_else(|| d.tag.clone()).or_else(|| d.branch.clone());
                return Ok(DepSource::Git {
                    url: url.clone(),
                    reference,
                });
            }
            if let Some(url) = &d.url {
                return Ok(DepSource::Http(url.clone()));
            }
            let req = d
                .version
                .clone()
                .ok_or_else(|| eyre!("dependency needs one of: version, path, git, or url"))?;
            Ok(DepSource::Registry {
                req,
                registry: d.registry.clone(),
            })
        }
    }
}

impl DepSource {
    /// A stable string describing this source, recorded in `jumpjet.lock`.
    pub fn lock_string(&self) -> String {
        match self {
            DepSource::Path(p) => format!("path+{}", p.display()),
            DepSource::Git { url, reference } => match reference {
                Some(r) => format!("git+{url}#{r}"),
                None => format!("git+{url}"),
            },
            DepSource::Http(url) => format!("url+{url}"),
            DepSource::Registry { registry, .. } => match registry {
                Some(r) => format!("registry+{r}"),
                None => "registry".to_string(),
            },
        }
    }

    pub async fn fetch(&self, name: &PackageName) -> Result<FetchedPackage> {
        match self {
            DepSource::Path(dir) => fetch_from_project(dir, name),
            DepSource::Git { url, reference } => fetch_from_git(url, reference.as_deref(), name),
            DepSource::Http(url) => fetch_from_http(url, name).await,
            DepSource::Registry { req, registry } => {
                fetch_from_registry(name, req, registry.as_deref())
            }
        }
    }
}

/// Fetches a published package component from a registry via the `wkg` CLI (the
/// wasm-pkg-tools tool), which resolves namespace→registry mappings from the
/// shared `~/.config/wasm-pkg/config.toml`.
fn fetch_from_registry(
    name: &PackageName,
    req: &VersionReq,
    registry: Option<&str>,
) -> Result<FetchedPackage> {
    let version = exact_version(req).ok_or_else(|| {
        eyre!(
            "registry dependency `{name}` needs an exact version (e.g. `0.2.0`), found `{req}`"
        )
    })?;

    let tmp = tempfile::tempdir()?;
    let out = tmp.path().join("component.wasm");

    let mut cmd = Exec::cmd("wkg")
        .arg("get")
        .arg(format!("{name}@{version}"))
        .arg("--output")
        .arg(&out);
    if let Some(registry) = registry {
        cmd = cmd.args(&["--registry", registry]);
    }

    let cap = cmd
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture();
    let cap = match cap {
        Ok(c) => c,
        Err(_) => {
            return Err(eyre!(
                "`wkg` was not found on your PATH. Install wasm-pkg-tools to use registry dependencies: https://github.com/bytecodealliance/wasm-pkg-tools"
            ))
        }
    };
    if !cap.success() {
        return Err(eyre!("`wkg get {name}@{version}` failed:\n{}", cap.stdout_str()));
    }

    let component = std::fs::read(&out)
        .map_err(|_| eyre!("`wkg get {name}@{version}` produced no component"))?;
    let resolved = component_export_version(&component, name)?;
    let wit_text = interface_wit(&component, name)?;
    let wit = vec![(PathBuf::from(format!("{}.wit", name.name)), wit_text.into_bytes())];

    Ok(FetchedPackage {
        version: resolved,
        component,
        wit,
    })
}

/// Extracts a single concrete version from a requirement like `0.2.0`/`=0.2.0`.
/// Returns `None` for open ranges, which need registry version listing we don't do.
fn exact_version(req: &VersionReq) -> Option<Version> {
    if req.comparators.len() != 1 {
        return None;
    }
    let c = &req.comparators[0];
    Some(Version::new(c.major, c.minor?, c.patch?))
}

/// Prints the public WIT for `name`'s package as encoded in `component` — a
/// self-contained, interface-only package with no back-reference to
/// `jumpjet:runtime`.
pub fn interface_wit(component: &[u8], name: &PackageName) -> Result<String> {
    let decoded =
        wit_component::decode(component).map_err(|e| eyre!("decoding component: {e:#}"))?;
    let resolve = match decoded {
        wit_component::DecodedWasm::Component(resolve, _) => resolve,
        wit_component::DecodedWasm::WitPackage(resolve, _) => resolve,
    };

    let pkg_id = resolve
        .packages
        .iter()
        .find(|(_, p)| p.name.namespace == name.namespace && p.name.name == name.name)
        .map(|(id, _)| id)
        .ok_or_else(|| eyre!("component does not contain package `{name}`"))?;

    let mut printer = wit_component::WitPrinter::default();
    printer
        .print(&resolve, pkg_id, &[])
        .map_err(|e| eyre!("printing WIT for `{name}`: {e:#}"))?;
    Ok(printer.output.to_string())
}

/// Reads a built Jumpjet package project (building it first if needed) into a
/// [`FetchedPackage`].
fn fetch_from_project(dir: &Path, name: &PackageName) -> Result<FetchedPackage> {
    let manifest = Manifest::load_from(dir)
        .wrap_err_with(|| format!("reading dependency project at {}", dir.display()))?;

    let declared = manifest.package_name()?;
    if &declared != name {
        return Err(eyre!(
            "dependency at {} declares `{declared}` but was referenced as `{name}`",
            dir.display()
        ));
    }
    if !manifest.is_lib() {
        return Err(eyre!(
            "dependency `{name}` at {} is not a package (`type` must be `lib`)",
            dir.display()
        ));
    }

    let version = manifest
        .package
        .version
        .clone()
        .ok_or_else(|| eyre!("dependency `{name}` is missing [package].version"))?;
    let output = manifest.build.output.clone().unwrap_or_else(|| "bin".into());
    let entrypoint = manifest
        .build
        .entrypoint
        .clone()
        .ok_or_else(|| eyre!("dependency `{name}` has no [build].entrypoint"))?;

    let component_path = dir.join(&output).join(&entrypoint);
    let wit_dir = dir.join(&output).join("wit");

    if !component_path.exists() || !wit_dir.exists() {
        build_project(dir)?;
    }

    let component = std::fs::read(&component_path)
        .wrap_err_with(|| format!("reading built component {}", component_path.display()))?;
    let wit = read_wit_tree(&wit_dir)?;
    Ok(FetchedPackage {
        version,
        component,
        wit,
    })
}

/// Builds a dependency project by re-invoking this CLI's `build` in its directory,
/// reusing the exact same componentize/finalize pipeline games use.
fn build_project(dir: &Path) -> Result<()> {
    let exe = std::env::current_exe()?;
    let cap = Exec::cmd(exe)
        .arg("build")
        .cwd(dir)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .wrap_err_with(|| format!("running `jumpjet build` in {}", dir.display()))?;
    if !cap.success() {
        return Err(eyre!(
            "building dependency at {} failed:\n{}",
            dir.display(),
            cap.stdout_str()
        ));
    }
    Ok(())
}

fn fetch_from_git(url: &str, reference: Option<&str>, name: &PackageName) -> Result<FetchedPackage> {
    let tmp = tempfile::tempdir()?;
    let clone = Exec::cmd("git")
        .args(&["clone", "--quiet", url])
        .arg(tmp.path())
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .wrap_err("running git clone")?;
    if !clone.success() {
        return Err(eyre!("git clone {url} failed:\n{}", clone.stdout_str()));
    }
    if let Some(reference) = reference {
        let co = Exec::cmd("git")
            .arg("-C")
            .arg(tmp.path())
            .args(&["checkout", "--quiet", reference])
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .wrap_err("running git checkout")?;
        if !co.success() {
            return Err(eyre!("git checkout {reference} failed:\n{}", co.stdout_str()));
        }
    }
    let fetched = fetch_from_project(tmp.path(), name)?;
    drop(tmp);
    Ok(fetched)
}

async fn fetch_from_http(url: &str, name: &PackageName) -> Result<FetchedPackage> {
    let bytes = reqwest::get(url)
        .await
        .wrap_err_with(|| format!("downloading {url}"))?
        .error_for_status()
        .wrap_err_with(|| format!("downloading {url}"))?
        .bytes()
        .await?
        .to_vec();

    let tmp = tempfile::tempdir()?;
    unpack_archive(url, &bytes, tmp.path())?;
    let bundle = locate_bundle(tmp.path())?;

    let component = std::fs::read(bundle.join("component.wasm"))
        .wrap_err("bundle is missing component.wasm")?;
    let wit = read_wit_tree(&bundle.join("wit"))?;
    let version = component_export_version(&component, name)?;
    Ok(FetchedPackage {
        version,
        component,
        wit,
    })
}

/// Unpacks `.tar.gz`/`.tgz`, `.tar.xz`, or `.zip` (detected by URL extension) into
/// `dest`.
fn unpack_archive(url: &str, bytes: &[u8], dest: &Path) -> Result<()> {
    let lower = url.to_ascii_lowercase();
    if lower.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
        archive.extract(dest)?;
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        let mut ar = tar::Archive::new(xz2::read::XzDecoder::new(std::io::Cursor::new(bytes)));
        ar.unpack(dest)?;
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(std::io::Cursor::new(bytes)));
        ar.unpack(dest)?;
    } else {
        return Err(eyre!(
            "unsupported archive extension for {url} (expected .tar.gz, .tar.xz, or .zip)"
        ));
    }
    Ok(())
}

/// A bundle is a dir containing `component.wasm` + `wit/`. Tolerates archives that
/// wrap everything in a single top-level directory.
fn locate_bundle(root: &Path) -> Result<PathBuf> {
    if root.join("component.wasm").exists() {
        return Ok(root.to_path_buf());
    }
    if let Ok(entries) = std::fs::read_dir(root) {
        let dirs: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        if dirs.len() == 1 && dirs[0].join("component.wasm").exists() {
            return Ok(dirs[0].clone());
        }
    }
    Err(eyre!(
        "downloaded archive is not a package bundle (expected component.wasm + wit/ at its root)"
    ))
}

/// Reads the version of `name` from a component's exported package, used for
/// bundle sources that carry no manifest.
pub fn component_export_version(component: &[u8], name: &PackageName) -> Result<Version> {
    let decoded = wit_component::decode(component).map_err(|e| eyre!("decoding component: {e}"))?;
    let (resolve, world_id) = match decoded {
        wit_component::DecodedWasm::Component(resolve, world) => (resolve, world),
        wit_component::DecodedWasm::WitPackage(..) => {
            return Err(eyre!("expected a component, found a WIT package"))
        }
    };
    for key in resolve.worlds[world_id].exports.keys() {
        if let wit_parser::WorldKey::Interface(id) = key {
            if let Some(pkg) = resolve.interfaces[*id].package {
                let pn = &resolve.packages[pkg].name;
                if pn.namespace == name.namespace && pn.name == name.name {
                    if let Some(v) = &pn.version {
                        return Ok(v.clone());
                    }
                }
            }
        }
    }
    Err(eyre!(
        "could not determine the version of `{name}` from the component's exports"
    ))
}

/// Recursively reads every `.wit` file under `dir` into a [`WitTree`].
pub fn read_wit_tree(dir: &Path) -> Result<WitTree> {
    let mut out = WitTree::new();
    read_wit_tree_into(dir, dir, &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn read_wit_tree_into(root: &Path, dir: &Path, out: &mut WitTree) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .wrap_err_with(|| format!("reading WIT directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_wit_tree_into(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("wit") {
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            out.push((rel, std::fs::read(&path)?));
        }
    }
    Ok(())
}

/// Writes a [`WitTree`] out under `dir`.
pub fn write_wit_tree(dir: &Path, tree: &WitTree) -> Result<()> {
    for (rel, bytes) in tree {
        let dest = dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, bytes)?;
    }
    Ok(())
}
