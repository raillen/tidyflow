CREATE TABLE IF NOT EXISTS admin_commands (
    id TEXT PRIMARY KEY NOT NULL,
    source TEXT NOT NULL,
    request_payload TEXT NOT NULL,
    status TEXT NOT NULL,
    result_payload TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_admin_commands_status ON admin_commands(status);
CREATE INDEX IF NOT EXISTS idx_admin_commands_created_at ON admin_commands(created_at);
