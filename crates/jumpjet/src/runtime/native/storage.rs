use std::fs;
use std::io::Write;

use vfs::{AltrootFS, FileSystem, PhysicalFS, VfsPath};

use wasmtime::Result;
use wasmtime::component::Resource;

use crate::jumpjet::runtime::storage::*;
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
            Storage::Cloud => todo!(),
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
            Storage::Cloud => todo!(),
        }
    }

    async fn exists(&mut self, storage: Resource<StorageDevice>, path: Resource<Path>) -> bool {
        let storage = self.storages.get(storage.rep() as usize).unwrap();
        let path = self.paths.get(path.rep() as usize).unwrap();

        match storage {
            Storage::Local(_root, vfs) => vfs.exists(path.as_str()).unwrap(),
            Storage::Cloud => todo!(),
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
                let mut file = vfs.open_file(path.as_str()).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                Some(buffer)
            }
            Storage::Cloud => todo!(),
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
                let mut file = vfs.open_file(path.as_str()).unwrap();
                let mut str = String::new();
                file.read_to_string(&mut str).unwrap();
                Some(str)
            }
            Storage::Cloud => todo!(),
        }
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
                let mut file: Box<dyn Write + Send>;
                if path.exists().unwrap() {
                    file = vfs.append_file(path.as_str()).unwrap();
                } else {
                    file = vfs.create_file(path.as_str()).unwrap();
                }

                match content {
                    // WriteableContent::Stream(_) => todo!(),
                    WriteableContent::String(data) => file.write_all(data.as_bytes()).unwrap(),
                    WriteableContent::Bytes(bytes) => file.write_all(&bytes).unwrap(),
                }
            }
            Storage::Cloud => todo!(),
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
            Storage::Cloud => todo!(),
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
            Storage::Cloud => todo!(),
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
