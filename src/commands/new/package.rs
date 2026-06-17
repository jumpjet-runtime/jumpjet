use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use color_eyre::eyre::eyre;

use crate::{
    assets::{JumpjetRuntimeWits, Templates},
    pkg::manifest::PackageName,
    Result,
};

use super::game::{template_files, to_snake_case};

/// Scaffolds a new Jumpjet package (a `type = "lib"` project) from a template.
pub async fn package(name: &str, template: &str) -> Result<()> {
    let pkg_name: PackageName = name.parse()?;
    let template_key = template.to_owned();

    let paths = Templates::iter()
        .filter(|p| p.starts_with(&format!("package/{template_key}/")))
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Err(eyre!(
            "unknown package template `{template_key}` (available: lib-rust)"
        ));
    }

    let project_root = env::current_dir()?;

    let globals = liquid::object!({
        "name": pkg_name.to_string(),
        "namespace": pkg_name.namespace.clone(),
        "package": pkg_name.name.clone(),
        "namespace_snake": to_snake_case(&pkg_name.namespace),
        "package_snake": to_snake_case(&pkg_name.name),
        "runtime_version": env!("CARGO_PKG_VERSION"),
    });

    template_files("package", &template_key, project_root.as_path(), paths, &globals)?;

    // Stage the Jumpjet runtime WIT as a dependency of the package's own WIT so
    // `wit-bindgen`/`jco` can resolve `import jumpjet:runtime/*`.
    copy_runtime_wit(&project_root.join("wit/deps/jumpjet-runtime"))?;

    Ok(())
}

/// Copies the `jumpjet:runtime` WIT package (every `crates/jumpjet/wit/runtime`
/// file) into `dest`, used both when scaffolding a package and when staging a
/// resolved dependency's transitive runtime dep.
pub fn copy_runtime_wit(dest: &Path) -> Result<()> {
    for wit_path in JumpjetRuntimeWits::iter() {
        let contents = JumpjetRuntimeWits::get(wit_path.as_ref()).unwrap();
        let destination_path = dest.join(PathBuf::from(wit_path.as_ref()));
        if let Some(parent) = destination_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&destination_path)?;
        file.write_all(&contents.data)?;
    }
    Ok(())
}
