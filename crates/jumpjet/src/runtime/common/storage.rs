use vfs::{AltrootFS, VfsPath};

pub enum Storage {
    Local(VfsPath, AltrootFS),
    Cloud,
}
