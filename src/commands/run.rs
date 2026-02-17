use std::env;

use toml::Table;

use crate::cli::NewSubcommand;

use crate::Result;


pub async fn run(release: &bool) -> Result<()> {
    crate::commands::build::build(release).await?;

    let current_dir = env::current_dir()?;
    let config = std::fs::read_to_string(current_dir.join("rune.toml"))
        .unwrap()
        .parse::<Table>()
        .unwrap();

    let entrypoint_path = match config["build"]["entrypoint"].as_str() {
        Some(entrypoint_path) => entrypoint_path,
        None => panic!("No build input provided in config!"),
    };

    match config["build"]["output"].as_str() {
        Some(output_path) => {
            let output_path = current_dir.join(output_path);
            let entrypoint_path = output_path.join(entrypoint_path);
            let binary = std::fs::read(entrypoint_path).unwrap();
            rune::runtime::run(output_path.to_path_buf(), binary, !release);
        }
        None => panic!("No build input provided in config!"),
    }

    Ok(())
}
