CREATE TABLE IF NOT EXISTS rollback_manifests (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL,
    execution_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    rolled_back_at TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rollback_job_id ON rollback_manifests(job_id);
CREATE INDEX IF NOT EXISTS idx_rollback_created_at ON rollback_manifests(created_at);