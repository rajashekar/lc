use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub id: i64,
    pub chat_id: String,
    pub model: String,
    pub question: String,
    pub response: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total_entries: usize,
    pub unique_sessions: usize,
    pub file_size_bytes: u64,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub model_usage: Vec<(String, i64)>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::database_path()?;
        let conn = Connection::open(&db_path)?;
        
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }
    
    fn initialize(&self) -> Result<()> {
        // Create chat_logs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                model TEXT NOT NULL,
                question TEXT NOT NULL,
                response TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create session_state table for tracking current session
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS session_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create indexes for better performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_logs_chat_id ON chat_logs(chat_id)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_logs_timestamp ON chat_logs(timestamp)",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn save_chat_entry(
        &self,
        chat_id: &str,
        model: &str,
        question: &str,
        response: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO chat_logs (chat_id, model, question, response, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![chat_id, model, question, response, Utc::now()],
        )?;
        Ok(())
    }
    
    pub fn get_chat_history(&self, chat_id: &str) -> Result<Vec<ChatEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chat_id, model, question, response, timestamp
             FROM chat_logs
             WHERE chat_id = ?1
             ORDER BY timestamp ASC"
        )?;
        
        let rows = stmt.query_map([chat_id], |row| {
            Ok(ChatEntry {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                model: row.get(2)?,
                question: row.get(3)?,
                response: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    pub fn get_last_response(&self) -> Result<Option<ChatEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chat_id, model, question, response, timestamp
             FROM chat_logs
             ORDER BY timestamp DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query_map([], |row| {
            Ok(ChatEntry {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                model: row.get(2)?,
                question: row.get(3)?,
                response: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_all_logs(&self) -> Result<Vec<ChatEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chat_id, model, question, response, timestamp
             FROM chat_logs
             ORDER BY timestamp DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(ChatEntry {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                model: row.get(2)?,
                question: row.get(3)?,
                response: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    pub fn set_current_session_id(&self, session_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO session_state (key, value) VALUES ('current_session', ?1)",
            [session_id],
        )?;
        Ok(())
    }
    
    pub fn get_current_session_id(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM session_state WHERE key = 'current_session'"
        )?;
        
        let mut rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    pub fn purge_all_logs(&self) -> Result<()> {
        self.conn.execute("DELETE FROM chat_logs", [])?;
        self.conn.execute("DELETE FROM session_state", [])?;
        Ok(())
    }
    
    pub fn clear_session(&self, session_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM chat_logs WHERE chat_id = ?1",
            [session_id],
        )?;
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        // Get total number of entries
        let total_entries: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM chat_logs",
            [],
            |row| row.get(0)
        )?;
        
        // Get number of unique sessions
        let unique_sessions: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT chat_id) FROM chat_logs",
            [],
            |row| row.get(0)
        )?;
        
        // Get database file size
        let db_path = Self::database_path()?;
        let file_size = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // Get earliest and latest timestamps
        let date_range = if total_entries > 0 {
            let earliest: Option<DateTime<Utc>> = self.conn.query_row(
                "SELECT MIN(timestamp) FROM chat_logs",
                [],
                |row| row.get(0)
            ).ok();
            
            let latest: Option<DateTime<Utc>> = self.conn.query_row(
                "SELECT MAX(timestamp) FROM chat_logs",
                [],
                |row| row.get(0)
            ).ok();
            
            match (earliest, latest) {
                (Some(e), Some(l)) => Some((e, l)),
                _ => None,
            }
        } else {
            None
        };
        
        // Get model usage statistics
        let mut stmt = self.conn.prepare(
            "SELECT model, COUNT(*) as count FROM chat_logs GROUP BY model ORDER BY count DESC"
        )?;
        
        let model_stats = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(DatabaseStats {
            total_entries: total_entries as usize,
            unique_sessions: unique_sessions as usize,
            file_size_bytes: file_size,
            date_range,
            model_usage: model_stats,
        })
    }
    
    fn database_path() -> Result<PathBuf> {
        let config_dir = Config::config_dir()?;
        Ok(config_dir.join("logs.db"))
    }
}