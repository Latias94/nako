ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN forced_replay BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN replay_reason_code TEXT;
