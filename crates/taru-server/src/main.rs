use std::{path::PathBuf, process::ExitCode};

use axum::Router;
use clap::{Parser, Subcommand};
use serde::Serialize;
use taru_core::{Result, TaruError};
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

mod app;
mod config;
mod http;

use app::TaruApp;
use config::{example_config, load_config};
use http::build_router;

#[derive(Debug, Parser)]
#[command(name = "taru-server")]
#[command(about = "Taru self-hosted media server foundation")]
struct Cli {
    #[arg(short, long, default_value = "taru.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print an example configuration.
    ConfigExample,
    /// Run the HTTP server.
    Serve,
    /// Scan the configured local library and probe discovered media.
    Scan,
    /// List indexed media sources and probe results as JSON.
    List,
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "taru-server command failed");
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command.unwrap_or(Command::Serve) {
        Command::ConfigExample => {
            println!("{}", example_config()?);
            Ok(())
        }
        Command::Serve => {
            let config = load_config(&cli.config)?;
            let listen_addr = config.listen_addr;
            let app = TaruApp::new(config).await?;
            serve(listen_addr, build_router(app)).await
        }
        Command::Scan => {
            let config = load_config(&cli.config)?;
            let app = TaruApp::new(config).await?;
            print_json(&app.scan_configured_library().await?)
        }
        Command::List => {
            let config = load_config(&cli.config)?;
            let app = TaruApp::new(config).await?;
            print_json(&app.list_library_sources(app.config().library.id).await?)
        }
    }
}

async fn serve(listen_addr: std::net::SocketAddr, router: Router) -> Result<()> {
    let listener = TcpListener::bind(listen_addr)
        .await
        .map_err(|err| TaruError::InvalidInput {
            message: format!("failed to bind HTTP listener {listen_addr}: {err}"),
        })?;
    let local_addr = listener
        .local_addr()
        .map_err(|err| TaruError::InvalidInput {
            message: format!("failed to read HTTP listener address: {err}"),
        })?;

    info!(listen_addr = %local_addr, "taru HTTP server listening");

    axum::serve(listener, router)
        .await
        .map_err(|err| TaruError::Provider {
            provider: "http_server".to_owned(),
            message: format!("HTTP server failed: {err}"),
        })
}

fn print_json(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to serialize output: {err}"),
    })?;
    println!("{json}");
    Ok(())
}

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("taru_server=info"));

    let _ = fmt().with_env_filter(env_filter).try_init();
}
