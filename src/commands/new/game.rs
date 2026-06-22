use rust_embed::Embed;
use std::{
    borrow::Cow,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{Result, assets::Templates};

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

    // Stage the Jumpjet runtime WIT as a dependency of the game's own world, then
    // generate that world (`jumpjet:game`'s `game` world, which includes the
    // runtime). The guest targets `world: "game"` at `.jumpjet/wit`; `jumpjet add`
    // later regenerates the same world with extra imports as packages are added.
    let wit_root = project_root_path.join(".jumpjet").join("wit");
    super::package::copy_runtime_wit(&wit_root.join("deps").join("runtime"))?;
    // Multiplayer templates ship a `[server.build]`; detect it from the just-written
    // manifest so the generated WIT includes the `server` world too.
    let multiplayer = crate::pkg::manifest::Manifest::load_from(&project_root_path)
        .map(|m| m.server_build().is_some())
        .unwrap_or(false);
    crate::pkg::stage::write_generated_worlds(&wit_root, &[], multiplayer)?;

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
