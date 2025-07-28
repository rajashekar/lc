use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: i64,
    pub text: String,
    pub vector: Vec<f64>,
    pub model: String,
    pub provider: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct VectorDatabase {
    db_path: PathBuf,
    name: String,
}

impl VectorDatabase {
    pub fn new(name: &str) -> Result<Self> {
        let embeddings_dir = Self::embeddings_dir()?;
        fs::create_dir_all(&embeddings_dir)?;
        
        let db_path = embeddings_dir.join(format!("{}.db", name));
        
        let db = Self {
            db_path,
            name: name.to_string(),
        };
        
        db.initialize()?;
        Ok(db)
    }
    
    pub fn embeddings_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join("Library/Application Support/lc/embeddings"))
    }
    
    pub fn list_databases() -> Result<Vec<String>> {
        let embeddings_dir = Self::embeddings_dir()?;
        
        if !embeddings_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut databases = Vec::new();
        
        for entry in fs::read_dir(&embeddings_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "db" {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            databases.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        databases.sort();
        Ok(databases)
    }
    
    pub fn delete_database(name: &str) -> Result<()> {
        let embeddings_dir = Self::embeddings_dir()?;
        let db_path = embeddings_dir.join(format!("{}.db", name));
        
        if db_path.exists() {
            fs::remove_file(db_path)?;
        }
        
        Ok(())
    }
    
    fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                vector BLOB NOT NULL,
                model TEXT NOT NULL,
                provider TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index for faster similarity searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_provider ON vectors(model, provider)",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn add_vector(&self, text: &str, vector: &[f64], model: &str, provider: &str) -> Result<i64> {
        let conn = Connection::open(&self.db_path)?;
        
        // Serialize vector as JSON for storage
        let vector_json = serde_json::to_string(vector)?;
        let created_at = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO vectors (text, vector, model, provider, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![text, vector_json, model, provider, created_at],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub fn get_all_vectors(&self) -> Result<Vec<VectorEntry>> {
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, text, vector, model, provider, created_at FROM vectors ORDER BY created_at DESC"
        )?;
        
        let vector_iter = stmt.query_map([], |row| {
            let vector_json: String = row.get(2)?;
            let vector: Vec<f64> = serde_json::from_str(&vector_json)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "vector".to_string(), rusqlite::types::Type::Text))?;
            
            let created_at_str: String = row.get(5)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&chrono::Utc);
            
            Ok(VectorEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                vector,
                model: row.get(3)?,
                provider: row.get(4)?,
                created_at,
            })
        })?;
        
        let mut vectors = Vec::new();
        for vector in vector_iter {
            vectors.push(vector?);
        }
        
        Ok(vectors)
    }
    
    pub fn get_model_info(&self) -> Result<Option<(String, String)>> {
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT model, provider FROM vectors LIMIT 1"
        )?;
        
        let mut rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    pub fn find_similar(&self, query_vector: &[f64], limit: usize) -> Result<Vec<(VectorEntry, f64)>> {
        let vectors = self.get_all_vectors()?;
        
        let mut similarities = Vec::new();
        
        for vector_entry in vectors {
            let similarity = cosine_similarity(query_vector, &vector_entry.vector);
            similarities.push((vector_entry, similarity));
        }
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top results
        similarities.truncate(limit);
        
        Ok(similarities)
    }
    
    pub fn count(&self) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vectors",
            [],
            |row| row.get(0)
        )?;
        
        Ok(count as usize)
    }
}

// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        crate::debug_log!("Vector dimension mismatch: query={}, stored={}", a.len(), b.len());
        return 0.0;
    }
    
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);
        
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-10);
    }
}