ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN forced_replay INTEGER NOT NULL DEFAULT 0;

ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN replay_reason_code TEXT;
