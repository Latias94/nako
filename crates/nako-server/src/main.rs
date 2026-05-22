use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use nako_core::{
    IngestionFailurePhase, IngestionFailureStatus, LibraryId, MediaItemId, NakoError, Result,
};
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod app;
mod config;
mod http;

use app::NakoApp;
use config::{
    ConfigPreflightOptions, NakoServerConfig, example_config, load_config, preflight_config,
    render_config_preflight_text,
};
use http::build_router;

#[derive(Debug, Parser)]
#[command(name = "nako-server")]
#[command(about = "Nako self-hosted media server foundation")]
struct Cli {
    #[arg(short, long, default_value = "nako.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print an example configuration.
    ConfigExample,
    /// Validate configuration without starting the server.
    ConfigCheck {
        /// Print the preflight report as JSON.
        #[arg(long)]
        json: bool,
        /// Create and write-probe Nako-owned runtime directories.
        #[arg(long)]
        create_dirs: bool,
    },
    /// Run the HTTP server.
    Serve,
    /// Scan one configured library and probe discovered media.
    Scan {
        #[arg(long)]
        library_id: Option<LibraryId>,
    },
    /// Scan every configured library and probe discovered media.
    ScanAll,
    /// List indexed media sources and probe results as JSON.
    List {
        #[arg(long)]
        library_id: Option<LibraryId>,
    },
    /// List scan/probe ingestion failures as JSON.
    IngestionFailures {
        #[arg(long)]
        library_id: Option<LibraryId>,
        #[arg(long)]
        phase: Option<IngestionFailurePhase>,
        #[arg(long)]
        status: Option<IngestionFailureStatus>,
        #[arg(long)]
        all: bool,
    },
    /// Refresh TMDB metadata for one indexed media item.
    RefreshMetadata { item_id: MediaItemId },
    /// Import NFO sidecar metadata for one configured library.
    ImportNfo { library_id: Option<LibraryId> },
    /// Export canonical metadata to NFO sidecars for one configured library.
    ExportNfo { library_id: Option<LibraryId> },
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "nako-server command failed");
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
        Command::ConfigCheck { json, create_dirs } => {
            let config = load_config(&cli.config)?;
            let report = preflight_config(&config, ConfigPreflightOptions { create_dirs });
            if json {
                print_json(&report)?;
            } else {
                print!("{}", render_config_preflight_text(&report));
            }
            if report.has_failures() {
                Err(NakoError::InvalidInput {
                    message: "configuration preflight failed".to_owned(),
                })
            } else {
                Ok(())
            }
        }
        Command::Serve => {
            let config = load_config(&cli.config)?;
            let listen_addr = config.listen_addr;
            let app = NakoApp::new(config).await?;
            serve(listen_addr, app).await
        }
        Command::Scan { library_id } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            let library_id = resolve_cli_library_id(app.config(), library_id, "scan")?;
            print_json(&app.library_scan().scan_library(library_id).await?)
        }
        Command::ScanAll => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            print_json(&app.library_scan().scan_all_configured_libraries().await?)
        }
        Command::List { library_id } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            let library_id = resolve_cli_library_id(app.config(), library_id, "list")?;
            print_json(
                &app.library()
                    .list_library_sources(library_id, nako_core::PageRequest::first_page())
                    .await?,
            )
        }
        Command::IngestionFailures {
            library_id,
            phase,
            status,
            all,
        } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            let library_id =
                resolve_cli_library_id(app.config(), library_id, "ingestion-failures")?;
            let status = if all {
                None
            } else {
                status.or(Some(IngestionFailureStatus::Open))
            };
            print_json(
                &app.library()
                    .list_ingestion_failures(
                        library_id,
                        phase,
                        status,
                        nako_core::PageRequest::first_page(),
                    )
                    .await?,
            )
        }
        Command::RefreshMetadata { item_id } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            print_json(&app.metadata().refresh_item_metadata(item_id).await?)
        }
        Command::ImportNfo { library_id } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            let library_id = resolve_cli_library_id(app.config(), library_id, "import-nfo")?;
            print_json(&app.nfo().import_library_nfo(library_id).await?)
        }
        Command::ExportNfo { library_id } => {
            let config = load_config(&cli.config)?;
            let app = NakoApp::new(config).await?;
            let library_id = resolve_cli_library_id(app.config(), library_id, "export-nfo")?;
            print_json(&app.nfo().export_library_nfo(library_id).await?)
        }
    }
}

fn resolve_cli_library_id(
    config: &NakoServerConfig,
    library_id: Option<LibraryId>,
    command: &str,
) -> Result<LibraryId> {
    if let Some(library_id) = library_id {
        return Ok(library_id);
    }

    match config.libraries.as_slice() {
        [] => Err(NakoError::InvalidInput {
            message: "server config must include at least one library".to_owned(),
        }),
        [library] => Ok(library.id),
        _ => {
            let scan_hint = if command == "scan" {
                "; use scan-all for a full scan"
            } else {
                ""
            };
            Err(NakoError::InvalidInput {
                message: format!(
                    "{command} requires --library-id when multiple libraries are configured{scan_hint}"
                ),
            })
        }
    }
}

async fn serve(listen_addr: std::net::SocketAddr, app: NakoApp) -> Result<()> {
    let listener = TcpListener::bind(listen_addr)
        .await
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to bind HTTP listener {listen_addr}: {err}"),
        })?;
    let local_addr = listener
        .local_addr()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to read HTTP listener address: {err}"),
        })?;

    info!(listen_addr = %local_addr, "nako HTTP server listening");

    let result = axum::serve(
        listener,
        build_router(app.clone()).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|err| NakoError::Provider {
        provider: "http_server".to_owned(),
        message: format!("HTTP server failed: {err}"),
    });
    app.shutdown_runtime();
    result
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("shutdown signal received");
        }
        Err(err) => {
            warn!(error = %err, "failed to listen for shutdown signal");
        }
    }
}

fn print_json(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize output: {err}"),
    })?;
    println!("{json}");
    Ok(())
}

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("nako_server=info"));

    let _ = fmt().with_env_filter(env_filter).try_init();
}
