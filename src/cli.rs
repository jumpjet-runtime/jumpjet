use std::path::PathBuf;

use clap::builder::PossibleValuesParser;
use clap::{Parser, Subcommand};

use crate::utils::version;

/// Block-letter "JUMPJET" wordmark shown atop `--help`, colored orange.
const LOGO: &str = concat!(
    "\n\x1b[38;5;202m",
    "  ▅▅▅▅ ▅▅   ▅▅ ▅▅   ▅▅ ▅▅▅▅▅▅▅  ▅▅▅▅ ▅▅▅▅▅▅ ▅▅▅▅▅▅\n",
    "    ▅▅ ▅▅   ▅▅ ▅▅▅▅▅▅▅ ▅▅▅▅▅▅▅    ▅▅ ▅▅▅▅▅▅   ▅▅  \n",
    "▅▅▅▅▅▅ ▅▅▅▅▅▅▅ ▅▅   ▅▅ ▅▅     ▅▅▅▅▅▅ ▅▅▅▅▅▅   ▅▅  ",
    "\x1b[0m\n",
);

/// Distinct template names embedded under `src/templates/<category>/` (e.g.
/// `game` -> `hello-rust`, `cube-python`, ...). Drives both `--help` listing and
/// validation, so it always matches the templates actually shipped in the binary.
fn template_names(category: &str) -> Vec<String> {
    let prefix = format!("{category}/");
    let mut names: Vec<String> = crate::assets::Templates::iter()
        .filter_map(|p| {
            p.strip_prefix(&prefix)
                .and_then(|rest| rest.split('/').next())
                .map(|name| name.to_string())
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

fn game_templates() -> PossibleValuesParser {
    PossibleValuesParser::new(template_names("game"))
}

fn package_templates() -> PossibleValuesParser {
    PossibleValuesParser::new(template_names("package"))
}

#[derive(Parser)]
#[command(author, version = version(), about, before_help = LOGO)]
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
        /// Run the headless server component (`[server.build]`) instead of the
        /// game/client. Native only.
        #[clap(long, default_value_t = false)]
        server: bool,
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
    /// Re-sync the project's staged WIT to this CLI's embedded runtime definitions
    Wit,
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
        /// Game template to scaffold from
        #[clap(long, short = 't', value_name = "TEMPLATE", value_parser = game_templates())]
        template: String,
    },
    /// New package (library) that other games or packages can depend on
    Package {
        /// Package name in `namespace:name` form (e.g. `acme:physics`)
        #[clap(long, short = 'n', value_name = "NAME")]
        name: String,
        /// Package template to scaffold from
        #[clap(long, short = 't', value_name = "TEMPLATE", value_parser = package_templates())]
        template: String,
    },
}
