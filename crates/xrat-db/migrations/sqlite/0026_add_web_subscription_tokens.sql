CREATE TABLE web_subscription_tokens (
    token_hash TEXT PRIMARY KEY,
    parent_hash TEXT NOT NULL REFERENCES web_tokens(token_hash) ON DELETE CASCADE,
    created_at BIGINT NOT NULL
);
CREATE INDEX web_subscription_tokens_parent ON web_subscription_tokens(parent_hash);
