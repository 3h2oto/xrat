CREATE INDEX idx_connection_tests_config_latest
ON connection_tests (config_id, tested_at DESC, id DESC);
