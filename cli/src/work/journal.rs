use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Journal {
    conn: Connection,
}

impl Journal {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // WAL mode for performance and concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;
        
        // Create basic tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS actions (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                action_data TEXT NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                state_data TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn save_action(&self, id: &str, task_id: &str, timestamp: &str, data: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO actions (id, task_id, timestamp, action_data) VALUES (?1, ?2, ?3, ?4)",
            params![id, task_id, timestamp, data],
        )?;
        Ok(())
    }

    pub fn save_checkpoint(&self, id: &str, task_id: &str, timestamp: &str, data: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO checkpoints (id, task_id, timestamp, state_data) VALUES (?1, ?2, ?3, ?4)",
            params![id, task_id, timestamp, data],
        )?;
        Ok(())
    }
}
