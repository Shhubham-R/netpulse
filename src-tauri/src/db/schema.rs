pub const CREATE_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER,
    process_name TEXT NOT NULL DEFAULT 'unknown',
    protocol TEXT NOT NULL DEFAULT 'TCP',
    src_port INTEGER NOT NULL DEFAULT 0,
    dst_ip TEXT NOT NULL,
    dst_host TEXT,
    country_code TEXT,
    city TEXT,
    bytes_sent INTEGER NOT NULL DEFAULT 0,
    bytes_recv INTEGER NOT NULL DEFAULT 0,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    is_blocked INTEGER NOT NULL DEFAULT 0,
    is_trusted INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_connections_process ON connections(process_name);
CREATE INDEX IF NOT EXISTS idx_connections_dst_ip ON connections(dst_ip);
CREATE INDEX IF NOT EXISTS idx_connections_first_seen ON connections(first_seen);

CREATE TABLE IF NOT EXISTS alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER,
    rule_name TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    message TEXT NOT NULL DEFAULT '',
    process_name TEXT,
    triggered_at TEXT NOT NULL,
    dismissed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

CREATE INDEX IF NOT EXISTS idx_alerts_dismissed ON alerts(dismissed);
CREATE INDEX IF NOT EXISTS idx_alerts_triggered_at ON alerts(triggered_at);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dns_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER,
    process_name TEXT,
    query_name TEXT NOT NULL,
    query_type TEXT NOT NULL DEFAULT 'A',
    resolver_ip TEXT,
    response_ip TEXT,
    captured_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dns_queries_process ON dns_queries(process_name);
CREATE INDEX IF NOT EXISTS idx_dns_queries_captured_at ON dns_queries(captured_at);

CREATE TABLE IF NOT EXISTS blocked_apps (
    process_name TEXT PRIMARY KEY,
    blocked_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trusted_apps (
    process_name TEXT PRIMARY KEY,
    trusted_at TEXT NOT NULL
);
"#;
