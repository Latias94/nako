use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    sync::{Arc, Mutex},
};

use nako_core::{AuthenticatedPrincipal, MediaSourceId, NakoError, Result};
use sha2::{Digest, Sha256};

const TICKET_TTL_MS: i64 = 6 * 60 * 60 * 1_000;
const TOKEN_PREFIX: &str = "nako_bpt_";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BrowserPlaybackTicketMode {
    Direct,
    Remux,
    Hls,
    Subtitle,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct IssuedBrowserPlaybackTicket {
    pub(crate) token: String,
    pub(crate) expires_at_ms: i64,
}

impl fmt::Debug for IssuedBrowserPlaybackTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedBrowserPlaybackTicket")
            .field("token", &"<redacted>")
            .field("expires_at_ms", &self.expires_at_ms)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BrowserPlaybackTicketService {
    store: Arc<Mutex<BrowserPlaybackTicketStore>>,
}

impl BrowserPlaybackTicketService {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BrowserPlaybackTicketStore::default())),
        }
    }

    pub(crate) fn issue_source_ticket(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
        mode: BrowserPlaybackTicketMode,
        now_ms: i64,
    ) -> Result<IssuedBrowserPlaybackTicket> {
        self.issue_scoped_source_ticket(principal, source_id, mode, None, now_ms)
    }

    pub(crate) fn issue_subtitle_ticket(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
        stream_index: u32,
        now_ms: i64,
    ) -> Result<IssuedBrowserPlaybackTicket> {
        self.issue_scoped_source_ticket(
            principal,
            source_id,
            BrowserPlaybackTicketMode::Subtitle,
            Some(stream_index),
            now_ms,
        )
    }

    fn issue_scoped_source_ticket(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
        mode: BrowserPlaybackTicketMode,
        stream_index: Option<u32>,
        now_ms: i64,
    ) -> Result<IssuedBrowserPlaybackTicket> {
        let expires_at_ms =
            now_ms
                .checked_add(TICKET_TTL_MS)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "browser playback ticket expiry overflowed".to_owned(),
                })?;
        let mut store = self
            .store
            .lock()
            .expect("browser playback ticket store mutex poisoned");
        store.cleanup_expired(now_ms);

        for _ in 0..8 {
            let token = generate_ticket_token();
            let token_hash = hash_ticket_token(&token);
            match store.tickets_by_hash.entry(token_hash) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(entry) => {
                    entry.insert(BrowserPlaybackTicketRecord {
                        source_id,
                        mode,
                        stream_index,
                        principal: principal.clone(),
                        expires_at_ms,
                    });
                    return Ok(IssuedBrowserPlaybackTicket {
                        token,
                        expires_at_ms,
                    });
                }
            }
        }

        Err(NakoError::Conflict {
            message: "could not allocate a unique browser playback ticket".to_owned(),
        })
    }

    pub(crate) fn validate_source_ticket(
        &self,
        token: &str,
        source_id: MediaSourceId,
        mode: BrowserPlaybackTicketMode,
        now_ms: i64,
    ) -> Result<AuthenticatedPrincipal> {
        self.validate_scoped_source_ticket(token, source_id, mode, None, now_ms)
    }

    pub(crate) fn validate_subtitle_ticket(
        &self,
        token: &str,
        source_id: MediaSourceId,
        stream_index: u32,
        now_ms: i64,
    ) -> Result<AuthenticatedPrincipal> {
        self.validate_scoped_source_ticket(
            token,
            source_id,
            BrowserPlaybackTicketMode::Subtitle,
            Some(stream_index),
            now_ms,
        )
    }

    fn validate_scoped_source_ticket(
        &self,
        token: &str,
        source_id: MediaSourceId,
        mode: BrowserPlaybackTicketMode,
        stream_index: Option<u32>,
        now_ms: i64,
    ) -> Result<AuthenticatedPrincipal> {
        if token.trim().is_empty() {
            return Err(invalid_playback_ticket());
        }

        let token_hash = hash_ticket_token(token);
        let mut store = self
            .store
            .lock()
            .expect("browser playback ticket store mutex poisoned");
        let Some(record) = store.tickets_by_hash.get(&token_hash) else {
            return Err(invalid_playback_ticket());
        };

        if record.expires_at_ms <= now_ms {
            store.tickets_by_hash.remove(&token_hash);
            return Err(invalid_playback_ticket());
        }

        if record.source_id != source_id
            || record.mode != mode
            || record.stream_index != stream_index
        {
            return Err(invalid_playback_ticket());
        }

        Ok(record.principal.clone())
    }
}

#[derive(Debug, Default)]
struct BrowserPlaybackTicketStore {
    tickets_by_hash: HashMap<String, BrowserPlaybackTicketRecord>,
}

impl BrowserPlaybackTicketStore {
    fn cleanup_expired(&mut self, now_ms: i64) {
        self.tickets_by_hash
            .retain(|_, record| record.expires_at_ms > now_ms);
    }
}

#[derive(Clone, Debug)]
struct BrowserPlaybackTicketRecord {
    source_id: MediaSourceId,
    mode: BrowserPlaybackTicketMode,
    stream_index: Option<u32>,
    principal: AuthenticatedPrincipal,
    expires_at_ms: i64,
}

fn generate_ticket_token() -> String {
    format!(
        "{TOKEN_PREFIX}{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn hash_ticket_token(token: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

fn invalid_playback_ticket() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid browser playback ticket".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ticket_is_opaque_and_validates_scope_and_expiry() {
        let service = BrowserPlaybackTicketService::new();
        let principal = AuthenticatedPrincipal::bootstrap_admin();
        let source_id = MediaSourceId::new();
        let other_source_id = MediaSourceId::new();
        let issued = service
            .issue_source_ticket(
                &principal,
                source_id,
                BrowserPlaybackTicketMode::Direct,
                100,
            )
            .unwrap();

        assert!(issued.token.starts_with(TOKEN_PREFIX));
        assert!(!format!("{issued:?}").contains(&issued.token));
        assert!(!issued.token.contains(&source_id.to_string()));
        assert!(!issued.token.contains(&principal.principal_id.to_string()));
        assert_eq!(issued.expires_at_ms, 100 + TICKET_TTL_MS);

        let validated = service
            .validate_source_ticket(
                &issued.token,
                source_id,
                BrowserPlaybackTicketMode::Direct,
                101,
            )
            .unwrap();
        assert_eq!(validated, principal);

        assert!(
            service
                .validate_source_ticket(
                    &issued.token,
                    other_source_id,
                    BrowserPlaybackTicketMode::Direct,
                    101
                )
                .is_err()
        );
        assert!(
            service
                .validate_source_ticket(
                    &issued.token,
                    source_id,
                    BrowserPlaybackTicketMode::Hls,
                    101
                )
                .is_err()
        );
        assert!(
            service
                .validate_subtitle_ticket(&issued.token, source_id, 2, 101)
                .is_err()
        );
        assert!(
            service
                .validate_source_ticket(
                    &issued.token,
                    source_id,
                    BrowserPlaybackTicketMode::Direct,
                    issued.expires_at_ms
                )
                .is_err()
        );
        assert!(
            service
                .validate_source_ticket(
                    "not-a-ticket",
                    source_id,
                    BrowserPlaybackTicketMode::Direct,
                    101
                )
                .is_err()
        );
    }

    #[test]
    fn subtitle_ticket_is_scoped_to_stream_index() {
        let service = BrowserPlaybackTicketService::new();
        let principal = AuthenticatedPrincipal::bootstrap_admin();
        let source_id = MediaSourceId::new();
        let issued = service
            .issue_subtitle_ticket(&principal, source_id, 2, 100)
            .unwrap();

        assert_eq!(
            service
                .validate_subtitle_ticket(&issued.token, source_id, 2, 101)
                .unwrap(),
            principal
        );
        assert!(
            service
                .validate_subtitle_ticket(&issued.token, source_id, 3, 101)
                .is_err()
        );
        assert!(
            service
                .validate_source_ticket(
                    &issued.token,
                    source_id,
                    BrowserPlaybackTicketMode::Direct,
                    101
                )
                .is_err()
        );
    }
}
