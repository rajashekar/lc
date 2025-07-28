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
    pub file_path: Option<String>,
    pub chunk_index: Option<i32>,
    pub total_chunks: Option<i32>,
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
                created_at TEXT NOT NULL,
                file_path TEXT,
                chunk_index INTEGER,
                total_chunks INTEGER
            )",
            [],
        )?;
        
        // Create index for faster similarity searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_provider ON vectors(model, provider)",
            [],
        )?;
        
        // Create index for file-based searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON vectors(file_path)",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn add_vector(&self, text: &str, vector: &[f64], model: &str, provider: &str) -> Result<i64> {
        self.add_vector_with_metadata(text, vector, model, provider, None, None, None)
    }
    
    pub fn add_vector_with_metadata(
        &self,
        text: &str,
        vector: &[f64],
        model: &str,
        provider: &str,
        file_path: Option<&str>,
        chunk_index: Option<i32>,
        total_chunks: Option<i32>
    ) -> Result<i64> {
        let conn = Connection::open(&self.db_path)?;
        
        // Serialize vector as JSON for storage
        let vector_json = serde_json::to_string(vector)?;
        let created_at = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO vectors (text, vector, model, provider, created_at, file_path, chunk_index, total_chunks) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![text, vector_json, model, provider, created_at, file_path, chunk_index, total_chunks],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub fn get_all_vectors(&self) -> Result<Vec<VectorEntry>> {
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, text, vector, model, provider, created_at, file_path, chunk_index, total_chunks FROM vectors ORDER BY created_at DESC"
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
                file_path: row.get(6).ok(),
                chunk_index: row.get(7).ok(),
                total_chunks: row.get(8).ok(),
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

// File processing utilities
pub struct FileProcessor;

impl FileProcessor {
    /// Check if a file is likely to be a text file based on extension and content
    pub fn is_text_file(path: &std::path::Path) -> bool {
        // Check extension first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            match ext.as_str() {
                // Text files
                "txt" | "md" | "markdown" | "rst" | "org" | "tex" | "rtf" => true,
                // Code files
                "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" |
                "go" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bash" |
                "zsh" | "fish" | "ps1" | "bat" | "cmd" | "html" | "css" | "scss" |
                "sass" | "less" | "xml" | "json" | "yaml" | "yml" | "toml" | "ini" |
                "cfg" | "conf" | "sql" | "r" | "m" | "mm" | "pl" | "pm" | "lua" |
                "vim" | "dockerfile" | "makefile" | "cmake" | "gradle" => true,
                // Log files
                "log" | "out" | "err" => true,
                // Binary files to exclude
                "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" |
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "pdf" | "doc" |
                "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "jpg" | "jpeg" | "png" |
                "gif" | "bmp" | "tiff" | "svg" | "ico" | "mp3" | "mp4" | "avi" |
                "mov" | "wmv" | "flv" | "mkv" | "wav" | "flac" | "ogg" => false,
                _ => {
                    // For unknown extensions, check if file has no extension (might be text)
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| !name.contains('.'))
                        .unwrap_or(false)
                }
            }
        } else {
            // No extension - might be a text file, check content
            Self::is_text_content(path).unwrap_or(false)
        }
    }
    
    /// Check if file content appears to be text by sampling first few bytes
    fn is_text_content(path: &std::path::Path) -> Result<bool> {
        use std::fs::File;
        use std::io::Read;
        
        let mut file = File::open(path)?;
        let mut buffer = [0; 512]; // Sample first 512 bytes
        let bytes_read = file.read(&mut buffer)?;
        
        if bytes_read == 0 {
            return Ok(true); // Empty file is considered text
        }
        
        // Check for null bytes (strong indicator of binary content)
        let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
        if null_count > 0 {
            return Ok(false);
        }
        
        // Check for high ratio of printable ASCII characters
        let printable_count = buffer[..bytes_read]
            .iter()
            .filter(|&&b| b >= 32 && b <= 126 || b == 9 || b == 10 || b == 13)
            .count();
        
        let printable_ratio = printable_count as f64 / bytes_read as f64;
        Ok(printable_ratio > 0.7) // At least 70% printable characters
    }
    
    /// Expand glob patterns and filter for text files
    pub fn expand_file_patterns(patterns: &[String]) -> Result<Vec<std::path::PathBuf>> {
        use glob::glob;
        
        let mut files = Vec::new();
        
        for pattern in patterns {
            crate::debug_log!("Processing file pattern: {}", pattern);
            
            match glob(pattern) {
                Ok(paths) => {
                    for path_result in paths {
                        match path_result {
                            Ok(path) => {
                                if path.is_file() && Self::is_text_file(&path) {
                                    crate::debug_log!("Adding text file: {}", path.display());
                                    files.push(path);
                                } else if path.is_file() {
                                    crate::debug_log!("Skipping non-text file: {}", path.display());
                                } else {
                                    crate::debug_log!("Skipping non-file: {}", path.display());
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Error processing path in pattern '{}': {}", pattern, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Invalid glob pattern '{}': {}", pattern, e);
                }
            }
        }
        
        files.sort();
        files.dedup();
        Ok(files)
    }
    
    /// Split text into chunks with overlap for better context preservation
    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        crate::debug_log!("Chunking text: {} chars, chunk_size: {}, overlap: {}", text.len(), chunk_size, overlap);
        
        if text.len() <= chunk_size {
            crate::debug_log!("Text is smaller than chunk size, returning single chunk");
            return vec![text.to_string()];
        }
        
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut iteration = 0;
        
        while start < text.len() {
            iteration += 1;
            crate::debug_log!("Chunk iteration {}: start={}, text.len()={}", iteration, start, text.len());
            
            let end = std::cmp::min(start + chunk_size, text.len());
            let mut chunk_end = end;
            
            // Try to break at sentence boundary
            if end < text.len() {
                if let Some(sentence_end) = text[start..end].rfind(". ") {
                    chunk_end = start + sentence_end + 1;
                } else if let Some(para_end) = text[start..end].rfind("\n\n") {
                    chunk_end = start + para_end + 1;
                } else if let Some(line_end) = text[start..end].rfind('\n') {
                    chunk_end = start + line_end + 1;
                }
            }
            
            let chunk = text[start..chunk_end].trim().to_string();
            if !chunk.is_empty() {
                let chunk_len = chunk.len();
                chunks.push(chunk);
                crate::debug_log!("Added chunk {}: {} chars", chunks.len(), chunk_len);
            }
            
            // Move start position with overlap
            if chunk_end >= text.len() {
                crate::debug_log!("Reached end of text, breaking");
                break;
            }
            
            let new_start = if chunk_end > overlap {
                chunk_end - overlap
            } else {
                chunk_end
            };
            
            // Ensure we're making progress - if new start is not greater than current start,
            // move forward by at least 1 character to prevent infinite loops
            if new_start <= start {
                start = start + 1;
                crate::debug_log!("Preventing infinite loop: moving start from {} to {}", new_start, start);
            } else {
                start = new_start;
            }
            
            crate::debug_log!("Next start position: {}", start);
            
            // Safety check to prevent infinite loop
            if iteration > 1000 {
                crate::debug_log!("WARNING: Too many iterations, breaking to prevent infinite loop");
                break;
            }
        }
        
        crate::debug_log!("Chunking complete: {} chunks created", chunks.len());
        chunks
    }
    
    /// Read and chunk a file
    pub fn process_file(path: &std::path::Path) -> Result<Vec<String>> {
        crate::debug_log!("Reading file: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        crate::debug_log!("File content length: {} characters", content.len());
        
        // Use 1200 character chunks with 200 character overlap
        crate::debug_log!("Starting text chunking with 1200 char chunks, 200 char overlap");
        let chunks = Self::chunk_text(&content, 1200, 200);
        
        crate::debug_log!("File '{}' split into {} chunks", path.display(), chunks.len());
        
        Ok(chunks)
    }
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
    
    #[test]
    fn test_chunk_text() {
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = FileProcessor::chunk_text(text, 30, 10);
        
        assert!(chunks.len() > 1);
        assert!(chunks[0].contains("sentence one"));
    }
    
    #[test]
    fn test_is_text_file() {
        use std::path::Path;
        
        assert!(FileProcessor::is_text_file(Path::new("test.txt")));
        assert!(FileProcessor::is_text_file(Path::new("test.rs")));
        assert!(FileProcessor::is_text_file(Path::new("test.py")));
        assert!(!FileProcessor::is_text_file(Path::new("test.exe")));
        assert!(!FileProcessor::is_text_file(Path::new("test.jpg")));
    }
}