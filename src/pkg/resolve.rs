//! Turns a manifest's `[dependencies]` into a concrete set of stored packages,
//! updating `jumpjet.lock` as it goes.
//!
//! Path and git dependencies are treated as live (always re-read, building the
//! project on demand) since they're development sources; immutable bundle sources
//! (url/registry) are served from the [`store`](crate::pkg::store) cache when the
//! lock already pins them.

use std::collections::BTreeSet;
use std::path::Path;

use color_eyre::eyre::{Result, WrapErr};

use crate::pkg::lock::{Lock, LockedPackage};
use crate::pkg::manifest::{Manifest, PackageId, PackageName};
use crate::pkg::source::{resolve_source, DepSource};
use crate::pkg::store::{self, StoredPackage};

pub struct ResolvedPackage {
    pub id: PackageId,
    pub stored: StoredPackage,
}

pub struct Resolution {
    pub packages: Vec<ResolvedPackage>,
}

/// Resolves all dependencies of the project in `dir`. When `update` is false the
/// existing lock is honored for immutable sources; when true the lock is rebuilt.
pub async fn resolve(dir: &Path, update: bool) -> Result<Resolution> {
    let manifest = Manifest::load_from(dir)?;
    let mut lock = if update { Lock::default() } else { Lock::load(dir)? };

    let mut packages = Vec::new();
    let mut seen = BTreeSet::new();
    resolve_into(dir, &manifest, update, &mut lock, &mut packages, &mut seen).await?;

    // Don't litter dependency-free projects with an empty lockfile.
    if lock.packages.is_empty() {
        let lock_path = dir.join(crate::pkg::lock::FILE_NAME);
        if lock_path.exists() {
            std::fs::remove_file(lock_path)?;
        }
    } else {
        lock.save(dir)?;
    }
    Ok(Resolution { packages })
}

async fn resolve_into(
    base_dir: &Path,
    manifest: &Manifest,
    update: bool,
    lock: &mut Lock,
    packages: &mut Vec<ResolvedPackage>,
    seen: &mut BTreeSet<String>,
) -> Result<()> {
    for (key, dep) in &manifest.dependencies {
        let name: PackageName = key
            .parse()
            .wrap_err_with(|| format!("invalid dependency key `{key}`"))?;
        if !seen.insert(name.to_string()) {
            continue;
        }

        let source = resolve_source(dep, base_dir)?;

        // Reuse the cached bundle for immutable sources already pinned by the lock.
        if !update && is_immutable(&source) {
            if let Some(locked) = lock.get(&name.to_string()).cloned() {
                if let Ok(version) = locked.version.parse() {
                    let id = PackageId::new(name.clone(), version);
                    if let Some(stored) = store::get(&id) {
                        if stored.integrity == locked.integrity {
                            packages.push(ResolvedPackage { id, stored });
                            continue;
                        }
                    }
                }
            }
        }

        let fetched = source
            .fetch(&name)
            .await
            .wrap_err_with(|| format!("fetching dependency `{name}`"))?;
        let id = PackageId::new(name.clone(), fetched.version.clone());
        let stored = store::put(&id, &fetched)?;

        lock.upsert(LockedPackage {
            name: name.to_string(),
            version: id.version.to_string(),
            source: source.lock_string(),
            integrity: stored.integrity.clone(),
        });

        // Walk transitive dependencies of live project sources.
        if let DepSource::Path(child_dir) = &source {
            if let Ok(child) = Manifest::load_from(child_dir) {
                Box::pin(resolve_into(child_dir, &child, update, lock, packages, seen)).await?;
            }
        }

        packages.push(ResolvedPackage { id, stored });
    }
    Ok(())
}

fn is_immutable(source: &DepSource) -> bool {
    matches!(source, DepSource::Http(_) | DepSource::Registry { .. })
}
