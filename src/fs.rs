use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
};

pub fn create_file(path: &PathBuf) -> io::Result<BufWriter<File>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;

    let dst_canonical = fs::canonicalize(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            if let Ok(canon) = fs::canonicalize(entry.path()) {
                if canon == dst_canonical {
                    continue;
                }
            }

            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn copy_file_to_dir(
    source: impl AsRef<Path>,
    destination_dir: impl AsRef<Path>,
) -> io::Result<()> {
    fs::create_dir_all(&destination_dir)?;

    let source_path = source.as_ref();
    let destination_path = destination_dir
        .as_ref()
        .join(source_path.file_name().expect("Failed to get file name"));

    fs::copy(source_path, &destination_path)?;

    Ok(())
}

pub fn copy_file(source: PathBuf, destination: PathBuf) -> io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}
