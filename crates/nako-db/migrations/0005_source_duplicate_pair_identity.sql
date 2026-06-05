CREATE UNIQUE INDEX IF NOT EXISTS source_duplicate_relationships_pair_idx
    ON source_duplicate_relationships(source_id, duplicate_source_id);
