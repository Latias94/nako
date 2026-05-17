use std::{collections::BTreeMap, env};

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use taru_client::{
    ClientOutputContainer, ClientRequest, ClientTransport, PageQuery, PlaybackCapabilitiesQuery,
    RemuxPlaybackQuery, ReqwestTransport, SearchQuery, TaruClient, TaruClientError,
};
use thiserror::Error;

#[derive(Clone, Debug, Parser)]
#[command(name = "taru-client-cli")]
#[command(about = "Taru public client CLI")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:3000")]
    pub base_url: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub token_env: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Health,
    Libraries {
        #[command(flatten)]
        page: PageArgs,
    },
    Items {
        #[command(flatten)]
        page: PageArgs,
    },
    Search {
        q: Option<String>,
        #[arg(long)]
        facet: Option<String>,
        #[command(flatten)]
        page: PageArgs,
    },
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },
    Playback {
        #[command(subcommand)]
        command: PlaybackCommand,
    },
    Stream {
        #[command(subcommand)]
        command: StreamCommand,
    },
}

#[derive(Clone, Copy, Debug, Default, Args)]
pub struct PageArgs {
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub offset: Option<u64>,
}

impl PageArgs {
    #[must_use]
    pub const fn into_query(self) -> Option<PageQuery> {
        if self.limit.is_none() && self.offset.is_none() {
            None
        } else {
            Some(PageQuery::new(self.limit, self.offset))
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum SourceCommand {
    Probe { source_id: String },
}

#[derive(Clone, Debug, Subcommand)]
pub enum PlaybackCommand {
    Decision {
        source_id: String,
        #[command(flatten)]
        capabilities: PlaybackCapabilityArgs,
    },
    Session {
        session_id: String,
    },
    Cancel {
        session_id: String,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum StreamCommand {
    Direct {
        source_id: String,
        #[arg(long)]
        range: Option<String>,
    },
    Head {
        source_id: String,
        #[arg(long)]
        range: Option<String>,
    },
    Remux {
        source_id: String,
        #[arg(long)]
        range: Option<String>,
        #[arg(long)]
        output_container: Option<RemuxOutputContainer>,
        #[command(flatten)]
        capabilities: PlaybackCapabilityArgs,
    },
    HlsPlaylist {
        source_id: String,
        #[command(flatten)]
        capabilities: PlaybackCapabilityArgs,
    },
    HlsSegment {
        session_id: String,
        segment_name: String,
    },
}

#[derive(Clone, Debug, Default, Args)]
pub struct PlaybackCapabilityArgs {
    #[arg(long)]
    pub direct_play: Option<bool>,
    #[arg(long)]
    pub container: Option<String>,
    #[arg(long)]
    pub video_codec: Option<String>,
    #[arg(long)]
    pub audio_codec: Option<String>,
}

impl PlaybackCapabilityArgs {
    #[must_use]
    pub fn as_query(&self) -> Option<PlaybackCapabilitiesQuery<'_>> {
        if self.direct_play.is_none()
            && self.container.is_none()
            && self.video_codec.is_none()
            && self.audio_codec.is_none()
        {
            None
        } else {
            Some(PlaybackCapabilitiesQuery {
                direct_play: self.direct_play,
                container: self.container.as_deref(),
                video_codec: self.video_codec.as_deref(),
                audio_codec: self.audio_codec.as_deref(),
            })
        }
    }

    #[must_use]
    pub fn as_query_or_default(&self) -> PlaybackCapabilitiesQuery<'_> {
        self.as_query().unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum RemuxOutputContainer {
    Mp4,
    Mkv,
}

impl From<RemuxOutputContainer> for ClientOutputContainer {
    fn from(value: RemuxOutputContainer) -> Self {
        match value {
            RemuxOutputContainer::Mp4 => Self::Mp4,
            RemuxOutputContainer::Mkv => Self::Mkv,
        }
    }
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("token environment variable {name} is not set")]
    MissingTokenEnv { name: String },
    #[error(transparent)]
    Client(#[from] TaruClientError),
    #[error("failed to serialize CLI output")]
    Serialize(#[from] serde_json::Error),
}

pub async fn run(cli: Cli) -> Result<String, CliError> {
    run_with_transport(cli, ReqwestTransport::default()).await
}

pub async fn run_with_transport(
    cli: Cli,
    transport: impl ClientTransport + 'static,
) -> Result<String, CliError> {
    let command = cli.command.clone();
    let client = build_client(&cli, transport)?;
    execute(client, command).await
}

fn build_client(
    cli: &Cli,
    transport: impl ClientTransport + 'static,
) -> Result<TaruClient, CliError> {
    let mut client = TaruClient::with_transport(&cli.base_url, transport)?;
    if let Some(token) = resolve_token(cli)? {
        client = client.bearer_token(token);
    }
    Ok(client)
}

fn resolve_token(cli: &Cli) -> Result<Option<String>, CliError> {
    if let Some(token) = &cli.token {
        return Ok(Some(token.clone()));
    }
    if let Some(name) = &cli.token_env {
        return env::var(name)
            .map(Some)
            .map_err(|_| CliError::MissingTokenEnv { name: name.clone() });
    }
    Ok(None)
}

async fn execute(client: TaruClient, command: Command) -> Result<String, CliError> {
    let value = match command {
        Command::Health => serde_json::to_value(client.health().await?)?,
        Command::Libraries { page } => {
            serde_json::to_value(client.list_libraries(page.into_query()).await?)?
        }
        Command::Items { page } => {
            serde_json::to_value(client.list_items(page.into_query()).await?)?
        }
        Command::Search { q, facet, page } => {
            let query = SearchQuery {
                q: q.as_deref(),
                facet: facet.as_deref(),
                page: page.into_query(),
            };
            serde_json::to_value(client.search_items(query).await?)?
        }
        Command::Source { command } => match command {
            SourceCommand::Probe { source_id } => {
                serde_json::to_value(client.get_source_probe(source_id).await?)?
            }
        },
        Command::Playback { command } => match command {
            PlaybackCommand::Decision {
                source_id,
                capabilities,
            } => serde_json::to_value(
                client
                    .get_playback_decision(source_id, capabilities.as_query())
                    .await?,
            )?,
            PlaybackCommand::Session { session_id } => {
                serde_json::to_value(client.get_playback_session(session_id).await?)?
            }
            PlaybackCommand::Cancel { session_id } => {
                serde_json::to_value(client.cancel_playback_session(session_id).await?)?
            }
        },
        Command::Stream { command } => streaming_command_output(&client, command)?,
    };
    serde_json::to_string_pretty(&value).map_err(CliError::Serialize)
}

fn streaming_command_output(
    client: &TaruClient,
    command: StreamCommand,
) -> Result<serde_json::Value, CliError> {
    let request = match command {
        StreamCommand::Direct { source_id, range } => {
            client.stream_source_request(source_id, range.as_deref())?
        }
        StreamCommand::Head { source_id, range } => {
            client.head_stream_source_request(source_id, range.as_deref())?
        }
        StreamCommand::Remux {
            source_id,
            range,
            output_container,
            capabilities,
        } => {
            let query = RemuxPlaybackQuery {
                capabilities: capabilities.as_query_or_default(),
                output_container: output_container.map(Into::into),
            };
            client.remux_stream_source_request(source_id, Some(query), range.as_deref())?
        }
        StreamCommand::HlsPlaylist {
            source_id,
            capabilities,
        } => client.hls_playlist_request(source_id, capabilities.as_query())?,
        StreamCommand::HlsSegment {
            session_id,
            segment_name,
        } => client.hls_segment_request(session_id, segment_name)?,
    };
    serde_json::to_value(SafeRequestOutput::from_request(&request)).map_err(CliError::Serialize)
}

#[derive(Debug, Serialize)]
struct SafeRequestOutput {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
}

impl SafeRequestOutput {
    fn from_request(request: &ClientRequest) -> Self {
        let mut headers = BTreeMap::new();
        for (name, value) in &request.headers {
            let name = name.as_str().to_ascii_lowercase();
            let value = if name.eq_ignore_ascii_case("authorization") {
                "<redacted>".to_owned()
            } else {
                value.to_str().unwrap_or("<non-utf8>").to_owned()
            };
            headers.insert(name, value);
        }

        Self {
            method: request.method.as_str().to_owned(),
            url: request.url.as_str().to_owned(),
            headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use clap::Parser;
    use reqwest::{
        Method, StatusCode,
        header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue},
    };
    use serde_json::{Value, json};
    use taru_client::{
        API_VERSION, API_VERSION_HEADER, ClientRequest, ClientResponse, ClientTransport,
        TaruClientError,
    };

    use super::*;

    #[derive(Clone, Default)]
    struct MockTransport {
        requests: Arc<Mutex<Vec<ClientRequest>>>,
        responses: Arc<Mutex<VecDeque<ClientResponse>>>,
    }

    impl MockTransport {
        fn push_json(&self, status: StatusCode, body: Value) {
            let mut headers = HeaderMap::new();
            headers.insert(
                HeaderName::from_static(API_VERSION_HEADER),
                HeaderValue::from_static(API_VERSION),
            );
            self.responses.lock().unwrap().push_back(ClientResponse {
                status,
                headers,
                body: serde_json::to_vec(&body).unwrap(),
            });
        }

        fn requests(&self) -> Vec<ClientRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ClientTransport for MockTransport {
        async fn send(&self, request: ClientRequest) -> Result<ClientResponse, TaruClientError> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| panic!("missing mock response"))
        }
    }

    #[tokio::test]
    async fn health_command_uses_sdk_transport_without_auth_header() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "status": "ok",
                "version": "v1"
            }),
        );
        let cli = Cli::parse_from([
            "taru-client-cli",
            "--base-url",
            "http://localhost:3000/api",
            "--token",
            "secret-token",
            "health",
        ]);

        let output = run_with_transport(cli, transport.clone()).await.unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(value["status"], "ok");
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(requests[0].url.as_str(), "http://localhost:3000/api/health");
        assert!(requests[0].headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn search_command_serializes_query_and_pagination_through_sdk() {
        let transport = MockTransport::default();
        transport.push_json(
            StatusCode::OK,
            json!({
                "hits": [],
                "page": {
                    "limit": 25,
                    "offset": 50,
                    "returned": 0
                }
            }),
        );
        let cli = Cli::parse_from([
            "taru-client-cli",
            "--base-url",
            "http://localhost:3000",
            "--token",
            "secret-token",
            "search",
            "matrix",
            "--facet",
            "genre:sci-fi",
            "--limit",
            "25",
            "--offset",
            "50",
        ]);

        let output = run_with_transport(cli, transport.clone()).await.unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(value["page"]["limit"], 25);
        let requests = transport.requests();
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(
            requests[0].url.as_str(),
            "http://localhost:3000/search?q=matrix&facet=genre%3Asci-fi&limit=25&offset=50"
        );
        assert_eq!(
            requests[0].headers.get(AUTHORIZATION).unwrap(),
            HeaderValue::from_static("Bearer secret-token")
        );
    }

    #[tokio::test]
    async fn remux_stream_command_prints_safe_request_facts() {
        let cli = Cli::parse_from([
            "taru-client-cli",
            "--base-url",
            "http://localhost:3000/api",
            "--token",
            "secret-token",
            "stream",
            "remux",
            "source 1",
            "--range",
            "bytes=0-",
            "--output-container",
            "mkv",
            "--direct-play",
            "false",
            "--container",
            "mp4,mkv",
            "--video-codec",
            "h264",
            "--audio-codec",
            "aac",
        ]);

        let output = run_with_transport(cli, MockTransport::default())
            .await
            .unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(value["method"], "GET");
        assert_eq!(
            value["url"],
            "http://localhost:3000/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv"
        );
        assert_eq!(value["headers"]["authorization"], "<redacted>");
        assert_eq!(value["headers"]["range"], "bytes=0-");
        assert!(!output.contains("secret-token"));
    }

    #[tokio::test]
    async fn hls_segment_command_encodes_path_without_transport_call() {
        let cli = Cli::parse_from([
            "taru-client-cli",
            "--base-url",
            "http://localhost:3000",
            "--token",
            "secret-token",
            "stream",
            "hls-segment",
            "session 1",
            "seg 001.ts",
        ]);
        let transport = MockTransport::default();

        let output = run_with_transport(cli, transport.clone()).await.unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(value["method"], "GET");
        assert_eq!(
            value["url"],
            "http://localhost:3000/playback/sessions/session%201/hls/segments/seg%20001.ts"
        );
        assert_eq!(value["headers"]["authorization"], "<redacted>");
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn manifest_keeps_client_cli_apache_and_outside_server_crates() {
        let manifest = include_str!("../Cargo.toml");

        assert!(manifest.contains("license = \"Apache-2.0\""));
        assert!(manifest.contains("taru-client ="));
        for forbidden in [
            "taru-api",
            "taru-server",
            "taru-core",
            "taru-streaming",
            "taru-transcode",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "client CLI manifest leaked forbidden dependency {forbidden}"
            );
        }
    }
}
