use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRecord {
    pub id: i64,
    pub pid: Option<u32>,
    pub process_name: String,
    pub protocol: String,
    pub src_port: u16,
    pub dst_ip: String,
    pub dst_host: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub is_blocked: bool,
    pub is_trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub id: i64,
    pub connection_id: Option<i64>,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub process_name: Option<String>,
    pub triggered_at: String,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsQueryRecord {
    pub id: i64,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub query_name: String,
    pub query_type: String,
    pub resolver_ip: Option<String>,
    pub response_ip: Option<String>,
    pub captured_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    pub process_name: Option<String>,
    pub country_code: Option<String>,
    pub hostname: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub fn upsert_connection(conn: &Connection, rec: &ConnectionRecord) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO connections (pid, process_name, protocol, src_port, dst_ip, dst_host,
         country_code, city, bytes_sent, bytes_recv, first_seen, last_seen, is_blocked, is_trusted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            bytes_sent = bytes_sent + excluded.bytes_sent,
            bytes_recv = bytes_recv + excluded.bytes_recv,
            last_seen = excluded.last_seen",
        params![
            rec.pid.map(|p| p as i64),
            rec.process_name,
            rec.protocol,
            rec.src_port as i64,
            rec.dst_ip,
            rec.dst_host,
            rec.country_code,
            rec.city,
            rec.bytes_sent as i64,
            rec.bytes_recv as i64,
            rec.first_seen,
            rec.last_seen,
            rec.is_blocked as i64,
            rec.is_trusted as i64,
        ],
    )
    .map_err(|e| format!("Failed to upsert connection: {e}"))?;

    Ok(conn.last_insert_rowid())
}

pub fn update_connection_bytes(
    conn: &Connection,
    dst_ip: &str,
    src_port: u16,
    process_name: &str,
    bytes_sent: u64,
    bytes_recv: u64,
    last_seen: &str,
) -> Result<(), String> {
    let affected = conn
        .execute(
            "UPDATE connections SET
                bytes_sent = bytes_sent + ?1,
                bytes_recv = bytes_recv + ?2,
                last_seen = ?3
             WHERE dst_ip = ?4 AND src_port = ?5 AND process_name = ?6
             ORDER BY first_seen DESC LIMIT 1",
            params![
                bytes_sent as i64,
                bytes_recv as i64,
                last_seen,
                dst_ip,
                src_port as i64,
                process_name,
            ],
        )
        .map_err(|e| format!("Failed to update connection bytes: {e}"))?;

    if affected == 0 {
        conn.execute(
            "INSERT INTO connections (pid, process_name, protocol, src_port, dst_ip, bytes_sent, bytes_recv, first_seen, last_seen)
             VALUES (NULL, ?1, 'TCP', ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                process_name,
                src_port as i64,
                dst_ip,
                bytes_sent as i64,
                bytes_recv as i64,
                last_seen,
            ],
        )
        .map_err(|e| format!("Failed to insert new connection: {e}"))?;
    }

    Ok(())
}

pub fn get_active_connections(conn: &Connection) -> Result<Vec<ConnectionRecord>, String> {
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::seconds(30))
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, pid, process_name, protocol, src_port, dst_ip, dst_host,
                    country_code, city, bytes_sent, bytes_recv, first_seen, last_seen,
                    is_blocked, is_trusted
             FROM connections
             WHERE last_seen >= ?1
             ORDER BY last_seen DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(params![cutoff], |row| {
            Ok(ConnectionRecord {
                id: row.get(0)?,
                pid: row.get::<_, Option<i64>>(1)?.map(|v| v as u32),
                process_name: row.get(2)?,
                protocol: row.get(3)?,
                src_port: row.get::<_, i64>(4)? as u16,
                dst_ip: row.get(5)?,
                dst_host: row.get(6)?,
                country_code: row.get(7)?,
                city: row.get(8)?,
                bytes_sent: row.get::<_, i64>(9)? as u64,
                bytes_recv: row.get::<_, i64>(10)? as u64,
                first_seen: row.get(11)?,
                last_seen: row.get(12)?,
                is_blocked: row.get::<_, i64>(13)? != 0,
                is_trusted: row.get::<_, i64>(14)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query connections: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {e}"))?);
    }
    Ok(results)
}

pub fn get_connection_history(
    conn: &Connection,
    filter: &HistoryFilter,
) -> Result<Vec<ConnectionRecord>, String> {
    let mut sql = String::from(
        "SELECT id, pid, process_name, protocol, src_port, dst_ip, dst_host,
                country_code, city, bytes_sent, bytes_recv, first_seen, last_seen,
                is_blocked, is_trusted
         FROM connections WHERE 1=1",
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref name) = filter.process_name {
        sql.push_str(" AND process_name LIKE ?");
        param_values.push(Box::new(format!("%{name}%")));
    }
    if let Some(ref code) = filter.country_code {
        sql.push_str(" AND country_code = ?");
        param_values.push(Box::new(code.clone()));
    }
    if let Some(ref host) = filter.hostname {
        sql.push_str(" AND (dst_host LIKE ? OR dst_ip LIKE ?)");
        param_values.push(Box::new(format!("%{host}%")));
        param_values.push(Box::new(format!("%{host}%")));
    }
    if let Some(ref start) = filter.start_date {
        sql.push_str(" AND first_seen >= ?");
        param_values.push(Box::new(start.clone()));
    }
    if let Some(ref end) = filter.end_date {
        sql.push_str(" AND first_seen <= ?");
        param_values.push(Box::new(end.clone()));
    }

    sql.push_str(" ORDER BY first_seen DESC");

    let limit = filter.limit.unwrap_or(500);
    let offset = filter.offset.unwrap_or(0);
    sql.push_str(&format!(" LIMIT {limit} OFFSET {offset}"));

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare history query: {e}"))?;

    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(ConnectionRecord {
                id: row.get(0)?,
                pid: row.get::<_, Option<i64>>(1)?.map(|v| v as u32),
                process_name: row.get(2)?,
                protocol: row.get(3)?,
                src_port: row.get::<_, i64>(4)? as u16,
                dst_ip: row.get(5)?,
                dst_host: row.get(6)?,
                country_code: row.get(7)?,
                city: row.get(8)?,
                bytes_sent: row.get::<_, i64>(9)? as u64,
                bytes_recv: row.get::<_, i64>(10)? as u64,
                first_seen: row.get(11)?,
                last_seen: row.get(12)?,
                is_blocked: row.get::<_, i64>(13)? != 0,
                is_trusted: row.get::<_, i64>(14)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query history: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {e}"))?);
    }
    Ok(results)
}

pub fn insert_alert(conn: &Connection, alert: &AlertRecord) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO alerts (connection_id, rule_name, severity, message, process_name, triggered_at, dismissed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            alert.connection_id,
            alert.rule_name,
            alert.severity,
            alert.message,
            alert.process_name,
            alert.triggered_at,
            alert.dismissed as i64,
        ],
    )
    .map_err(|e| format!("Failed to insert alert: {e}"))?;

    Ok(conn.last_insert_rowid())
}

pub fn get_alerts(conn: &Connection, include_dismissed: bool) -> Result<Vec<AlertRecord>, String> {
    let sql = if include_dismissed {
        "SELECT id, connection_id, rule_name, severity, message, process_name, triggered_at, dismissed
         FROM alerts ORDER BY triggered_at DESC LIMIT 200"
    } else {
        "SELECT id, connection_id, rule_name, severity, message, process_name, triggered_at, dismissed
         FROM alerts WHERE dismissed = 0 ORDER BY triggered_at DESC LIMIT 200"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| format!("Failed to prepare alerts query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AlertRecord {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                rule_name: row.get(2)?,
                severity: row.get(3)?,
                message: row.get(4)?,
                process_name: row.get(5)?,
                triggered_at: row.get(6)?,
                dismissed: row.get::<_, i64>(7)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query alerts: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read alert row: {e}"))?);
    }
    Ok(results)
}

pub fn dismiss_alert(conn: &Connection, alert_id: i64) -> Result<(), String> {
    conn.execute("UPDATE alerts SET dismissed = 1 WHERE id = ?1", params![alert_id])
        .map_err(|e| format!("Failed to dismiss alert: {e}"))?;
    Ok(())
}

pub fn insert_dns_query(conn: &Connection, record: &DnsQueryRecord) -> Result<(), String> {
    conn.execute(
        "INSERT INTO dns_queries (pid, process_name, query_name, query_type, resolver_ip, response_ip, captured_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            record.pid.map(|p| p as i64),
            record.process_name,
            record.query_name,
            record.query_type,
            record.resolver_ip,
            record.response_ip,
            record.captured_at,
        ],
    )
    .map_err(|e| format!("Failed to insert DNS query: {e}"))?;
    Ok(())
}

pub fn get_dns_queries(
    conn: &Connection,
    pid: Option<u32>,
) -> Result<Vec<DnsQueryRecord>, String> {
    let (sql, param_values): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(pid_val) = pid {
        (
            "SELECT id, pid, process_name, query_name, query_type, resolver_ip, response_ip, captured_at
             FROM dns_queries WHERE pid = ? ORDER BY captured_at DESC LIMIT 500",
            vec![Box::new(pid_val as i64) as Box<dyn rusqlite::types::ToSql>],
        )
    } else {
        (
            "SELECT id, pid, process_name, query_name, query_type, resolver_ip, response_ip, captured_at
             FROM dns_queries ORDER BY captured_at DESC LIMIT 500",
            vec![],
        )
    };

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(sql).map_err(|e| format!("Failed to prepare DNS query: {e}"))?;

    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(DnsQueryRecord {
                id: row.get(0)?,
                pid: row.get::<_, Option<i64>>(1)?.map(|v| v as u32),
                process_name: row.get(2)?,
                query_name: row.get(3)?,
                query_type: row.get(4)?,
                resolver_ip: row.get(5)?,
                response_ip: row.get(6)?,
                captured_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query DNS records: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read DNS row: {e}"))?);
    }
    Ok(results)
}

pub fn block_app(conn: &Connection, process_name: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO blocked_apps (process_name, blocked_at) VALUES (?1, ?2)",
        params![process_name, now],
    )
    .map_err(|e| format!("Failed to block app: {e}"))?;

    conn.execute(
        "DELETE FROM trusted_apps WHERE process_name = ?1",
        params![process_name],
    )
    .map_err(|e| format!("Failed to remove from trusted: {e}"))?;

    conn.execute(
        "UPDATE connections SET is_blocked = 1 WHERE process_name = ?1",
        params![process_name],
    )
    .map_err(|e| format!("Failed to update connections: {e}"))?;

    Ok(())
}

pub fn trust_app(conn: &Connection, process_name: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO trusted_apps (process_name, trusted_at) VALUES (?1, ?2)",
        params![process_name, now],
    )
    .map_err(|e| format!("Failed to trust app: {e}"))?;

    conn.execute(
        "DELETE FROM blocked_apps WHERE process_name = ?1",
        params![process_name],
    )
    .map_err(|e| format!("Failed to remove from blocked: {e}"))?;

    conn.execute(
        "UPDATE connections SET is_trusted = 1, is_blocked = 0 WHERE process_name = ?1",
        params![process_name],
    )
    .map_err(|e| format!("Failed to update connections: {e}"))?;

    Ok(())
}

pub fn is_blocked(conn: &Connection, process_name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM blocked_apps WHERE process_name = ?1",
            params![process_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check blocked status: {e}"))?;
    Ok(count > 0)
}

pub fn is_trusted(conn: &Connection, process_name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM trusted_apps WHERE process_name = ?1",
            params![process_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check trusted status: {e}"))?;
    Ok(count > 0)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get setting: {e}")),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| format!("Failed to set setting: {e}"))?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<std::collections::HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| format!("Failed to prepare settings query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query settings: {e}"))?;

    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| format!("Failed to read setting: {e}"))?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn get_blocked_apps(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT process_name FROM blocked_apps")
        .map_err(|e| format!("Failed to query blocked apps: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query blocked apps: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read blocked app: {e}"))?);
    }
    Ok(results)
}

pub fn get_trusted_apps(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT process_name FROM trusted_apps")
        .map_err(|e| format!("Failed to query trusted apps: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query trusted apps: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read trusted app: {e}"))?);
    }
    Ok(results)
}

pub fn cleanup_old_records(conn: &Connection, days: u32) -> Result<u64, String> {
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(days as i64))
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339();

    let deleted = conn
        .execute(
            "DELETE FROM connections WHERE last_seen < ?1",
            params![cutoff],
        )
        .map_err(|e| format!("Failed to cleanup old connections: {e}"))?;

    conn.execute(
        "DELETE FROM dns_queries WHERE captured_at < ?1",
        params![cutoff],
    )
    .map_err(|e| format!("Failed to cleanup old DNS queries: {e}"))?;

    conn.execute(
        "DELETE FROM alerts WHERE triggered_at < ?1 AND dismissed = 1",
        params![cutoff],
    )
    .map_err(|e| format!("Failed to cleanup old alerts: {e}"))?;

    Ok(deleted as u64)
}
