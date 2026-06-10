-- ColGREP Cloud: Initial schema
-- Users and codebases keyed by user_id

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Codebases table (one per user project)
CREATE TABLE IF NOT EXISTS codebases (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    file_count INTEGER DEFAULT 0,
    code_unit_count INTEGER DEFAULT 0,
    last_indexed TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_codebases_user ON codebases(user_id);

-- Code units metadata (for search - full code in R2)
CREATE TABLE IF NOT EXISTS code_units (
    id TEXT PRIMARY KEY,
    codebase_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    code TEXT NOT NULL,
    unit_type TEXT NOT NULL,
    language TEXT,
    r2_key TEXT,
    FOREIGN KEY (codebase_id) REFERENCES codebases(id)
);

CREATE INDEX IF NOT EXISTS idx_code_units_codebase ON code_units(codebase_id);
CREATE INDEX IF NOT EXISTS idx_code_units_file ON code_units(file_path);
