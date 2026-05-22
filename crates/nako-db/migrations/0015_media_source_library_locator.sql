DROP INDEX IF EXISTS media_sources_locator_idx;

CREATE UNIQUE INDEX IF NOT EXISTS media_sources_library_locator_idx
    ON media_sources(library_id, locator);
