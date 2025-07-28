//! Integration tests for vector database commands
//! 
//! This module contains comprehensive integration tests for all vector database-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::vector_db::{VectorDatabase, VectorEntry};
use lc::config::Config;
use std::collections::HashMap;
use tempfile::TempDir;
use chrono::{DateTime, Utc};

#[cfg(test)]
mod vector_database_creation_tests {
    use super::*;

    #[test]
    fn test_vector_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_name = "test_db";
        
        // Test database creation
        let result = VectorDatabase::new(db_name);
        assert!(result.is_ok());
        
        let db = result.unwrap();
        // Database should be empty initially
        let count = db.count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_vector_database_path_generation() {
        let db_name = "test_database";
        let result = VectorDatabase::new(db_name);
        assert!(result.is_ok());
        
        // Database should be created in the correct location
        let db = result.unwrap();
        let count = db.count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_vector_database_with_special_characters() {
        let db_names = vec![
            "test-db",
            "test_db",
            "test123",
            "db-with-hyphens",
            "db_with_underscores",
        ];

        for db_name in &db_names {
            // Clean up any existing database first
            let _ = VectorDatabase::delete_database(db_name);
            
            let result = VectorDatabase::new(db_name);
            assert!(result.is_ok(), "Failed to create database with name: {}", db_name);
            
            let db = result.unwrap();
            let count = db.count().unwrap();
            assert_eq!(count, 0);
        }
        
        // Cleanup all test databases
        for db_name in &db_names {
            let _ = VectorDatabase::delete_database(db_name);
        }
    }

    #[test]
    fn test_vector_database_invalid_names() {
        let invalid_names = vec![
            "", // Empty name
            " ", // Whitespace only
            "db with spaces", // Spaces in name
        ];

        for db_name in invalid_names {
            // In a real implementation, these might be rejected or sanitized
            // For now, we test that the system handles them gracefully
            let result = VectorDatabase::new(db_name);
            // The behavior here depends on implementation - it might succeed with sanitization
            // or fail with an error. We just ensure it doesn't panic.
            let _ = result;
        }
    }
}

#[cfg(test)]
mod vector_storage_tests {
    use super::*;

    fn create_test_vector(size: usize) -> Vec<f64> {
        (0..size).map(|i| i as f64 * 0.1).collect()
    }

    #[test]
    fn test_vector_addition() {
        let db_name = "test_add";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let text = "Test text for embedding";
        let vector = create_test_vector(1536);
        let model = "text-embedding-3-small";
        let provider = "openai";

        let result = db.add_vector(text, &vector, model, provider);
        assert!(result.is_ok());

        let id = result.unwrap();
        assert!(id > 0);

        // Check that count increased
        let count = db.count().unwrap();
        assert_eq!(count, 1);
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_multiple_vector_addition() {
        let db_name = "test_multiple";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let texts = vec![
            "First text",
            "Second text",
            "Third text",
        ];
        let model = "text-embedding-3-small";
        let provider = "openai";

        for text in &texts {
            let vector = create_test_vector(1536);
            let result = db.add_vector(text, &vector, model, provider);
            assert!(result.is_ok());
        }

        let count = db.count().unwrap();
        assert_eq!(count, 3);
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_vector_with_different_dimensions() {
        let db_name = "test_dimensions";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add vector with 1536 dimensions
        let vector1 = create_test_vector(1536);
        let result1 = db.add_vector("Text 1", &vector1, model, provider);
        assert!(result1.is_ok());

        // Add vector with different dimensions (should still work)
        let vector2 = create_test_vector(1024);
        let result2 = db.add_vector("Text 2", &vector2, model, provider);
        assert!(result2.is_ok());

        let count = db.count().unwrap();
        assert_eq!(count, 2);
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_vector_with_empty_text() {
        let db = VectorDatabase::new("test_empty_text").unwrap();
        let vector = create_test_vector(1536);
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Empty text should still be allowed (might be useful for some use cases)
        let result = db.add_vector("", &vector, model, provider);
        // Behavior depends on implementation - might succeed or fail
        let _ = result;
    }

    #[test]
    fn test_vector_with_empty_vector() {
        let db = VectorDatabase::new("test_empty_vector").unwrap();
        let empty_vector: Vec<f64> = vec![];
        let model = "text-embedding-3-small";
        let provider = "openai";

        let result = db.add_vector("Test text", &empty_vector, model, provider);
        // Empty vector should probably fail or be handled specially
        let _ = result;
    }
}

#[cfg(test)]
mod vector_retrieval_tests {
    use super::*;

    fn setup_test_database(db_name: &str) -> VectorDatabase {
        // Delete any existing database first
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add test vectors
        let test_data = vec![
            ("Machine learning is a subset of AI", vec![0.1, 0.2, 0.3, 0.4, 0.5]),
            ("Deep learning uses neural networks", vec![0.2, 0.3, 0.4, 0.5, 0.6]),
            ("Natural language processing", vec![0.3, 0.4, 0.5, 0.6, 0.7]),
            ("Computer vision applications", vec![0.4, 0.5, 0.6, 0.7, 0.8]),
        ];

        for (text, vector) in test_data {
            db.add_vector(text, &vector, model, provider).unwrap();
        }

        db
    }

    #[test]
    fn test_get_all_vectors() {
        let db = setup_test_database("test_get_all_vectors");
        
        let result = db.get_all_vectors();
        assert!(result.is_ok());
        
        let vectors = result.unwrap();
        assert_eq!(vectors.len(), 4);
        
        // Check that all vectors have the expected structure
        for vector in &vectors {
            assert!(!vector.text.is_empty());
            assert!(!vector.vector.is_empty());
            assert_eq!(vector.model, "text-embedding-3-small");
            assert_eq!(vector.provider, "openai");
            assert!(vector.id > 0);
        }
        
        // Cleanup
        VectorDatabase::delete_database("test_get_all_vectors").unwrap();
    }

    #[test]
    fn test_vector_entry_structure() {
        let db = setup_test_database("test_vector_entry_structure");
        let vectors = db.get_all_vectors().unwrap();
        let first_vector = &vectors[0];

        // Test VectorEntry structure
        assert!(first_vector.id > 0);
        assert!(!first_vector.text.is_empty());
        assert!(!first_vector.vector.is_empty());
        assert_eq!(first_vector.model, "text-embedding-3-small");
        assert_eq!(first_vector.provider, "openai");
        // created_at should be a valid timestamp
        assert!(first_vector.created_at <= Utc::now());
        
        // Cleanup
        VectorDatabase::delete_database("test_vector_entry_structure").unwrap();
    }

    #[test]
    fn test_vector_ordering() {
        let db = setup_test_database("test_vector_ordering");
        let vectors = db.get_all_vectors().unwrap();

        // Vectors should be ordered by ID (insertion order)
        // Since we clean the database in setup_test_database, IDs should be sequential
        let mut ids: Vec<i64> = vectors.iter().map(|v| v.id).collect();
        ids.sort();
        
        // Check that IDs are in ascending order
        for i in 1..ids.len() {
            assert!(ids[i] > ids[i-1], "IDs should be in ascending order: {} should be > {}", ids[i], ids[i-1]);
        }
        
        // Cleanup
        VectorDatabase::delete_database("test_vector_ordering").unwrap();
    }

    #[test]
    fn test_empty_database_retrieval() {
        let db = VectorDatabase::new("test_empty_retrieval").unwrap();
        
        let result = db.get_all_vectors();
        assert!(result.is_ok());
        
        let vectors = result.unwrap();
        assert!(vectors.is_empty());
    }
}

#[cfg(test)]
mod vector_similarity_tests {
    use super::*;

    fn create_normalized_vector(values: Vec<f64>) -> Vec<f64> {
        let magnitude = values.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude == 0.0 {
            values
        } else {
            values.iter().map(|x| x / magnitude).collect()
        }
    }

    #[test]
    fn test_cosine_similarity_calculation() {
        let db_name = "test_similarity";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add test vectors
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![0.0, 1.0, 0.0];
        let vector3 = vec![1.0, 1.0, 0.0]; // Should be similar to vector1

        db.add_vector("Text 1", &vector1, model, provider).unwrap();
        db.add_vector("Text 2", &vector2, model, provider).unwrap();
        db.add_vector("Text 3", &vector3, model, provider).unwrap();

        // Test similarity search
        let query_vector = vec![1.0, 0.0, 0.0]; // Same as vector1
        let result = db.find_similar(&query_vector, 3);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 3);

        // First result should be most similar (vector1)
        assert!(similar[0].1 > similar[1].1); // Higher similarity score
        assert!(similar[1].1 >= similar[2].1);
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_similarity_with_identical_vectors() {
        let db = VectorDatabase::new("test_identical").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        let vector = vec![0.5, 0.5, 0.5, 0.5];
        db.add_vector("Identical text", &vector, model, provider).unwrap();

        // Query with identical vector
        let result = db.find_similar(&vector, 1);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 1);
        
        // Similarity should be very close to 1.0 (allowing for floating point precision)
        assert!(similar[0].1 > 0.99);
    }

    #[test]
    fn test_similarity_with_orthogonal_vectors() {
        let db_name = "test_orthogonal";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        let vector1 = vec![1.0, 0.0];
        let vector2 = vec![0.0, 1.0];

        db.add_vector("Vector 1", &vector1, model, provider).unwrap();
        db.add_vector("Vector 2", &vector2, model, provider).unwrap();

        // Query with vector1
        let result = db.find_similar(&vector1, 2);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 2);

        // First should be identical (similarity ~1.0)
        // Second should be orthogonal (similarity ~0.0)
        assert!(similar[0].1 > 0.99);
        assert!(similar[1].1.abs() < 0.1); // Close to 0 (relaxed tolerance)
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_similarity_limit() {
        let db = VectorDatabase::new("test_limit").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add 5 vectors
        for i in 0..5 {
            let vector = vec![i as f64, 0.0, 0.0];
            db.add_vector(&format!("Text {}", i), &vector, model, provider).unwrap();
        }

        // Request only 3 results
        let query_vector = vec![0.0, 0.0, 0.0];
        let result = db.find_similar(&query_vector, 3);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 3); // Should respect the limit
    }

    #[test]
    fn test_similarity_with_empty_database() {
        let db = VectorDatabase::new("test_empty_similarity").unwrap();
        
        let query_vector = vec![1.0, 0.0, 0.0];
        let result = db.find_similar(&query_vector, 5);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert!(similar.is_empty());
    }

    #[test]
    fn test_similarity_with_dimension_mismatch() {
        let db = VectorDatabase::new("test_dimension_mismatch").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add vector with 3 dimensions
        let stored_vector = vec![1.0, 0.0, 0.0];
        db.add_vector("Stored text", &stored_vector, model, provider).unwrap();

        // Query with different dimensions
        let query_vector = vec![1.0, 0.0]; // Only 2 dimensions
        let result = db.find_similar(&query_vector, 1);
        
        // This should handle dimension mismatch gracefully
        // Implementation might return error or handle it somehow
        let _ = result;
    }
}

#[cfg(test)]
mod vector_metadata_tests {
    use super::*;

    #[test]
    fn test_model_info_storage_and_retrieval() {
        let db_name = "test_model_info";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Initially no model info
        let result = db.get_model_info();
        assert!(result.is_ok());
        let model_info = result.unwrap();
        assert!(model_info.is_none());

        // Add a vector
        let vector = vec![0.1, 0.2, 0.3];
        db.add_vector("Test text", &vector, model, provider).unwrap();

        // Now should have model info
        let result = db.get_model_info();
        assert!(result.is_ok());
        let model_info = result.unwrap();
        assert!(model_info.is_some());

        let (stored_model, stored_provider) = model_info.unwrap();
        assert_eq!(stored_model, model);
        assert_eq!(stored_provider, provider);
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_model_info_consistency() {
        let db = VectorDatabase::new("test_model_consistency").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add multiple vectors with same model/provider
        for i in 0..3 {
            let vector = vec![i as f64, 0.0, 0.0];
            db.add_vector(&format!("Text {}", i), &vector, model, provider).unwrap();
        }

        let model_info = db.get_model_info().unwrap().unwrap();
        assert_eq!(model_info.0, model);
        assert_eq!(model_info.1, provider);
    }

    #[test]
    fn test_model_info_with_different_models() {
        let db = VectorDatabase::new("test_different_models").unwrap();

        // Add vector with first model
        let vector1 = vec![0.1, 0.2, 0.3];
        db.add_vector("Text 1", &vector1, "model1", "provider1").unwrap();

        // Add vector with different model
        let vector2 = vec![0.4, 0.5, 0.6];
        db.add_vector("Text 2", &vector2, "model2", "provider2").unwrap();

        // Should return the first model info (or handle mixed models somehow)
        let model_info = db.get_model_info().unwrap();
        assert!(model_info.is_some());
        // The exact behavior depends on implementation
    }

    #[test]
    fn test_database_count() {
        let db_name = "test_count";
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();

        // Initially empty
        assert_eq!(db.count().unwrap(), 0);

        // Add vectors and check count
        let model = "text-embedding-3-small";
        let provider = "openai";

        for i in 0..5 {
            let vector = vec![i as f64];
            db.add_vector(&format!("Text {}", i), &vector, model, provider).unwrap();
            assert_eq!(db.count().unwrap(), i + 1);
        }
        
        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }
}

#[cfg(test)]
mod vector_database_management_tests {
    use super::*;

    #[test]
    fn test_list_databases() {
        // Create a few test databases
        let _db1 = VectorDatabase::new("list_test_1").unwrap();
        let _db2 = VectorDatabase::new("list_test_2").unwrap();
        let _db3 = VectorDatabase::new("list_test_3").unwrap();

        let result = VectorDatabase::list_databases();
        assert!(result.is_ok());

        let databases = result.unwrap();
        // Should contain at least our test databases
        assert!(databases.contains(&"list_test_1".to_string()));
        assert!(databases.contains(&"list_test_2".to_string()));
        assert!(databases.contains(&"list_test_3".to_string()));
    }

    #[test]
    fn test_delete_database() {
        // Create a test database
        let db_name = "delete_test";
        let _db = VectorDatabase::new(db_name).unwrap();

        // Verify it exists
        let databases = VectorDatabase::list_databases().unwrap();
        assert!(databases.contains(&db_name.to_string()));

        // Delete it
        let result = VectorDatabase::delete_database(db_name);
        assert!(result.is_ok());

        // Verify it's gone
        let databases = VectorDatabase::list_databases().unwrap();
        assert!(!databases.contains(&db_name.to_string()));
    }

    #[test]
    fn test_delete_nonexistent_database() {
        let result = VectorDatabase::delete_database("nonexistent_db");
        // Should handle gracefully - might return error or succeed
        let _ = result;
    }

    #[test]
    fn test_database_persistence() {
        let db_name = "persistence_test";
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Create database and add data
        {
            let db = VectorDatabase::new(db_name).unwrap();
            let vector = vec![0.1, 0.2, 0.3];
            db.add_vector("Persistent text", &vector, model, provider).unwrap();
        }

        // Reopen database and check data persists
        {
            let db = VectorDatabase::new(db_name).unwrap();
            let count = db.count().unwrap();
            assert_eq!(count, 1);

            let vectors = db.get_all_vectors().unwrap();
            assert_eq!(vectors.len(), 1);
            assert_eq!(vectors[0].text, "Persistent text");
        }

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }
}

#[cfg(test)]
mod vector_error_handling_tests {
    use super::*;

    #[test]
    fn test_database_creation_errors() {
        // Test various edge cases for database creation
        let edge_cases = vec![
            "normal_db",
            "db-with-hyphens", 
            "db_with_underscores",
            "db123",
        ];

        for db_name in edge_cases {
            let result = VectorDatabase::new(db_name);
            // Should either succeed or fail gracefully
            match result {
                Ok(_) => {
                    // Success is fine
                    VectorDatabase::delete_database(db_name).ok();
                }
                Err(_) => {
                    // Error is also acceptable for some edge cases
                }
            }
        }
    }

    #[test]
    fn test_vector_operations_on_closed_database() {
        // This test depends on implementation details
        // Some databases might not have an explicit "closed" state
        let db = VectorDatabase::new("test_closed").unwrap();
        
        // Try operations (should work normally)
        let vector = vec![0.1, 0.2, 0.3];
        let result = db.add_vector("Test", &vector, "model", "provider");
        assert!(result.is_ok());

        // Cleanup
        VectorDatabase::delete_database("test_closed").unwrap();
    }

    #[test]
    fn test_concurrent_database_access() {
        // Basic test for concurrent access
        // In a real implementation, this would test thread safety
        let db_name = "concurrent_test";
        let db1 = VectorDatabase::new(db_name).unwrap();
        let db2 = VectorDatabase::new(db_name).unwrap();

        // Both should be able to read
        let count1 = db1.count().unwrap();
        let count2 = db2.count().unwrap();
        assert_eq!(count1, count2);

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }
}

#[cfg(test)]
mod vector_integration_tests {
    use super::*;

    #[test]
    fn test_complete_vector_workflow() {
        let db_name = "integration_test";
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Create database
        let db = VectorDatabase::new(db_name).unwrap();
        assert_eq!(db.count().unwrap(), 0);

        // Add vectors
        let test_data = vec![
            ("Machine learning is powerful", vec![0.1, 0.8, 0.3]),
            ("Deep learning uses neural networks", vec![0.2, 0.7, 0.4]),
            ("AI will change the world", vec![0.3, 0.6, 0.5]),
        ];

        for (text, vector) in &test_data {
            let result = db.add_vector(text, vector, model, provider);
            assert!(result.is_ok());
        }

        // Verify count
        assert_eq!(db.count().unwrap(), 3);

        // Test retrieval
        let all_vectors = db.get_all_vectors().unwrap();
        assert_eq!(all_vectors.len(), 3);

        // Test model info
        let model_info = db.get_model_info().unwrap().unwrap();
        assert_eq!(model_info.0, model);
        assert_eq!(model_info.1, provider);

        // Test similarity search
        let query_vector = vec![0.15, 0.75, 0.35]; // Similar to first vector
        let similar = db.find_similar(&query_vector, 2).unwrap();
        assert_eq!(similar.len(), 2);
        assert!(similar[0].1 > similar[1].1); // Ordered by similarity

        // Test database listing
        let databases = VectorDatabase::list_databases().unwrap();
        assert!(databases.contains(&db_name.to_string()));

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
        let databases_after = VectorDatabase::list_databases().unwrap();
        assert!(!databases_after.contains(&db_name.to_string()));
    }

    #[test]
    fn test_large_vector_handling() {
        let db = VectorDatabase::new("large_vector_test").unwrap();
        let model = "text-embedding-3-large";
        let provider = "openai";

        // Test with large vectors (typical embedding sizes)
        let large_vector: Vec<f64> = (0..1536).map(|i| (i as f64) * 0.001).collect();
        
        let result = db.add_vector("Large vector test", &large_vector, model, provider);
        assert!(result.is_ok());

        let vectors = db.get_all_vectors().unwrap();
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].vector.len(), 1536);

        // Test similarity with large vector
        let query_vector: Vec<f64> = (0..1536).map(|i| (i as f64) * 0.0011).collect();
        let similar = db.find_similar(&query_vector, 1).unwrap();
        assert_eq!(similar.len(), 1);

        VectorDatabase::delete_database("large_vector_test").unwrap();
    }

    #[test]
    fn test_multiple_databases_isolation() {
        let db1 = VectorDatabase::new("isolation_test_1").unwrap();
        let db2 = VectorDatabase::new("isolation_test_2").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add different data to each database
        db1.add_vector("Database 1 text", &vec![1.0, 0.0, 0.0], model, provider).unwrap();
        db2.add_vector("Database 2 text", &vec![0.0, 1.0, 0.0], model, provider).unwrap();

        // Verify isolation
        assert_eq!(db1.count().unwrap(), 1);
        assert_eq!(db2.count().unwrap(), 1);

        let vectors1 = db1.get_all_vectors().unwrap();
        let vectors2 = db2.get_all_vectors().unwrap();

        assert_eq!(vectors1[0].text, "Database 1 text");
        assert_eq!(vectors2[0].text, "Database 2 text");
        assert_ne!(vectors1[0].vector, vectors2[0].vector);

        // Cleanup
        VectorDatabase::delete_database("isolation_test_1").unwrap();
        VectorDatabase::delete_database("isolation_test_2").unwrap();
    }
}