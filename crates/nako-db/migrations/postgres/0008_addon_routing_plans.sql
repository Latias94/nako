CREATE TABLE IF NOT EXISTS addon_routing_plans (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id text NOT NULL,
    manifest_version text NOT NULL,
    manifest_fingerprint text NOT NULL,
    declaration_kind text NOT NULL,
    declaration_id text NOT NULL,
    status text NOT NULL,
    target text NOT NULL,
    safe_reason_code text,
    job_kind text,
    event_kind text,
    plan_json text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(addon_id, declaration_kind, declaration_id)
);

CREATE INDEX IF NOT EXISTS addon_routing_plans_addon_idx
    ON addon_routing_plans(addon_id, declaration_kind, declaration_id);

CREATE INDEX IF NOT EXISTS addon_routing_plans_status_idx
    ON addon_routing_plans(status, target, updated_at);
