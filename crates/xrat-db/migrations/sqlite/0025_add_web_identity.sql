CREATE TABLE web_users (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'operator', 'viewer')),
    enabled BIGINT NOT NULL DEFAULT 1,
    must_change_password BIGINT NOT NULL DEFAULT 1,
    credential_version TEXT NOT NULL
);
CREATE TABLE web_sessions (
    token_hash TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    credential_version TEXT NOT NULL,
    csrf_hash TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    accessed_at BIGINT NOT NULL
);
CREATE INDEX web_sessions_username ON web_sessions (username);
CREATE TABLE web_tokens (
    token_hash TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    credential_version TEXT NOT NULL,
    scope TEXT NOT NULL CHECK (scope IN ('read', 'manage')),
    name TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX web_tokens_username ON web_tokens (username);
CREATE TABLE web_jobs (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    result TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
ALTER TABLE events ADD COLUMN actor TEXT;
