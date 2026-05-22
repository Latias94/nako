ALTER TABLE addon_registrations
    DROP CONSTRAINT IF EXISTS addon_registrations_manifest_id_key;

CREATE UNIQUE INDEX IF NOT EXISTS addon_registrations_active_manifest_idx
    ON addon_registrations(manifest_id)
    WHERE status <> 'unregistered';
