ALTER TABLE addon_side_effects
    ADD COLUMN apply_status TEXT NOT NULL DEFAULT 'pending';

ALTER TABLE addon_side_effects
    ADD COLUMN apply_error_code TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_item_id TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_source TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_at TEXT;

CREATE INDEX addon_side_effects_apply_status_idx
    ON addon_side_effects(apply_status, created_at, id);
