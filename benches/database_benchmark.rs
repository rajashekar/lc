use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lc::database::Database;
use tempfile::tempdir;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Benchmark for the original database implementation
fn benchmark_original_database(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("original_database_save_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                std::env::set_var("HOME", temp_dir.path());
                
                // Create new database instance each time (simulating original behavior)
                for i in 0..100 {
                    let db = Database::new().unwrap();
                    db.save_chat_entry_with_tokens(
                        &format!("session_{}", i),
                        "test_model",
                        &format!("question_{}", i),
                        &format!("response_{}", i),
                        Some(100),
                        Some(50),
                    ).unwrap();
                }
            });
        });
    });
}

// Benchmark for the optimized database implementation
fn benchmark_optimized_database(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("optimized_database_save_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                std::env::set_var("HOME", temp_dir.path());
                
                // Use single database instance with connection pooling
                let db = Database::new().unwrap();
                for i in 0..100 {
                    db.save_chat_entry_with_tokens(
                        &format!("session_{}", i),
                        "test_model",
                        &format!("question_{}", i),
                        &format!("response_{}", i),
                        Some(100),
                        Some(50),
                    ).unwrap();
                }
            });
        });
    });
}

// Benchmark query operations
fn benchmark_query_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("optimized_database_query_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                std::env::set_var("HOME", temp_dir.path());
                
                let db = Database::new().unwrap();
                
                // Pre-populate database
                for i in 0..100 {
                    db.save_chat_entry_with_tokens(
                        &format!("session_{}", i % 10), // 10 different sessions
                        "test_model",
                        &format!("question_{}", i),
                        &format!("response_{}", i),
                        Some(100),
                        Some(50),
                    ).unwrap();
                }
                
                // Benchmark queries
                for i in 0..10 {
                    let history = db.get_chat_history(&format!("session_{}", i)).unwrap();
                    black_box(history);
                }
            });
        });
    });
}

// Benchmark concurrent access
fn benchmark_concurrent_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("optimized_database_concurrent_50", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                std::env::set_var("HOME", temp_dir.path());
                
                let db = Arc::new(Database::new().unwrap());
                
                // Spawn multiple concurrent tasks
                let mut handles = Vec::new();
                for i in 0..50 {
                    let db_clone = Arc::clone(&db);
                    let handle = tokio::spawn(async move {
                        db_clone.save_chat_entry_with_tokens(
                            &format!("concurrent_session_{}", i),
                            "test_model",
                            &format!("concurrent_question_{}", i),
                            &format!("concurrent_response_{}", i),
                            Some(100),
                            Some(50),
                        ).unwrap();
                    });
                    handles.push(handle);
                }
                
                // Wait for all tasks to complete
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });
}

// Memory usage benchmark
fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("optimized_database_memory_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                std::env::set_var("HOME", temp_dir.path());
                
                let db = Database::new().unwrap();
                
                // Large batch operation to test memory efficiency
                for i in 0..1000 {
                    db.save_chat_entry_with_tokens(
                        &format!("memory_session_{}", i % 50), // 50 different sessions
                        "test_model",
                        &format!("Large question content that simulates real usage with substantial text content for testing memory efficiency and performance characteristics under load - iteration {}", i),
                        &format!("Large response content that simulates real LLM responses with substantial text content for testing memory efficiency and performance characteristics under load - iteration {}", i),
                        Some(500 + i as i32),
                        Some(300 + i as i32),
                    ).unwrap();
                }
                
                // Query operations to test memory usage during reads
                for i in 0..50 {
                    let history = db.get_chat_history(&format!("memory_session_{}", i)).unwrap();
                    black_box(history);
                }
                
                // Test stats operation
                let stats = db.get_stats().unwrap();
                black_box(stats);
            });
        });
    });
}

criterion_group!(
    benches,
    benchmark_original_database,
    benchmark_optimized_database,
    benchmark_query_operations,
    benchmark_concurrent_access,
    benchmark_memory_usage
);
criterion_main!(benches);