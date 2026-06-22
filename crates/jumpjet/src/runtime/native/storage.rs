use std::fs;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use vfs::{AltrootFS, FileSystem, PhysicalFS, VfsPath};

use wasmtime::Result;
use wasmtime::component::Resource;

use crate::jumpjet::runtime::storage::*;
use crate::runtime::common::tasks::TaskState;
use crate::runtime::storage::Storage;

use super::state::JumpjetRuntimeState;

impl Host for JumpjetRuntimeState {
    async fn local(&mut self) -> Resource<StorageDevice> {
        let app_root_path = &self.input_path;
        if !app_root_path.exists() {
            fs::create_dir(app_root_path.clone()).unwrap();
        }
        let app_root_path = VfsPath::new(PhysicalFS::new(app_root_path.clone()));
        let app_root = AltrootFS::new(app_root_path.clone());
        let storage = self
            .storages
            .insert(Storage::Local(app_root_path.clone(), app_root));
        Resource::new_own(storage as u32)
    }

    async fn cloud(&mut self) -> Option<Resource<StorageDevice>> {
        None
    }
}

impl HostStorageDevice for JumpjetRuntimeState {
    async fn create_dir(&mut self, storage: Resource<StorageDevice>, path: Resource<Path>) {
        let storage = self.storages.get(storage.rep() as usize).unwrap();

        match storage {
            Storage::Local(_root, vfs) => {
                let full_path = self.paths.get(path.rep() as usize).unwrap();
                vfs.create_dir(full_path.as_str()).unwrap();
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }

        ()
    }

    async fn list_dir(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
    ) -> Vec<Resource<Path>> {
        let storage = self.storages.get(storage.rep() as usize).unwrap();

        match storage {
            Storage::Local(root, vfs) => {
                let full_path = self.paths.get(path.rep() as usize).unwrap();
                match vfs.read_dir(full_path.as_str()) {
                    Ok(entries) => entries
                        .filter_map(|entry| {
                            Some(Resource::new_borrow(
                                self.paths.insert(root.join(entry).unwrap()) as u32,
                            ))
                        })
                        .collect(),
                    Err(err) => panic!("{}", err),
                }
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn exists(&mut self, storage: Resource<StorageDevice>, path: Resource<Path>) -> bool {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_root, vfs) => vfs.exists(path.as_str()).unwrap(),
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn read(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
    ) -> Option<Vec<u8>> {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_root, vfs) => {
                if !vfs.exists(path.as_str()).unwrap_or(false) {
                    return None;
                }
                let mut file = vfs.open_file(path.as_str()).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                Some(buffer)
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn read_string(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
    ) -> Option<String> {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_, vfs) => {
                if !vfs.exists(path.as_str()).unwrap_or(false) {
                    return None;
                }
                let mut file = vfs.open_file(path.as_str()).unwrap();
                let mut str = String::new();
                file.read_to_string(&mut str).unwrap();
                Some(str)
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn open(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
    ) -> Resource<Task> {
        let state = Arc::new(Mutex::new(TaskState::Pending));

        // Resolve to an absolute filesystem path (a `Send` `PathBuf`) so the read
        // can run off-thread; the VFS itself isn't moved into the worker.
        let abs = {
            let st = self.storages.get(storage.rep() as usize);
            let p = self.paths.get(path.rep() as usize);
            match (st, p) {
                (Some(Storage::Local(_, _)), Some(p)) => {
                    Some(self.input_path.join(p.as_str().trim_start_matches('/')))
                }
                _ => None,
            }
        };

        match abs {
            Some(abs) => {
                let worker = state.clone();
                std::thread::spawn(move || {
                    let outcome = match std::fs::read(&abs) {
                        Ok(bytes) => TaskState::Complete(bytes),
                        Err(e) => TaskState::Failed(e.to_string()),
                    };
                    *worker.lock().unwrap() = outcome;
                });
            }
            None => *state.lock().unwrap() = TaskState::Failed("path not available".into()),
        }

        self.table.push(Task { state }).unwrap()
    }

    async fn write(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
        content: WriteableContent,
    ) {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_, vfs) => {
                // WIT contract: overwrite existing files and create parent directories
                // as needed. `create_file` truncates, so it covers the overwrite case.
                path.parent().create_dir_all().unwrap();
                let mut file = vfs.create_file(path.as_str()).unwrap();

                match content {
                    // WriteableContent::Stream(_) => todo!(),
                    WriteableContent::String(data) => file.write_all(data.as_bytes()).unwrap(),
                    WriteableContent::Bytes(bytes) => file.write_all(&bytes).unwrap(),
                }
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn remove(
        &mut self,
        storage: Resource<StorageDevice>,
        path: Resource<Path>,
    ) -> Option<bool> {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_, vfs) => {
                if path.is_root() {
                    None
                } else if path.is_dir().unwrap() {
                    if vfs.remove_dir(path.as_str()).is_ok() {
                        Some(true)
                    } else {
                        Some(false)
                    }
                } else {
                    if vfs.remove_file(path.as_str()).is_ok() {
                        Some(true)
                    } else {
                        Some(false)
                    }
                }
            }
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        }
    }

    async fn drop(&mut self, rep: Resource<StorageDevice>) -> Result<()> {
        self.storages.remove(rep.rep() as usize);
        Ok(())
    }
}

impl HostPath for JumpjetRuntimeState {
    async fn new(&mut self, storage: Resource<StorageDevice>, path: String) -> Resource<Path> {
        let storage = self.storages.get(storage.rep() as usize).unwrap();

        Resource::new_own(match storage {
            Storage::Local(root, _) => self.paths.insert(root.join(path).unwrap()) as u32,
            Storage::Cloud => unreachable!("cloud storage is not yet available"),
        })
    }

    async fn to_string(&mut self, res: Resource<Path>) -> String {
        let path = self.paths.get(res.rep() as usize).unwrap();
        path.as_str().to_owned()
    }

    async fn is_dir(&mut self, res: Resource<Path>) -> bool {
        let path = self.paths.get(res.rep() as usize).unwrap();
        path.is_dir().unwrap()
    }

    async fn is_file(&mut self, res: Resource<Path>) -> bool {
        let path = self.paths.get(res.rep() as usize).unwrap();
        path.is_file().unwrap()
    }

    async fn is_root(&mut self, res: Resource<Path>) -> bool {
        let path = self.paths.get(res.rep() as usize).unwrap();
        path.is_root()
    }

    async fn extension(&mut self, res: Resource<Path>) -> Option<String> {
        let path = self.paths.get(res.rep() as usize).unwrap();
        path.extension()
    }

    async fn filename(&mut self, res: Resource<Path>) -> Option<String> {
        let path = self.paths.get(res.rep() as usize).unwrap();
        if path.is_file().unwrap() {
            Some(path.filename())
        } else {
            None
        }
    }

    async fn join(&mut self, res: Resource<Path>, path: String) -> Resource<Path> {
        let parent = self.paths.get(res.rep() as usize).unwrap();
        Resource::new_own(self.paths.insert(parent.join(path).unwrap()) as u32)
    }

    async fn parent(&mut self, res: Resource<Path>) -> Resource<Path> {
        let path = self.paths.get(res.rep() as usize).unwrap();
        Resource::new_own(self.paths.insert(path.parent()) as u32)
    }

    async fn drop(&mut self, rep: Resource<Path>) -> Result<()> {
        self.paths.remove(rep.rep() as usize);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Mirrors the exact vfs operations used by the `read`/`write` host functions to
    // verify the WIT contract: writes overwrite (not append), reads of a missing path
    // resolve to `None`, and parent directories are created on write.
    fn temp_root() -> (std::path::PathBuf, VfsPath, AltrootFS) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("jumpjet-storage-test-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let root = VfsPath::new(PhysicalFS::new(dir.clone()));
        let vfs = AltrootFS::new(root.clone());
        (dir, root, vfs)
    }

    fn write(root: &VfsPath, vfs: &AltrootFS, path: &str, data: &[u8]) {
        let full = root.join(path).unwrap();
        full.parent().create_dir_all().unwrap();
        let mut file = vfs.create_file(full.as_str()).unwrap();
        file.write_all(data).unwrap();
    }

    fn read(vfs: &AltrootFS, path: &str) -> Option<Vec<u8>> {
        if !vfs.exists(path).unwrap_or(false) {
            return None;
        }
        use std::io::Read;
        let mut file = vfs.open_file(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        Some(buf)
    }

    #[test]
    fn write_overwrites_existing_file() {
        let (dir, root, vfs) = temp_root();
        let full = root.join("save.dat").unwrap();

        write(&root, &vfs, "save.dat", b"first");
        write(&root, &vfs, "save.dat", b"second");

        assert_eq!(read(&vfs, full.as_str()), Some(b"second".to_vec()));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_missing_path_is_none() {
        let (dir, root, vfs) = temp_root();
        let full = root.join("does-not-exist.dat").unwrap();

        assert_eq!(read(&vfs, full.as_str()), None);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_creates_parent_directories() {
        let (dir, root, vfs) = temp_root();
        let full = root.join("nested/deep/save.dat").unwrap();

        write(&root, &vfs, "nested/deep/save.dat", b"data");

        assert_eq!(read(&vfs, full.as_str()), Some(b"data".to_vec()));
        fs::remove_dir_all(&dir).ok();
    }
}
