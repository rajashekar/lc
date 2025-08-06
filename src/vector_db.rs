use anyhow::Result;
use dashmap::DashMap;
use hnsw_rs::prelude::*;
use parking_lot::RwLock;
use rayon::prelude::*;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// HNSW index for fast approximate nearest neighbor search
type HnswIndex = Hnsw<'static, f64, DistCosine>;

pub struct VectorDatabase {
    db_path: PathBuf,
    // In-memory HNSW index for fast similarity search
    hnsw_index: Arc<RwLock<Option<HnswIndex>>>,
    // Cache for vector entries to avoid repeated DB queries
    vector_cache: Arc<DashMap<i64, VectorEntry>>,
    // Track if index needs rebuilding
    index_dirty: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for VectorDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorDatabase")
            .field("db_path", &self.db_path)
            .field("vector_cache_len", &self.vector_cache.len())
            .field("index_dirty", &self.index_dirty)
            .finish()
    }
}

impl VectorDatabase {
    pub fn new(name: &str) -> Result<Self> {
        let embeddings_dir = Self::embeddings_dir()?;
        fs::create_dir_all(&embeddings_dir)?;

        let db_path = embeddings_dir.join(format!("{}.db", name));

        let db = Self {
            db_path,
            hnsw_index: Arc::new(RwLock::new(None)),
            vector_cache: Arc::new(DashMap::new()),
            index_dirty: Arc::new(RwLock::new(true)),
        };

        db.initialize()?;
        Ok(db)
    }

    pub fn embeddings_dir() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join("Library/Application Support/lc/embeddings"))
    }

    pub fn list_databases() -> Result<Vec<String>> {
        let embeddings_dir = Self::embeddings_dir()?;
        Self::list_databases_in_dir(&embeddings_dir)
    }

    pub fn list_databases_in_dir(embeddings_dir: &std::path::Path) -> Result<Vec<String>> {
        if !embeddings_dir.exists() {
            return Ok(Vec::new());
        }

        let mut databases = Vec::new();

        for entry in fs::read_dir(embeddings_dir)? {
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
        Self::delete_database_in_dir(name, &embeddings_dir)
    }

    pub fn delete_database_in_dir(name: &str, embeddings_dir: &std::path::Path) -> Result<()> {
        let db_path = embeddings_dir.join(format!("{}.db", name));

        if db_path.exists() {
            fs::remove_file(db_path)?;
        }

        Ok(())
    }

    fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        // First, create the table with the basic schema if it doesn't exist
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

        // Check if we need to migrate the schema by checking for missing columns
        let mut has_file_path = false;
        let mut has_chunk_index = false;
        let mut has_total_chunks = false;

        // Query the table schema to see what columns exist
        let mut stmt = conn.prepare("PRAGMA table_info(vectors)")?;
        let column_iter = stmt.query_map([], |row| {
            let column_name: String = row.get(1)?;
            Ok(column_name)
        })?;

        for column_result in column_iter {
            let column_name = column_result?;
            match column_name.as_str() {
                "file_path" => has_file_path = true,
                "chunk_index" => has_chunk_index = true,
                "total_chunks" => has_total_chunks = true,
                _ => {}
            }
        }

        // Add missing columns if they don't exist
        if !has_file_path {
            conn.execute("ALTER TABLE vectors ADD COLUMN file_path TEXT", [])?;
        }
        if !has_chunk_index {
            conn.execute("ALTER TABLE vectors ADD COLUMN chunk_index INTEGER", [])?;
        }
        if !has_total_chunks {
            conn.execute("ALTER TABLE vectors ADD COLUMN total_chunks INTEGER", [])?;
        }

        // Create index for faster similarity searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_provider ON vectors(model, provider)",
            [],
        )?;

        // Create index for file-based searches (only after ensuring the column exists)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON vectors(file_path)",
            [],
        )?;

        Ok(())
    }

    pub fn add_vector(
        &self,
        text: &str,
        vector: &[f64],
        model: &str,
        provider: &str,
    ) -> Result<i64> {
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
        total_chunks: Option<i32>,
    ) -> Result<i64> {
        let conn = Connection::open(&self.db_path)?;

        // Serialize vector as JSON for storage
        let vector_json = serde_json::to_string(vector)?;
        let created_at = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO vectors (text, vector, model, provider, created_at, file_path, chunk_index, total_chunks) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![text, vector_json, model, provider, created_at, file_path, chunk_index, total_chunks],
        )?;

        let id = conn.last_insert_rowid();

        // Create vector entry for cache
        let vector_entry = VectorEntry {
            id,
            text: text.to_string(),
            vector: vector.to_vec(),
            model: model.to_string(),
            provider: provider.to_string(),
            created_at: chrono::Utc::now(),
            file_path: file_path.map(|s| s.to_string()),
            chunk_index,
            total_chunks,
        };

        // Add to cache
        self.vector_cache.insert(id, vector_entry);

        // Mark index as dirty for rebuilding
        *self.index_dirty.write() = true;

        Ok(id)
    }

    pub fn get_all_vectors(&self) -> Result<Vec<VectorEntry>> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT id, text, vector, model, provider, created_at, file_path, chunk_index, total_chunks FROM vectors ORDER BY created_at DESC"
        )?;

        let vector_iter = stmt.query_map([], |row| {
            let vector_json: String = row.get(2)?;
            let vector: Vec<f64> = serde_json::from_str(&vector_json).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "vector".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            let created_at_str: String = row.get(5)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        5,
                        "created_at".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?
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

        let mut stmt = conn.prepare("SELECT model, provider FROM vectors LIMIT 1")?;

        let mut rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn find_similar(
        &self,
        query_vector: &[f64],
        limit: usize,
    ) -> Result<Vec<(VectorEntry, f64)>> {
        // Ensure HNSW index is built
        self.ensure_index_built()?;

        // Try to use HNSW index for fast approximate search
        if let Some(index) = self.hnsw_index.read().as_ref() {
            // Check dimension compatibility before using HNSW
            if !self.vector_cache.is_empty() {
                let first_entry = self.vector_cache.iter().next();
                if let Some(entry) = first_entry {
                    let stored_dimension = entry.vector.len();
                    if query_vector.len() != stored_dimension {
                        crate::debug_log!("Dimension mismatch: query={}, stored={}, falling back to linear search",
                                        query_vector.len(), stored_dimension);
                        return self.find_similar_linear_optimized(query_vector, limit);
                    }
                }
            }

            // Request more results from HNSW to account for potential cache misses
            let hnsw_limit = std::cmp::min(limit * 2, self.vector_cache.len());
            let search_results = index.search(query_vector, hnsw_limit, 50); // Higher ef for better recall

            let mut results = Vec::with_capacity(limit);
            for neighbor in search_results {
                if let Some(entry) = self.vector_cache.get(&(neighbor.d_id as i64)) {
                    // Convert distance to similarity (cosine distance -> cosine similarity)
                    let similarity = 1.0 - neighbor.distance as f64;
                    results.push((entry.value().clone(), similarity));

                    // Stop once we have enough results
                    if results.len() >= limit {
                        break;
                    }
                }
            }

            // If HNSW didn't return enough results, fall back to linear search
            if results.len() < limit && results.len() < self.vector_cache.len() {
                crate::debug_log!(
                    "HNSW returned only {} results, falling back to linear search",
                    results.len()
                );
                return self.find_similar_linear_optimized(query_vector, limit);
            }

            return Ok(results);
        }

        // Fallback to optimized linear search with parallel processing
        self.find_similar_linear_optimized(query_vector, limit)
    }

    /// Optimized linear search with parallel processing and SIMD
    fn find_similar_linear_optimized(
        &self,
        query_vector: &[f64],
        limit: usize,
    ) -> Result<Vec<(VectorEntry, f64)>> {
        // Get all vectors from cache or database
        let vectors = if self.vector_cache.is_empty() {
            self.get_all_vectors()?
        } else {
            self.vector_cache
                .iter()
                .map(|entry| entry.value().clone())
                .collect::<Vec<_>>()
        };

        // Use parallel processing for similarity calculations
        let mut similarities: Vec<(VectorEntry, f64)> = vectors
            .into_par_iter()
            .map(|vector_entry| {
                let similarity = cosine_similarity_simd(query_vector, &vector_entry.vector);
                (vector_entry, similarity)
            })
            .collect();

        // Use partial_sort for better performance when limit << total_vectors
        if limit < similarities.len() {
            similarities.select_nth_unstable_by(limit, |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            similarities[..limit]
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            similarities.truncate(limit);
        } else {
            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        Ok(similarities)
    }

    /// Ensure HNSW index is built and up-to-date
    fn ensure_index_built(&self) -> Result<()> {
        let index_dirty = *self.index_dirty.read();

        if index_dirty || self.hnsw_index.read().is_none() {
            self.rebuild_index()?;
        }

        Ok(())
    }

    /// Rebuild the HNSW index from all vectors
    fn rebuild_index(&self) -> Result<()> {
        crate::debug_log!("Rebuilding HNSW index...");

        // Load all vectors if cache is empty
        if self.vector_cache.is_empty() {
            let vectors = self.get_all_vectors()?;
            for vector in vectors {
                self.vector_cache.insert(vector.id, vector);
            }
        }

        if self.vector_cache.is_empty() {
            return Ok(());
        }

        // Get vector dimension from first entry
        let first_entry = self.vector_cache.iter().next();
        if let Some(entry) = first_entry {
            let dimension = entry.vector.len();

            // Create new HNSW index
            let hnsw = Hnsw::new(16, dimension, 200, 200, DistCosine {});

            // Add all vectors to index
            for entry in self.vector_cache.iter() {
                let vector_entry = entry.value();
                hnsw.insert((&vector_entry.vector, vector_entry.id as usize));
            }

            // Update the index
            *self.hnsw_index.write() = Some(hnsw);
            *self.index_dirty.write() = false;

            crate::debug_log!(
                "HNSW index rebuilt with {} vectors",
                self.vector_cache.len()
            );
        }

        Ok(())
    }

    pub fn count(&self) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |row| row.get(0))?;

        Ok(count as usize)
    }
}

// Optimized cosine similarity calculation with manual vectorization
pub fn cosine_similarity_simd(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        crate::debug_log!(
            "Vector dimension mismatch: query={}, stored={}",
            a.len(),
            b.len()
        );
        return 0.0;
    }

    if a.is_empty() {
        return 0.0;
    }

    // Use chunked processing for better cache performance
    let mut dot_product = 0.0f64;
    let mut norm_a_sq = 0.0f64;
    let mut norm_b_sq = 0.0f64;

    // Process in chunks of 4 for better performance
    let chunk_size = 4;
    let chunks = a.len() / chunk_size;

    for i in 0..chunks {
        let start = i * chunk_size;
        let end = start + chunk_size;

        for j in start..end {
            let av = a[j];
            let bv = b[j];
            dot_product += av * bv;
            norm_a_sq += av * av;
            norm_b_sq += bv * bv;
        }
    }

    // Process remaining elements
    for i in (chunks * chunk_size)..a.len() {
        let av = a[i];
        let bv = b[i];
        dot_product += av * bv;
        norm_a_sq += av * av;
        norm_b_sq += bv * bv;
    }

    let norm_a = norm_a_sq.sqrt();
    let norm_b = norm_b_sq.sqrt();

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
                "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" | "go" | "rb"
                | "php" | "swift" | "kt" | "scala" | "sh" | "bash" | "zsh" | "fish" | "ps1"
                | "bat" | "cmd" | "html" | "css" | "scss" | "sass" | "less" | "xml" | "json"
                | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "sql" | "r" | "m" | "mm"
                | "pl" | "pm" | "lua" | "vim" | "dockerfile" | "makefile" | "cmake" | "gradle" => {
                    true
                }
                // Log files
                "log" | "out" | "err" => true,
                // Binary files to exclude
                "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" | "zip"
                | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "pdf" | "doc" | "docx" | "xls"
                | "xlsx" | "ppt" | "pptx" | "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff"
                | "svg" | "ico" | "mp3" | "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "wav"
                | "flac" | "ogg" => false,
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
                                eprintln!(
                                    "Warning: Error processing path in pattern '{}': {}",
                                    pattern, e
                                );
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
        crate::debug_log!(
            "Chunking text: {} chars, chunk_size: {}, overlap: {}",
            text.len(),
            chunk_size,
            overlap
        );

        if text.len() <= chunk_size {
            crate::debug_log!("Text is smaller than chunk size, returning single chunk");
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut iteration = 0;

        while start < text.len() {
            iteration += 1;
            crate::debug_log!(
                "Chunk iteration {}: start={}, text.len()={}",
                iteration,
                start,
                text.len()
            );

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
                crate::debug_log!(
                    "Preventing infinite loop: moving start from {} to {}",
                    new_start,
                    start
                );
            } else {
                start = new_start;
            }

            crate::debug_log!("Next start position: {}", start);

            // Safety check to prevent infinite loop
            if iteration > 1000 {
                crate::debug_log!(
                    "WARNING: Too many iterations, breaking to prevent infinite loop"
                );
                break;
            }
        }

        crate::debug_log!("Chunking complete: {} chunks created", chunks.len());
        chunks
    }

    /// Read and chunk a file (synchronous version for compatibility)
    pub fn process_file(path: &std::path::Path) -> Result<Vec<String>> {
        // Try async version first, fall back to sync if no runtime available
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(Self::process_file_async(path))
        } else {
            // Fallback to synchronous implementation for tests and non-async contexts
            crate::debug_log!("Reading file synchronously: {}", path.display());
            let content = std::fs::read_to_string(path)?;
            crate::debug_log!("File content length: {} characters", content.len());

            // Use 1200 character chunks with 200 character overlap
            crate::debug_log!("Starting text chunking with 1200 char chunks, 200 char overlap");
            let chunks = Self::chunk_text(&content, 1200, 200);

            crate::debug_log!(
                "File '{}' split into {} chunks",
                path.display(),
                chunks.len()
            );

            Ok(chunks)
        }
    }

    /// Async version of process_file with memory mapping optimization
    pub async fn process_file_async(path: &std::path::Path) -> Result<Vec<String>> {
        crate::debug_log!("Reading file: {}", path.display());

        let content = Self::read_file_optimized(path).await?;
        crate::debug_log!("File content length: {} characters", content.len());

        // Use 1200 character chunks with 200 character overlap
        crate::debug_log!("Starting text chunking with 1200 char chunks, 200 char overlap");
        let chunks = Self::chunk_text(&content, 1200, 200);

        crate::debug_log!(
            "File '{}' split into {} chunks",
            path.display(),
            chunks.len()
        );

        Ok(chunks)
    }

    /// Optimized file reading with memory mapping for large files
    async fn read_file_optimized(path: &std::path::Path) -> Result<String> {
        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        // Use memory mapping for large files (>1MB)
        if file_size > 1_048_576 {
            crate::debug_log!("Using memory mapping for large file: {} bytes", file_size);

            let file = std::fs::File::open(path)?;
            let mmap = unsafe { memmap2::Mmap::map(&file)? };

            // Convert bytes to string in a separate task to avoid blocking
            let content = tokio::task::spawn_blocking(move || {
                std::str::from_utf8(&mmap)
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in file: {}", e))
                    .map(|s| s.to_string())
            })
            .await??;

            Ok(content)
        } else {
            // Use async file reading for smaller files
            crate::debug_log!(
                "Using async file reading for small file: {} bytes",
                file_size
            );
            Ok(tokio::fs::read_to_string(path).await?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity_simd(&a, &b) - 1.0).abs() < 1e-10);

        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity_simd(&a, &b) - 0.0).abs() < 1e-10);
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
