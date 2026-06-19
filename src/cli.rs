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
    /// View Jumpjet API documentation
    Docs,
    /// Create a new Jumpjet project, specifying a template
    #[command(subcommand)]
    New(NewSubcommand),
    // /// Pushes a new release candidate for an unreleased version. Will NOT publish
    // Push,
    /// Just for testin'
    Test,
    // Temporarily disabled — not exposed publicly for now
    // /// Re-resolve dependencies and refresh jumpjet.lock
    // Update,
    // /// Add a package dependency to jumpjet.toml and resolve it
    // Add {
    //     /// Package spec: `namespace:name[@version]`
    //     spec: String,
    //     /// Use a local path dependency
    //     #[clap(long, value_name = "DIR")]
    //     path: Option<String>,
    //     /// Use a git dependency
    //     #[clap(long, value_name = "URL")]
    //     git: Option<String>,
    //     /// Use an http(s) bundle dependency
    //     #[clap(long, value_name = "URL")]
    //     url: Option<String>,
    //     /// Git tag (with --git)
    //     #[clap(long)]
    //     tag: Option<String>,
    //     /// Git branch (with --git)
    //     #[clap(long)]
    //     branch: Option<String>,
    //     /// Git revision (with --git)
    //     #[clap(long)]
    //     rev: Option<String>,
    // },
    // /// Publish this package (type = "lib") to a registry via `wkg`
    // Publish,
    /// Build and run the project
    Run {
        #[clap(long, default_value_t = false)]
        release: bool,
        /// Run target: "native" (default) or "web" (serve + open browser)
        #[clap(long, value_name = "TARGET")]
        target: Option<String>,
        /// Port for the web dev server (only with `--target web`)
        #[clap(long, value_name = "PORT")]
        port: Option<u16>,
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
    // /// Authorizes the Jumpjet CLI with the provided account token (useful for CI)
    // Auth {
    //     #[clap(long, short = 't', value_name = "TOKEN")]
    //     token: Option<String>,
    // },
    // /// Deathorizes the Jumpjet CLI
    // Deauth,
    // /// Publishes the specified release version, making it publicly available
    // Publish {
    //     #[clap(long, short = 'v', value_name = "VERSION")]
    //     version: Option<String>,
    // },
    /// Upgrade the Jumpjet CLI to the latest version
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
    /// New package (library) that other games or packages can depend on
    Package {
        /// Package name in `namespace:name` form (e.g. `acme:physics`)
        #[clap(long, short = 'n', value_name = "NAME")]
        name: String,
        /// One of: lib-rust
        #[clap(long, short = 't', value_name = "TEMPLATE")]
        template: String,
    },
}
