//! Content cache for fetched packages, shared across projects.
//!
//! Layout under the user data dir:
//! `packages/<namespace>/<name>/<version>/{component.wasm, wit/**}`.

use std::path::PathBuf;

use color_eyre::eyre::Result;
use sha2::{Digest, Sha256};

use crate::pkg::manifest::PackageId;
use crate::pkg::source::{read_wit_tree, write_wit_tree, FetchedPackage, WitTree};

pub fn packages_root() -> PathBuf {
    crate::utils::get_data_dir().join("packages")
}

pub fn dir_for(id: &PackageId) -> PathBuf {
    packages_root()
        .join(&id.name.namespace)
        .join(&id.name.name)
        .join(id.version.to_string())
}

pub fn integrity_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

pub struct StoredPackage {
    pub id: PackageId,
    pub dir: PathBuf,
    pub integrity: String,
}

impl StoredPackage {
    pub fn component_path(&self) -> PathBuf {
        self.dir.join("component.wasm")
    }

    pub fn wit_dir(&self) -> PathBuf {
        self.dir.join("wit")
    }

    pub fn read_component(&self) -> Result<Vec<u8>> {
        Ok(std::fs::read(self.component_path())?)
    }

    pub fn read_wit(&self) -> Result<WitTree> {
        read_wit_tree(&self.wit_dir())
    }
}

/// Writes a fetched package into the store (overwriting any existing copy of that
/// exact version) and returns its handle.
pub fn put(id: &PackageId, fetched: &FetchedPackage) -> Result<StoredPackage> {
    let dir = dir_for(id);
    std::fs::create_dir_all(&dir)?;

    let component_path = dir.join("component.wasm");
    std::fs::write(&component_path, &fetched.component)?;

    let wit_dir = dir.join("wit");
    if wit_dir.exists() {
        std::fs::remove_dir_all(&wit_dir)?;
    }
    write_wit_tree(&wit_dir, &fetched.wit)?;

    Ok(StoredPackage {
        id: id.clone(),
        dir,
        integrity: integrity_of(&fetched.component),
    })
}

/// Returns a handle to an already-stored package, if present.
pub fn get(id: &PackageId) -> Option<StoredPackage> {
    let dir = dir_for(id);
    let component_path = dir.join("component.wasm");
    if !component_path.exists() || !dir.join("wit").exists() {
        return None;
    }
    let bytes = std::fs::read(&component_path).ok()?;
    Some(StoredPackage {
        id: id.clone(),
        dir,
        integrity: integrity_of(&bytes),
    })
}
