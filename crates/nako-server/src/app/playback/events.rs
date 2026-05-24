use nako_core::{
    DomainEventKind, DomainEventSubject, EventId, NewOutboxEvent, TranscodeSessionRecord,
};
use tracing::warn;

pub(super) async fn record_playback_session_finished_event(
    store: &dyn super::PlaybackRuntimeStore,
    session: &TranscodeSessionRecord,
) {
    let payload = serde_json::json!({
        "session_id": session.id,
        "source_id": session.source_id,
        "kind": session.kind,
        "request_key": &session.request_key,
        "state": session.state,
    });
    let idempotency_key = format!("playback.session_finished:{}", session.id);
    if let Err(err) = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::PlaybackSessionFinished,
            subject: DomainEventSubject::PlaybackSession(session.id),
            library_id: None,
            source_id: Some(session.source_id),
            idempotency_key: idempotency_key.clone(),
            payload_json: payload.to_string(),
        })
        .await
    {
        warn!(
            session_id = %session.id,
            idempotency_key,
            error = %err,
            "failed to persist playback session outbox event"
        );
    }
}
