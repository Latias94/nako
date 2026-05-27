use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    sync::{Arc, Mutex},
};

use nako_core::{
    AuthenticatedPrincipal, MediaSourceId, NakoError, PlaybackSessionId, PlaybackSessionMode,
    PlaybackTargetNetworkScope, RendererSessionId, Result,
};
use sha2::{Digest, Sha256};

const TICKET_TTL_MS: i64 = 60 * 60 * 1_000;
const TOKEN_PREFIX: &str = "nako_rtt_";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RendererTransportTicketScope {
    pub(crate) renderer_session_id: RendererSessionId,
    pub(crate) playback_session_id: PlaybackSessionId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) mode: PlaybackSessionMode,
    pub(crate) network_scope: PlaybackTargetNetworkScope,
}

#[derive(Clone, Debug)]
pub(crate) struct IssueRendererTransportTicketRequest {
    pub(crate) principal: AuthenticatedPrincipal,
    pub(crate) scope: RendererTransportTicketScope,
    pub(crate) now_ms: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidateRendererTransportTicketRequest {
    pub(crate) token: String,
    pub(crate) scope: RendererTransportTicketScope,
    pub(crate) now_ms: i64,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct IssuedRendererTransportTicket {
    pub(crate) token: String,
    pub(crate) expires_at_ms: i64,
}

impl fmt::Debug for IssuedRendererTransportTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedRendererTransportTicket")
            .field("token", &"<redacted>")
            .field("expires_at_ms", &self.expires_at_ms)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedRendererTransportTicket {
    pub(crate) principal: AuthenticatedPrincipal,
    pub(crate) scope: RendererTransportTicketScope,
}

#[derive(Clone, Debug)]
pub(crate) struct RendererTransportTicketService {
    store: Arc<Mutex<RendererTransportTicketStore>>,
}

impl RendererTransportTicketService {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(RendererTransportTicketStore::default())),
        }
    }

    pub(crate) fn issue(
        &self,
        request: IssueRendererTransportTicketRequest,
    ) -> Result<IssuedRendererTransportTicket> {
        let expires_at_ms =
            request
                .now_ms
                .checked_add(TICKET_TTL_MS)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "renderer transport ticket expiry overflowed".to_owned(),
                })?;
        let mut store = self
            .store
            .lock()
            .expect("renderer transport ticket store mutex poisoned");
        store.cleanup_expired(request.now_ms);

        for _ in 0..8 {
            let token = generate_ticket_token();
            let token_hash = hash_ticket_token(&token);
            match store.tickets_by_hash.entry(token_hash) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(entry) => {
                    entry.insert(RendererTransportTicketRecord {
                        principal: request.principal,
                        scope: request.scope,
                        expires_at_ms,
                    });
                    return Ok(IssuedRendererTransportTicket {
                        token,
                        expires_at_ms,
                    });
                }
            }
        }

        Err(NakoError::Conflict {
            message: "could not allocate a unique renderer transport ticket".to_owned(),
        })
    }

    pub(crate) fn validate(
        &self,
        request: ValidateRendererTransportTicketRequest,
    ) -> Result<ValidatedRendererTransportTicket> {
        if request.token.trim().is_empty() {
            return Err(invalid_renderer_transport_ticket());
        }

        let token_hash = hash_ticket_token(&request.token);
        let mut store = self
            .store
            .lock()
            .expect("renderer transport ticket store mutex poisoned");
        let Some(record) = store.tickets_by_hash.get(&token_hash) else {
            return Err(invalid_renderer_transport_ticket());
        };

        if record.expires_at_ms <= request.now_ms {
            store.tickets_by_hash.remove(&token_hash);
            return Err(invalid_renderer_transport_ticket());
        }

        if record.scope != request.scope {
            return Err(invalid_renderer_transport_ticket());
        }

        Ok(ValidatedRendererTransportTicket {
            principal: record.principal.clone(),
            scope: record.scope.clone(),
        })
    }

    #[cfg(test)]
    pub(crate) fn ticket_count(&self) -> usize {
        self.store
            .lock()
            .expect("renderer transport ticket store mutex poisoned")
            .tickets_by_hash
            .len()
    }
}

#[derive(Debug, Default)]
struct RendererTransportTicketStore {
    tickets_by_hash: HashMap<String, RendererTransportTicketRecord>,
}

impl RendererTransportTicketStore {
    fn cleanup_expired(&mut self, now_ms: i64) {
        self.tickets_by_hash
            .retain(|_, record| record.expires_at_ms > now_ms);
    }
}

#[derive(Clone, Debug)]
struct RendererTransportTicketRecord {
    principal: AuthenticatedPrincipal,
    scope: RendererTransportTicketScope,
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

fn invalid_renderer_transport_ticket() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid renderer transport ticket".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_transport_ticket_validates_full_scope_and_expiry() {
        let service = RendererTransportTicketService::new();
        let principal = AuthenticatedPrincipal::bootstrap_admin();
        let scope = renderer_transport_scope();
        let issued = service
            .issue(IssueRendererTransportTicketRequest {
                principal: principal.clone(),
                scope: scope.clone(),
                now_ms: 100,
            })
            .unwrap();

        assert!(issued.token.starts_with(TOKEN_PREFIX));
        assert!(!format!("{issued:?}").contains(&issued.token));
        assert!(
            !issued
                .token
                .contains(&scope.renderer_session_id.to_string())
        );
        assert!(
            !issued
                .token
                .contains(&scope.playback_session_id.to_string())
        );
        assert!(!issued.token.contains(&scope.source_id.to_string()));
        assert!(!issued.token.contains(&principal.principal_id.to_string()));
        assert_eq!(issued.expires_at_ms, 100 + TICKET_TTL_MS);

        let validated = service
            .validate(ValidateRendererTransportTicketRequest {
                token: issued.token.clone(),
                scope: scope.clone(),
                now_ms: 101,
            })
            .unwrap();
        assert_eq!(validated.principal, principal);
        assert_eq!(validated.scope, scope);

        for mismatched_scope in mismatched_renderer_transport_scopes(&scope) {
            assert!(
                service
                    .validate(ValidateRendererTransportTicketRequest {
                        token: issued.token.clone(),
                        scope: mismatched_scope,
                        now_ms: 101,
                    })
                    .is_err()
            );
        }

        assert!(
            service
                .validate(ValidateRendererTransportTicketRequest {
                    token: issued.token.clone(),
                    scope: scope.clone(),
                    now_ms: issued.expires_at_ms,
                })
                .is_err()
        );
        assert_eq!(service.ticket_count(), 0);
        assert!(
            service
                .validate(ValidateRendererTransportTicketRequest {
                    token: "not-a-ticket".to_owned(),
                    scope,
                    now_ms: 101,
                })
                .is_err()
        );
    }

    #[test]
    fn issuing_ticket_cleans_expired_records_without_leaking_tokens() {
        let service = RendererTransportTicketService::new();
        let principal = AuthenticatedPrincipal::bootstrap_admin();
        let first = service
            .issue(IssueRendererTransportTicketRequest {
                principal: principal.clone(),
                scope: renderer_transport_scope(),
                now_ms: 100,
            })
            .unwrap();
        let second = service
            .issue(IssueRendererTransportTicketRequest {
                principal,
                scope: renderer_transport_scope(),
                now_ms: first.expires_at_ms,
            })
            .unwrap();

        assert_eq!(service.ticket_count(), 1);
        assert_ne!(first.token, second.token);
        assert!(!format!("{service:?}").contains(&first.token));
        assert!(!format!("{service:?}").contains(&second.token));
    }

    fn renderer_transport_scope() -> RendererTransportTicketScope {
        RendererTransportTicketScope {
            renderer_session_id: RendererSessionId::new(),
            playback_session_id: PlaybackSessionId::new(),
            source_id: MediaSourceId::new(),
            mode: PlaybackSessionMode::Hls,
            network_scope: PlaybackTargetNetworkScope::Local,
        }
    }

    fn mismatched_renderer_transport_scopes(
        scope: &RendererTransportTicketScope,
    ) -> Vec<RendererTransportTicketScope> {
        vec![
            RendererTransportTicketScope {
                renderer_session_id: RendererSessionId::new(),
                ..scope.clone()
            },
            RendererTransportTicketScope {
                playback_session_id: PlaybackSessionId::new(),
                ..scope.clone()
            },
            RendererTransportTicketScope {
                source_id: MediaSourceId::new(),
                ..scope.clone()
            },
            RendererTransportTicketScope {
                mode: PlaybackSessionMode::Direct,
                ..scope.clone()
            },
            RendererTransportTicketScope {
                network_scope: PlaybackTargetNetworkScope::Remote,
                ..scope.clone()
            },
        ]
    }
}
