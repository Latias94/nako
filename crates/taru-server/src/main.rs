use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use taru_core::{
    JobId, Library, LibraryId, LibraryRepository, MediaItem, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, Result, TaruError, TransactionManager,
};
use taru_db::SqliteStore;
use taru_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryProbeOptions, LibraryProbeRequest,
    LibraryProbeService,
};
use taru_media_probe::FfprobeMediaProbe;
use taru_vfs::LocalFsBackend;

#[derive(Debug, Parser)]
#[command(name = "taru-server")]
#[command(about = "Taru self-hosted media server foundation")]
struct Cli {
    #[arg(short, long, default_value = "taru.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print an example configuration.
    ConfigExample,
    /// Scan the configured local library and probe discovered media.
    Scan,
    /// List indexed media sources and probe results as JSON.
    List,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TaruServerConfig {
    database_url: String,
    #[serde(default = "default_ffprobe_path")]
    ffprobe_path: PathBuf,
    #[serde(default = "default_probe_concurrency")]
    probe_concurrency: usize,
    library: LocalLibraryConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct LocalLibraryConfig {
    id: LibraryId,
    name: String,
    root: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
struct ScanCommandOutput {
    index: taru_library::LibraryIndexSummary,
    probe: taru_library::LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
struct LibraryListOutput {
    library: Library,
    sources: Vec<LibrarySourceOutput>,
}

#[derive(Clone, Debug, Serialize)]
struct LibrarySourceOutput {
    source: MediaSource,
    item: Option<MediaItem>,
    probe: Option<MediaProbeResult>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::ConfigExample => {
            println!("{}", example_config()?);
            Ok(())
        }
        Command::Scan => {
            let config = load_config(&cli.config)?;
            let output = scan_configured_library(&config).await?;
            print_json(&output)
        }
        Command::List => {
            let config = load_config(&cli.config)?;
            let output = list_configured_library(&config).await?;
            print_json(&output)
        }
    }
}

async fn scan_configured_library(config: &TaruServerConfig) -> Result<ScanCommandOutput> {
    let store = connect_store(config).await?;
    let library = library_from_config(config);

    let index_backend = LocalFsBackend::new(&config.library.root)?;
    let scanner = taru_library::VfsLibraryScanner::new(index_backend);
    let index_service = LibraryIndexService::new(scanner, store.clone());
    let index = index_service
        .index_library(LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        })
        .await?;

    let probe_backend = LocalFsBackend::new(&config.library.root)?;
    let probe = FfprobeMediaProbe::new(&config.ffprobe_path);
    let probe_service = LibraryProbeService::with_options(
        probe_backend,
        probe,
        store,
        LibraryProbeOptions {
            max_concurrent_probes: config.probe_concurrency.max(1),
        },
    );
    let probe = probe_service
        .probe_library(LibraryProbeRequest {
            job_id: JobId::new(),
            library_id: library.id,
            force: false,
        })
        .await?;

    Ok(ScanCommandOutput { index, probe })
}

async fn list_configured_library(config: &TaruServerConfig) -> Result<LibraryListOutput> {
    let store = connect_store(config).await?;
    let library = store
        .get_library(config.library.id)
        .await?
        .unwrap_or_else(|| library_from_config(config));
    let sources = store.list_media_sources(library.id).await?;
    let mut output_sources = Vec::with_capacity(sources.len());

    for source in sources {
        let item = store.get_media_item(source.item_id).await?;
        let probe = store.get_media_probe(source.id).await?;
        output_sources.push(LibrarySourceOutput {
            source,
            item,
            probe,
        });
    }

    Ok(LibraryListOutput {
        library,
        sources: output_sources,
    })
}

async fn connect_store(config: &TaruServerConfig) -> Result<SqliteStore> {
    let store = SqliteStore::connect(&config.database_url).await?;
    store.migrate().await?;
    Ok(store)
}

fn load_config(path: &PathBuf) -> Result<TaruServerConfig> {
    let content = fs::read_to_string(path).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to read config {}: {err}", path.display()),
    })?;

    toml::from_str(&content).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to parse config {}: {err}", path.display()),
    })
}

fn example_config() -> Result<String> {
    let config = TaruServerConfig {
        database_url: "sqlite://taru.db".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        probe_concurrency: default_probe_concurrency(),
        library: LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: PathBuf::from("F:/Media/Movies"),
        },
    };

    toml::to_string_pretty(&config).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to render example config: {err}"),
    })
}

fn library_from_config(config: &TaruServerConfig) -> Library {
    Library {
        id: config.library.id,
        name: config.library.name.clone(),
        roots: vec!["local:///".to_owned()],
    }
}

fn print_json(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to serialize output: {err}"),
    })?;
    println!("{json}");
    Ok(())
}

fn default_ffprobe_path() -> PathBuf {
    PathBuf::from("ffprobe")
}

const fn default_probe_concurrency() -> usize {
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_from_toml() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"
            ffprobe_path = "ffprobe"
            probe_concurrency = 3

            [library]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(config.database_url, "sqlite://taru.db");
        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.probe_concurrency, 3);
        assert_eq!(config.library.name, "Movies");
        assert_eq!(config.library.root, PathBuf::from("F:/Media/Movies"));
        assert_eq!(library_from_config(&config).roots, vec!["local:///"]);
    }

    #[test]
    fn config_uses_default_probe_settings() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [library]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.probe_concurrency, 2);
    }
}
