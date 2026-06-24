use std::{
    fs::{self, File},
    path::Path,
};

use flate2::{Compression, write::GzEncoder};

use crate::settings::Settings;

/// Packages a Linux build into `bundle/linux/`. Produces a runnable directory
/// (`<name>/` holding the binary alongside the `.jumpjet/input/` guest tree the
/// runtime loads relative to the executable) plus a distributable `<name>.tar.gz`.
pub fn bundle_project(settings: &Settings) -> crate::Result<()> {
    let out_dir = settings.current_dir.join("bundle/linux");
    let app_dir = out_dir.join(&settings.bundle_name);
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)?;
    }
    fs::create_dir_all(&app_dir)?;

    // The binary and the guest input tree must sit together: the generated host
    // wrapper reads its entrypoint from `current_exe()/.jumpjet/input/`.
    crate::fs::copy_file(
        settings.target_binary_path(),
        app_dir.join(settings.binary_name()),
    )?;
    crate::fs::copy_dir_all(&settings.build_output_dir, app_dir.join(".jumpjet/input"))?;

    let tarball = out_dir.join(format!("{}.tar.gz", settings.bundle_name));
    create_tarball(&tarball, &app_dir, &settings.bundle_name)?;

    println!("Bundled {}", tarball.display());
    Ok(())
}

/// Writes a gzip-compressed tar of `app_dir`, with every entry rooted under
/// `root_name/` so the archive extracts into a single self-contained folder.
fn create_tarball(tarball: &Path, app_dir: &Path, root_name: &str) -> crate::Result<()> {
    let file = File::create(tarball)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder.append_dir_all(root_name, app_dir)?;
    builder.into_inner()?.finish()?;
    Ok(())
}
