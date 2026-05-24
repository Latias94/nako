ALTER TABLE addon_registrations
    ADD COLUMN IF NOT EXISTS outbound_task_dispatch_secret_env text;
