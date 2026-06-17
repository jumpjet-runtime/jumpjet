//! `jumpjet.lock` — the resolved, reproducible set of dependencies for a project.

use std::path::Path;

use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "jumpjet.lock";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lock {
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// `namespace:name`
    pub name: String,
    pub version: String,
    /// e.g. `path+../greeter`, `git+https://...#v1`, `url+https://...`, `registry`
    pub source: String,
    /// `sha256:...` of the component bytes.
    pub integrity: String,
}

impl Lock {
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).wrap_err("parsing jumpjet.lock")
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let mut sorted = self.clone_sorted();
        sorted.packages.dedup_by(|a, b| a.name == b.name);
        let text = toml::to_string_pretty(&sorted).wrap_err("serializing jumpjet.lock")?;
        std::fs::write(dir.join(FILE_NAME), text)?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Inserts or replaces the entry for a package name.
    pub fn upsert(&mut self, entry: LockedPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == entry.name) {
            *existing = entry;
        } else {
            self.packages.push(entry);
        }
    }

    fn clone_sorted(&self) -> Lock {
        let mut packages = self.packages.clone();
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Lock { packages }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_roundtrip() {
        let mut lock = Lock::default();
        lock.upsert(LockedPackage {
            name: "acme:greeter".into(),
            version: "0.1.0".into(),
            source: "path+../greeter".into(),
            integrity: "sha256:abc".into(),
        });
        lock.upsert(LockedPackage {
            name: "acme:greeter".into(),
            version: "0.2.0".into(),
            source: "path+../greeter".into(),
            integrity: "sha256:def".into(),
        });
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.get("acme:greeter").unwrap().version, "0.2.0");

        let dir = tempfile::tempdir().unwrap();
        lock.save(dir.path()).unwrap();
        let reloaded = Lock::load(dir.path()).unwrap();
        assert_eq!(reloaded.get("acme:greeter").unwrap().version, "0.2.0");
    }
}
