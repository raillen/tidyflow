CREATE TABLE IF NOT EXISTS blueprints (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS blueprint_counters (
    blueprint_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    value INTEGER NOT NULL,
    PRIMARY KEY (blueprint_id, scope_key)
);

CREATE INDEX IF NOT EXISTS idx_blueprints_enabled ON blueprints(enabled);
