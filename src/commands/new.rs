use crate::cli::NewSubcommand;

use crate::Result;

pub mod game;
pub mod package;

pub async fn new(new: &NewSubcommand) -> Result<()> {
    match new {
        crate::cli::NewSubcommand::Game {
            identifier,
            name,
            template,
        } => game::game(identifier, name, template).await?,
        crate::cli::NewSubcommand::Package { name, template } => {
            package::package(name, template).await?
        }
    }

    Ok(())
}
