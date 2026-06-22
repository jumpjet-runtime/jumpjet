#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod analytics;
pub mod app;
pub mod assets;
pub mod cli;
pub mod commands;
pub mod components;
pub mod config;
pub mod fs;
pub mod mode;
pub mod pkg;
pub mod settings;
pub mod tui;
pub mod utils;

use clap::Parser;
use cli::{Cli, CliCommand};
use color_eyre::eyre::Result;

use crate::{
    app::App,
    utils::{initialize_logging, initialize_panic_handler, version},
};

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let args = Cli::parse();

    let mut app = App::new(args)?;
    app.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
