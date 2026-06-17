use rust_embed::Embed;
use std::{
    borrow::Cow,
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{assets::{JumpjetWits, Templates}, Result};

use liquid::Object;

pub async fn game(
    identifier: &Option<String>,
    name: &Option<String>,
    template: &String,
) -> Result<()> {
    let identifier = identifier.clone().unwrap_or("my-game".to_owned());
    let name = name.clone().unwrap_or("My Game".to_owned());
    let template_key = template.clone();

    let paths = Templates::iter()
        .filter(|p| p.starts_with(&format!("game/{template_key}")))
        .collect::<Vec<_>>();

    let project_root_path = env::current_dir().ok().unwrap();

    let globals = liquid::object!({
        "identifier": identifier,
        "identifier_underscored": to_snake_case(&identifier),
        "name": name,
        "runtime_version": env!("CARGO_PKG_VERSION")
    });

    template_files(
        "game",
        &template_key,
        project_root_path.as_path(),
        paths,
        &globals,
    )?;

    copy_wits(project_root_path.as_path())?;

    Ok(())
}

pub fn template_files(
    template_type: &str,
    template_key: &str,
    project_root: &Path,
    paths: Vec<Cow<'static, str>>,
    globals: &Object,
) -> crate::Result<()> {
    for path in paths {
        let contents = Templates::get(path.as_ref()).unwrap();
        let relative_path = path
            .strip_prefix(&format!("{template_type}/{template_key}/"))
            .unwrap();
        let destination_path = project_root.join(PathBuf::from(relative_path));

        let destination_parent = destination_path.as_path().parent().unwrap();
        std::fs::create_dir_all(destination_parent).unwrap();
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&destination_path.clone())?;

        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(&std::str::from_utf8(&contents.data).unwrap())
            .unwrap();

        let contents = template.render(&globals).unwrap();

        file.write_all(contents.as_bytes())?;
    }

    Ok(())
}

pub fn copy_wits(
    project_root: &Path,
) -> crate::Result<()> {
    for wit_path in JumpjetWits::iter() {
        let contents = JumpjetWits::get(wit_path.as_ref()).unwrap();
        let destination_path = project_root
            .join(".jumpjet/wit")
            .join(PathBuf::from(wit_path.as_ref()));

        let destination_parent = destination_path.as_path().parent().unwrap();
        std::fs::create_dir_all(destination_parent).unwrap();
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&destination_path.clone())?;

        file.write_all(&contents.data)?;
    }

    Ok(())
}

pub fn to_snake_case(input: &str) -> String {
    let mut snake_case = String::new();
    
    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            // Add an underscore before uppercase letters (except for the first character)
            snake_case.push('_');
        }

        // Convert the character to lowercase
        snake_case.push(c.to_ascii_lowercase());
    }

    // Replace spaces with underscores
    snake_case.replace(" ", "_").replace("-", "_")
}
