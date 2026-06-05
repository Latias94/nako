use std::collections::HashMap;

use nako_core::AddonId;

pub(super) const ADDON_RESOURCE_FLOW_SESSION_TTL_MS: i64 = 15 * 60 * 1_000;
pub(super) const ADDON_RESOURCE_FLOW_SESSION_MAX_COUNT: usize = 64;

#[derive(Clone, Debug)]
pub(super) struct SelectionSession<TSelection, TContext> {
    search_id: String,
    addon_id: AddonId,
    manifest_id: String,
    context: TContext,
    created_at_ms: i64,
    expires_at_ms: i64,
    selections: HashMap<String, TSelection>,
}

impl<TSelection, TContext> SelectionSession<TSelection, TContext> {
    pub(super) fn new(
        search_id: String,
        addon_id: AddonId,
        manifest_id: String,
        context: TContext,
        created_at_ms: i64,
        selections: HashMap<String, TSelection>,
    ) -> Self {
        Self {
            search_id,
            addon_id,
            manifest_id,
            context,
            created_at_ms,
            expires_at_ms: created_at_ms.saturating_add(ADDON_RESOURCE_FLOW_SESSION_TTL_MS),
            selections,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectionSessionHandoff<TSelection, TContext> {
    pub(super) manifest_id: String,
    pub(super) context: TContext,
    pub(super) selection: TSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SelectionSessionLookup<TSelection, TContext> {
    Found(SelectionSessionHandoff<TSelection, TContext>),
    Missing,
    ManifestMismatch,
}

#[derive(Debug)]
pub(super) struct SelectionSessionStore<TSelection, TContext> {
    sessions: HashMap<String, SelectionSession<TSelection, TContext>>,
    max_count: usize,
}

impl<TSelection, TContext> Default for SelectionSessionStore<TSelection, TContext> {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            max_count: ADDON_RESOURCE_FLOW_SESSION_MAX_COUNT,
        }
    }
}

impl<TSelection, TContext> SelectionSessionStore<TSelection, TContext>
where
    TSelection: Clone,
    TContext: Clone,
{
    pub(super) fn insert(&mut self, session: SelectionSession<TSelection, TContext>) {
        self.prune(session.created_at_ms);
        self.sessions.insert(session.search_id.clone(), session);
        self.enforce_max_count();
    }

    pub(super) fn get_selection(
        &mut self,
        addon_id: AddonId,
        manifest_id: &str,
        search_id: &str,
        selection_id: &str,
        now_ms: i64,
    ) -> SelectionSessionLookup<TSelection, TContext> {
        self.prune(now_ms);
        let Some(session) = self.sessions.get(search_id) else {
            return SelectionSessionLookup::Missing;
        };
        if session.addon_id != addon_id {
            return SelectionSessionLookup::Missing;
        }
        let Some(selection) = session.selections.get(selection_id).cloned() else {
            return SelectionSessionLookup::Missing;
        };
        if session.manifest_id != manifest_id {
            return SelectionSessionLookup::ManifestMismatch;
        }

        SelectionSessionLookup::Found(SelectionSessionHandoff {
            manifest_id: session.manifest_id.clone(),
            context: session.context.clone(),
            selection,
        })
    }

    fn prune(&mut self, now_ms: i64) {
        self.sessions
            .retain(|_, session| session.expires_at_ms > now_ms);
    }

    fn enforce_max_count(&mut self) {
        while self.sessions.len() > self.max_count {
            let Some(oldest_search_id) = self
                .sessions
                .iter()
                .min_by_key(|(_, session)| session.created_at_ms)
                .map(|(search_id, _)| search_id.clone())
            else {
                break;
            };
            self.sessions.remove(&oldest_search_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_session_store_returns_selected_handoff() {
        let addon_id = AddonId::new();
        let mut store = SelectionSessionStore::default();
        store.insert(SelectionSession::new(
            "search".to_owned(),
            addon_id,
            "manifest.v1".to_owned(),
            "query-context".to_owned(),
            1_000,
            HashMap::from([("selection".to_owned(), "payload".to_owned())]),
        ));

        let lookup = store.get_selection(addon_id, "manifest.v1", "search", "selection", 1_001);

        assert_eq!(
            lookup,
            SelectionSessionLookup::Found(SelectionSessionHandoff {
                manifest_id: "manifest.v1".to_owned(),
                context: "query-context".to_owned(),
                selection: "payload".to_owned(),
            })
        );
    }

    #[test]
    fn selection_session_store_prunes_expired_sessions() {
        let addon_id = AddonId::new();
        let mut store = SelectionSessionStore::default();
        store.insert(SelectionSession::new(
            "search".to_owned(),
            addon_id,
            "manifest.v1".to_owned(),
            (),
            1_000,
            HashMap::from([("selection".to_owned(), "payload".to_owned())]),
        ));

        let lookup = store.get_selection(
            addon_id,
            "manifest.v1",
            "search",
            "selection",
            1_000 + ADDON_RESOURCE_FLOW_SESSION_TTL_MS,
        );

        assert_eq!(lookup, SelectionSessionLookup::Missing);
    }

    #[test]
    fn selection_session_store_evicts_oldest_sessions() {
        let addon_id = AddonId::new();
        let mut store = SelectionSessionStore::default();
        for index in 0..=ADDON_RESOURCE_FLOW_SESSION_MAX_COUNT {
            store.insert(SelectionSession::new(
                format!("search-{index}"),
                addon_id,
                "manifest.v1".to_owned(),
                (),
                index as i64,
                HashMap::from([("selection".to_owned(), index)]),
            ));
        }

        assert_eq!(
            store.get_selection(addon_id, "manifest.v1", "search-0", "selection", 100),
            SelectionSessionLookup::Missing
        );
        assert!(matches!(
            store.get_selection(
                addon_id,
                "manifest.v1",
                &format!("search-{ADDON_RESOURCE_FLOW_SESSION_MAX_COUNT}"),
                "selection",
                100,
            ),
            SelectionSessionLookup::Found(_)
        ));
    }

    #[test]
    fn selection_session_store_detects_manifest_mismatch_after_selection_lookup() {
        let addon_id = AddonId::new();
        let mut store = SelectionSessionStore::default();
        store.insert(SelectionSession::new(
            "search".to_owned(),
            addon_id,
            "manifest.v1".to_owned(),
            (),
            1_000,
            HashMap::from([("selection".to_owned(), "payload".to_owned())]),
        ));

        let mismatch = store.get_selection(addon_id, "manifest.v2", "search", "selection", 1_001);
        let missing_selection =
            store.get_selection(addon_id, "manifest.v2", "missing", "selection", 1_001);

        assert_eq!(mismatch, SelectionSessionLookup::ManifestMismatch);
        assert_eq!(missing_selection, SelectionSessionLookup::Missing);
    }
}
