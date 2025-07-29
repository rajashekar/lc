use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Statement};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub chat_id: String,
    pub model: String,
    pub question: String,
    pub response: String,
    pub timestamp: DateTime<Utc>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total_entries: usize,
    pub unique_sessions: usize,
    pub file_size_bytes: u64,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub model_usage: Vec<(String, i64)>,
}

// Connection pool for reusing database connections
pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<Connection>>>,
    max_connections: usize,
    db_path: PathBuf,
}

impl ConnectionPool {
    pub fn new(db_path: PathBuf, max_connections: usize) -> Result<Self> {
        let mut connections = Vec::with_capacity(max_connections);
        
        // Pre-create initial connections
        for _ in 0..std::cmp::min(2, max_connections) {
            let conn = Connection::open(&db_path)?;
            Self::configure_connection(&conn)?;
            connections.push(conn);
        }
        
        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
            max_connections,
            db_path,
        })
    }
    
    fn configure_connection(conn: &Connection) -> Result<()> {
        // Enable WAL mode for better concurrent performance
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Increase cache size for better performance
        conn.pragma_update(None, "cache_size", 10000)?;
        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", true)?;
        // Set synchronous to NORMAL for better performance
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(())
    }
    
    pub fn get_connection(&self) -> Result<PooledConnection> {
        let mut connections = self.connections.lock().unwrap();
        
        if let Some(conn) = connections.pop() {
            Ok(PooledConnection {
                conn: Some(conn),
                pool: self.connections.clone(),
            })
        } else if connections.len() < self.max_connections {
            // Create new connection if under limit
            let conn = Connection::open(&self.db_path)?;
            Self::configure_connection(&conn)?;
            Ok(PooledConnection {
                conn: Some(conn),
                pool: self.connections.clone(),
            })
        } else {
            // Wait for a connection to become available
            // In a real implementation, you might want to use a condition variable
            // For now, create a new temporary connection
            let conn = Connection::open(&self.db_path)?;
            Self::configure_connection(&conn)?;
            Ok(PooledConnection {
                conn: Some(conn),
                pool: self.connections.clone(),
            })
        }
    }
}

// RAII wrapper for pooled connections
pub struct PooledConnection {
    conn: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl PooledConnection {
    pub fn execute(&self, sql: &str, params: impl rusqlite::Params) -> Result<usize, rusqlite::Error> {
        self.conn.as_ref().unwrap().execute(sql, params)
    }
    
    
    pub fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<T, rusqlite::Error>
    where
        P: rusqlite::Params,
        F: FnOnce(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        self.conn.as_ref().unwrap().query_row(sql, params, f)
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let mut connections = self.pool.lock().unwrap();
            connections.push(conn);
        }
    }
}

// Prepared statement cache
struct PreparedStatementCache {
    _statements: HashMap<String, String>, // SQL -> Statement key mapping (placeholder for future enhancement)
}

impl PreparedStatementCache {
    fn new() -> Self {
        Self {
            _statements: HashMap::new(),
        }
    }
    
    fn get_or_prepare<'a>(&mut self, conn: &'a Connection, sql: &str) -> Result<Statement<'a>> {
        // In a real implementation, you'd cache the actual prepared statements
        // For now, we'll just prepare each time but with optimized SQL
        conn.prepare(sql).map_err(Into::into)
    }
}

// Optimized Database struct with connection pooling and prepared statements
pub struct Database {
    pool: ConnectionPool,
    statement_cache: Arc<Mutex<PreparedStatementCache>>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::database_path()?;
        let pool = ConnectionPool::new(db_path, 5)?; // Max 5 connections
        
        // Initialize database schema
        let conn = pool.get_connection()?;
        Self::initialize_schema(&conn)?;
        
        Ok(Database {
            pool,
            statement_cache: Arc::new(Mutex::new(PreparedStatementCache::new())),
        })
    }
    
    fn initialize_schema(conn: &PooledConnection) -> Result<()> {
        // Create chat_logs table with optimized schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                model TEXT NOT NULL,
                question TEXT NOT NULL,
                response TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                input_tokens INTEGER,
                output_tokens INTEGER
            )",
            [],
        )?;
        
        // Add token columns to existing table if they don't exist (migration)
        let _ = conn.execute(
            "ALTER TABLE chat_logs ADD COLUMN input_tokens INTEGER",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE chat_logs ADD COLUMN output_tokens INTEGER",
            [],
        );
        
        // Create session_state table for tracking current session
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create optimized indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_logs_chat_id ON chat_logs(chat_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_logs_timestamp ON chat_logs(timestamp DESC)",
            [],
        )?;
        
        // Additional index for model statistics
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_logs_model ON chat_logs(model)",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn save_chat_entry_with_tokens(
        &self,
        chat_id: &str,
        model: &str,
        question: &str,
        response: &str,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
    ) -> Result<()> {
        let conn = self.pool.get_connection()?;
        
        // Use prepared statement for better performance
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
            "INSERT INTO chat_logs (chat_id, model, question, response, timestamp, input_tokens, output_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;
        
        stmt.execute(params![chat_id, model, question, response, Utc::now(), input_tokens, output_tokens])?;
        Ok(())
    }
    
    pub fn get_chat_history(&self, chat_id: &str) -> Result<Vec<ChatEntry>> {
        let conn = self.pool.get_connection()?;
        
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
            "SELECT id, chat_id, model, question, response, timestamp, input_tokens, output_tokens
             FROM chat_logs
             WHERE chat_id = ?1
             ORDER BY timestamp ASC"
        )?;
        
        let rows = stmt.query_map([chat_id], |row| {
            Ok(ChatEntry {
                chat_id: row.get(1)?,
                model: row.get(2)?,
                question: row.get(3)?,
                response: row.get(4)?,
                timestamp: row.get(5)?,
                input_tokens: row.get(6).ok(),
                output_tokens: row.get(7).ok(),
            })
        })?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    // Optimized version with LIMIT for better performance on large datasets
    pub fn get_all_logs(&self) -> Result<Vec<ChatEntry>> {
        self.get_recent_logs(None)
    }
    
    pub fn get_recent_logs(&self, limit: Option<usize>) -> Result<Vec<ChatEntry>> {
        let conn = self.pool.get_connection()?;
        
        let sql = if let Some(limit) = limit {
            format!(
                "SELECT id, chat_id, model, question, response, timestamp, input_tokens, output_tokens
                 FROM chat_logs
                 ORDER BY timestamp DESC
                 LIMIT {}",
                limit
            )
        } else {
            "SELECT id, chat_id, model, question, response, timestamp, input_tokens, output_tokens
             FROM chat_logs
             ORDER BY timestamp DESC".to_string()
        };
        
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(conn.conn.as_ref().unwrap(), &sql)?;
        
        let rows = stmt.query_map([], |row| {
            Ok(ChatEntry {
                chat_id: row.get(1)?,
                model: row.get(2)?,
                question: row.get(3)?,
                response: row.get(4)?,
                timestamp: row.get(5)?,
                input_tokens: row.get(6).ok(),
                output_tokens: row.get(7).ok(),
            })
        })?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    pub fn set_current_session_id(&self, session_id: &str) -> Result<()> {
        let conn = self.pool.get_connection()?;
        
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
            "INSERT OR REPLACE INTO session_state (key, value) VALUES ('current_session', ?1)"
        )?;
        
        stmt.execute([session_id])?;
        Ok(())
    }
    
    pub fn get_current_session_id(&self) -> Result<Option<String>> {
        let conn = self.pool.get_connection()?;
        
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
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
        let conn = self.pool.get_connection()?;
        
        // Use transaction for atomic operation
        conn.execute("BEGIN TRANSACTION", [])?;
        
        match (|| -> Result<()> {
            conn.execute("DELETE FROM chat_logs", [])?;
            conn.execute("DELETE FROM session_state", [])?;
            Ok(())
        })() {
            Ok(_) => {
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", [])?;
                Err(e)
            }
        }
    }
    
    pub fn clear_session(&self, session_id: &str) -> Result<()> {
        let conn = self.pool.get_connection()?;
        
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
            "DELETE FROM chat_logs WHERE chat_id = ?1"
        )?;
        
        stmt.execute([session_id])?;
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.pool.get_connection()?;
        
        // Use single query with subqueries for better performance
        let total_entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chat_logs",
            [],
            |row| row.get(0)
        )?;
        
        let unique_sessions: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT chat_id) FROM chat_logs",
            [],
            |row| row.get(0)
        )?;
        
        // Get database file size
        let db_path = Self::database_path()?;
        let file_size = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // Get date range in single query
        let date_range = if total_entries > 0 {
            let (earliest, latest): (Option<DateTime<Utc>>, Option<DateTime<Utc>>) = conn.query_row(
                "SELECT MIN(timestamp), MAX(timestamp) FROM chat_logs",
                [],
                |row| Ok((row.get(0).ok(), row.get(1).ok()))
            )?;
            
            match (earliest, latest) {
                (Some(e), Some(l)) => Some((e, l)),
                _ => None,
            }
        } else {
            None
        };
        
        // Get model usage statistics with prepared statement
        let mut cache = self.statement_cache.lock().unwrap();
        let mut stmt = cache.get_or_prepare(
            conn.conn.as_ref().unwrap(),
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

// Thread-safe singleton for global database access

impl Database {
    #[allow(dead_code)]
    pub fn global() -> Result<Arc<Database>> {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<Result<Arc<Database>, String>> = OnceLock::new();
        
        INSTANCE.get_or_init(|| {
            Database::new()
                .map(Arc::new)
                .map_err(|e| e.to_string())
        }).clone().map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_connection_pool() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let pool = ConnectionPool::new(db_path, 3).unwrap();
        
        // Test getting multiple connections
        let conn1 = pool.get_connection().unwrap();
        let conn2 = pool.get_connection().unwrap();
        let conn3 = pool.get_connection().unwrap();
        
        // All connections should be valid
        assert!(conn1.query_row("SELECT 1", [], |_| Ok(())).is_ok());
        assert!(conn2.query_row("SELECT 1", [], |_| Ok(())).is_ok());
        assert!(conn3.query_row("SELECT 1", [], |_| Ok(())).is_ok());
    }
    
    #[test]
    fn test_optimized_database() {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        
        let db = Database::new().unwrap();
        
        // Test saving and retrieving
        db.save_chat_entry_with_tokens(
            "test_session",
            "test_model",
            "test question",
            "test response",
            Some(100),
            Some(50)
        ).unwrap();
        
        let history = db.get_chat_history("test_session").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].question, "test question");
        assert_eq!(history[0].input_tokens, Some(100));
        assert_eq!(history[0].output_tokens, Some(50));
    }
}