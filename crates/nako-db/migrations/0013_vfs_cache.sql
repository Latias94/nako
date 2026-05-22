CREATE TABLE vfs_cache_objects (
    uri TEXT PRIMARY KEY NOT NULL,
    scheme TEXT NOT NULL,
    kind TEXT NOT NULL,
    len INTEGER,
    modified_at TEXT,
    etag TEXT,
    fingerprint TEXT,
    capabilities_bits INTEGER NOT NULL,
    fetched_at_ms INTEGER NOT NULL,
    fresh_until_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX vfs_cache_objects_scheme_idx
    ON vfs_cache_objects(scheme, fresh_until_ms);

CREATE TABLE vfs_cache_listings (
    uri TEXT PRIMARY KEY NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    scheme TEXT NOT NULL,
    fetched_at_ms INTEGER NOT NULL,
    fresh_until_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX vfs_cache_listings_scheme_idx
    ON vfs_cache_listings(scheme, fresh_until_ms);

CREATE TABLE vfs_cache_listing_entries (
    listing_uri TEXT NOT NULL REFERENCES vfs_cache_listings(uri) ON DELETE CASCADE,
    entry_uri TEXT NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (listing_uri, entry_uri)
);

CREATE INDEX vfs_cache_listing_entries_entry_idx
    ON vfs_cache_listing_entries(entry_uri);

CREATE TABLE vfs_cache_failures (
    uri TEXT NOT NULL,
    scheme TEXT NOT NULL,
    operation TEXT NOT NULL,
    failed_at_ms INTEGER NOT NULL,
    failure_count INTEGER NOT NULL,
    error TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (uri, operation)
);

CREATE INDEX vfs_cache_failures_scheme_idx
    ON vfs_cache_failures(scheme, operation, failed_at_ms);
