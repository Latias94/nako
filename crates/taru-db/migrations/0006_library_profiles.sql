ALTER TABLE libraries ADD COLUMN domain TEXT NOT NULL DEFAULT 'mixed';
ALTER TABLE libraries ADD COLUMN preset TEXT NOT NULL DEFAULT 'mixed_video';
ALTER TABLE libraries ADD COLUMN options_json TEXT;
