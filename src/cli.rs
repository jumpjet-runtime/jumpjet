use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::utils::version;

#[derive(Parser)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// View Rune API documentation
    Docs,
    /// Create a new Rune project, specifying a template
    #[command(subcommand)]
    New(NewSubcommand),
    // /// Pushes a new release candidate for an unreleased version. Will NOT publish
    // Push,
    /// Just for testin'
    Test,
    /// Build and run the project
    Run {
        #[clap(long, default_value_t = false)]
        release: bool,
    },
    /// Build the project
    Build {
        #[clap(long, default_value_t = false)]
        release: bool,
        /// Build target: "native" (default) or "web"
        #[clap(long, value_name = "TARGET")]
        target: Option<String>,
    },
    /// Bundle the project for the target platform
    Bundle {
        #[clap(long, default_value_t = false)]
        release: bool,
        #[clap(long, value_name = "TARGET")]
        target: String,
    },
    // /// Authorizes the Rune CLI with the provided account token (useful for CI)
    // Auth {
    //     #[clap(long, short = 't', value_name = "TOKEN")]
    //     token: Option<String>,
    // },
    // /// Deathorizes the Rune CLI
    // Deauth,
    // /// Publishes the specified release version, making it publicly available
    // Publish {
    //     #[clap(long, short = 'v', value_name = "VERSION")]
    //     version: Option<String>,
    // },
    /// Upgrade the Rune CLI to the latest version
    Upgrade,
}

#[derive(Subcommand)]
pub enum NewSubcommand {
    // New game
    Game {
        #[clap(long, short = 'i', value_name = "IDENTIFIER")]
        identifier: Option<String>,
        #[clap(long, short = 'n', value_name = "NAME")]
        name: Option<String>,
        /// One of: hello-js, hello-rust, cube-rust
        #[clap(long, short = 't', value_name = "TEMPLATE")]
        template: String,
    },
}
