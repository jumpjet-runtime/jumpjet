use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;

use current_platform::CURRENT_PLATFORM;
use jumpjet::input;
use ratatui::prelude::Rect;
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use toml::Table;

use crate::{
    action::Action,
    cli::{Cli, CliCommand, NewSubcommand},
    components::Component,
    config::Config,
    mode::Mode,
    settings::Settings,
    tui,
};

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub command: Option<CliCommand>,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let config = Config::new()?;
        let mode = Mode::Home;
        let command = cli.command;

        Ok(Self {
            tick_rate: 4.0,
            frame_rate: 60.0,
            should_quit: false,
            should_suspend: false,
            config,
            command,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // On a brand-new install, report `install` once. Fired before the command
        // event so the shared first-run setup (consent notice, app_instance_id) is
        // persisted and reused below.
        let install = crate::analytics::track_install();

        // Report the invoked command (anonymous, no project data). Held until the
        // command finishes so the request can overlap execution, then flushed.
        let analytics = self
            .command
            .as_ref()
            .and_then(|c| crate::analytics::track("command_run", command_params(c)));

        let result = self.dispatch().await;

        crate::analytics::flush(install).await;
        crate::analytics::flush(analytics).await;

        result
    }

    async fn dispatch(&mut self) -> Result<()> {
        match &self.command {
            Some(CliCommand::New(new)) => crate::commands::new::new(new).await?,
            Some(CliCommand::Test) => {
                let input_path = Path::new("../test-game/dist/.jumpjet/input/");
                let binary = std::fs::read(input_path.join("test-game.wasm")).unwrap();
                jumpjet::runtime::test(input_path.to_path_buf(), binary).await;
            }
            Some(CliCommand::Run {
                release,
                target,
                port,
            }) => match target.as_deref() {
                Some("web") => crate::commands::run::run_web(release, port.unwrap_or(8731)).await?,
                Some(other) if other != "native" => {
                    return Err(color_eyre::eyre::eyre!("unknown run target: {other}"));
                }
                None | Some(_) => crate::commands::run::run(release).await?,
            },
            Some(CliCommand::Build { release, target }) => match target.as_deref() {
                Some("web") => crate::commands::build::build_web(release).await?,
                Some(other) if other != "native" => {
                    return Err(color_eyre::eyre::eyre!("unknown build target: {other}"));
                }
                None | Some(_) => crate::commands::build::build(release).await?,
            },
            Some(CliCommand::Bundle { target, release }) => match target.as_str() {
                "web" => {
                    crate::commands::build::build_web(release).await?;
                    crate::commands::bundle::web::bundle_project(release).await?;
                }
                _ => {
                    crate::commands::build::build(release).await?;
                    crate::commands::bundle::bundle(target, release).await?;
                }
            },
            // Temporarily disabled — not exposed publicly for now
            // Some(CliCommand::Update) => crate::commands::update::update().await?,
            // Some(CliCommand::Add {
            //     spec,
            //     path,
            //     git,
            //     url,
            //     tag,
            //     branch,
            //     rev,
            // }) => {
            //     crate::commands::add::add(
            //         spec,
            //         crate::commands::add::AddOptions {
            //             path: path.clone(),
            //             git: git.clone(),
            //             url: url.clone(),
            //             tag: tag.clone(),
            //             branch: branch.clone(),
            //             rev: rev.clone(),
            //         },
            //     )
            //     .await?
            // }
            // Some(CliCommand::Publish) => crate::commands::publish::publish().await?,
            Some(CliCommand::Wit) => crate::commands::wit::wit().await?,
            Some(CliCommand::Upgrade) => crate::commands::upgrade::upgrade().await?,
            Some(CliCommand::Docs) => crate::commands::docs::docs(&self.config, &self.mode).await?,
            None => {}
        }

        Ok(())
    }
}

/// Build the GA4 event params for an invoked command. Only the command name and
/// low-cardinality flags are recorded — never paths, names, identifiers, or any
/// other project content.
fn command_params(command: &CliCommand) -> serde_json::Value {
    use serde_json::json;

    let cli_version = env!("CARGO_PKG_VERSION");
    let (command, extra) = match command {
        CliCommand::New(NewSubcommand::Game { template, .. }) => {
            ("new_game", json!({ "template": template }))
        }
        CliCommand::New(NewSubcommand::Package { template, .. }) => {
            ("new_package", json!({ "template": template }))
        }
        CliCommand::Run {
            release, target, ..
        } => (
            "run",
            json!({ "release": release, "target": target.as_deref().unwrap_or("native") }),
        ),
        CliCommand::Build { release, target } => (
            "build",
            json!({ "release": release, "target": target.as_deref().unwrap_or("native") }),
        ),
        CliCommand::Bundle { release, target } => {
            ("bundle", json!({ "release": release, "target": target }))
        }
        CliCommand::Wit => ("wit", json!({})),
        CliCommand::Upgrade => ("upgrade", json!({})),
        CliCommand::Docs => ("docs", json!({})),
        CliCommand::Test => ("test", json!({})),
    };

    let mut params = json!({ "command": command, "cli_version": cli_version });
    if let (serde_json::Value::Object(params), serde_json::Value::Object(extra)) =
        (&mut params, extra)
    {
        params.extend(extra);
    }
    params
}
