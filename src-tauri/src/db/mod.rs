pub mod schema;
pub mod queries;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create data directory: {e}"))?;

        let db_path = app_data_dir.join("netpulse.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database at {}: {e}", db_path.display()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|e| format!("Failed to set pragmas: {e}"))?;

        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn run_migrations(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(schema::CREATE_TABLES)
            .map_err(|e| format!("Failed to run migrations: {e}"))?;
        Ok(())
    }
}
