CREATE UNIQUE INDEX IF NOT EXISTS addon_registrations_active_manifest_idx
    ON addon_registrations(manifest_id)
    WHERE status <> 'unregistered';
