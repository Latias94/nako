CREATE TABLE addon_routing_plans (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id TEXT NOT NULL,
    manifest_version TEXT NOT NULL,
    manifest_fingerprint TEXT NOT NULL,
    declaration_kind TEXT NOT NULL,
    declaration_id TEXT NOT NULL,
    status TEXT NOT NULL,
    target TEXT NOT NULL,
    safe_reason_code TEXT,
    job_kind TEXT,
    event_kind TEXT,
    plan_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, declaration_kind, declaration_id)
);

CREATE INDEX addon_routing_plans_addon_idx
    ON addon_routing_plans(addon_id, declaration_kind, declaration_id);

CREATE INDEX addon_routing_plans_status_idx
    ON addon_routing_plans(status, target, updated_at);
