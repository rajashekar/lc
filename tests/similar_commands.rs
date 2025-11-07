//! Integration tests for similar commands
//!
//! This module contains comprehensive integration tests for all similarity search-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::config::Config;
use lc::vector_db::VectorDatabase;
use std::collections::HashMap;

#[cfg(test)]
mod similar_search_tests {
    use super::*;

    fn setup_test_database_with_vectors(test_name: &str) -> VectorDatabase {
        let db_name = format!("similar_test_{}_{}", test_name, std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add test vectors with known relationships
        let test_data = vec![
            // AI/ML related (should be similar to each other)
            (
                "Machine learning is a subset of artificial intelligence",
                vec![0.8, 0.6, 0.2, 0.1, 0.0],
            ),
            (
                "Deep learning uses neural networks with multiple layers",
                vec![0.7, 0.7, 0.3, 0.1, 0.0],
            ),
            (
                "Artificial intelligence enables computers to think",
                vec![0.9, 0.5, 0.1, 0.1, 0.0],
            ),
            // Programming related (different cluster)
            (
                "Python is a popular programming language",
                vec![0.1, 0.2, 0.8, 0.7, 0.1],
            ),
            (
                "JavaScript runs in web browsers",
                vec![0.0, 0.1, 0.9, 0.6, 0.2],
            ),
            // Completely different topic
            (
                "Cooking pasta requires boiling water",
                vec![0.0, 0.0, 0.1, 0.1, 0.9],
            ),
        ];

        for (text, vector) in test_data {
            db.add_vector(text, &vector, model, provider).unwrap();
        }

        db
    }

    #[test]
    fn test_basic_similarity_search() {
        let db = setup_test_database_with_vectors("basic");

        // Query with AI-related vector
        let ai_query = vec![0.85, 0.55, 0.15, 0.1, 0.0];
        let result = db.find_similar(&ai_query, 3);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 3);

        // Results should be ordered by similarity (highest first)
        assert!(similar[0].1 >= similar[1].1);
        assert!(similar[1].1 >= similar[2].1);

        // First result should be most similar to AI content
        assert!(
            similar[0].0.text.to_lowercase().contains("artificial")
                || similar[0].0.text.to_lowercase().contains("machine")
                || similar[0].0.text.to_lowercase().contains("deep")
        );
    }

    #[test]
    fn test_similarity_search_with_limit() {
        let db = setup_test_database_with_vectors("with_limit");

        let query = vec![0.5, 0.5, 0.5, 0.5, 0.5];

        // Test different limits
        for limit in 1..=6 {
            let result = db.find_similar(&query, limit);
            assert!(result.is_ok());

            let similar = result.unwrap();
            let expected_len = std::cmp::min(limit, 6); // We have 6 vectors total
            assert_eq!(similar.len(), expected_len);
        }
    }

    #[test]
    fn test_similarity_search_with_zero_limit() {
        let db = setup_test_database_with_vectors("zero_limit");

        let query = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let result = db.find_similar(&query, 0);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 0);
    }

    #[test]
    fn test_similarity_search_exact_match() {
        let db = setup_test_database_with_vectors("exact_match");

        // Use exact vector from our test data
        let exact_query = vec![0.8, 0.6, 0.2, 0.1, 0.0];
        let result = db.find_similar(&exact_query, 1);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 1);

        // Should have very high similarity (close to 1.0)
        assert!(similar[0].1 > 0.95);
        assert!(similar[0].0.text.contains("Machine learning"));
    }

    #[test]
    fn test_similarity_search_ordering() {
        let db = setup_test_database_with_vectors("ordering");

        let query = vec![0.8, 0.6, 0.2, 0.1, 0.0]; // AI-related query
        let result = db.find_similar(&query, 6);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 6);

        // Verify ordering (similarity scores should be descending)
        for i in 1..similar.len() {
            assert!(
                similar[i - 1].1 >= similar[i].1,
                "Similarity scores not in descending order: {} >= {}",
                similar[i - 1].1,
                similar[i].1
            );
        }

        // AI-related content should be at the top
        let top_results = &similar[0..3];
        let ai_keywords = [
            "machine",
            "learning",
            "artificial",
            "intelligence",
            "deep",
            "neural",
        ];

        for (entry, _) in top_results {
            let text_lower = entry.text.to_lowercase();
            let has_ai_keyword = ai_keywords
                .iter()
                .any(|&keyword| text_lower.contains(keyword));
            assert!(
                has_ai_keyword,
                "Top result should contain AI-related keywords: {}",
                entry.text
            );
        }
    }

    #[test]
    fn test_similarity_search_empty_database() {
        let db_name = format!("empty_similar_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let result = db.find_similar(&query, 5);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert!(similar.is_empty());

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similarity_search_single_vector() {
        let db_name = format!("single_similar_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add single vector
        let vector = vec![0.5, 0.5, 0.5];
        db.add_vector("Single vector", &vector, model, provider)
            .unwrap();

        // Search should return that single vector
        let query = vec![0.6, 0.4, 0.5];
        let result = db.find_similar(&query, 5);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].0.text, "Single vector");

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }
}

#[cfg(test)]
mod similar_model_resolution_tests {
    use super::*;

    fn create_test_config_for_similarity() -> Config {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("text-embedding-3-small".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Add OpenAI provider
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec![
                    "text-embedding-3-small".to_string(),
                    "text-embedding-3-large".to_string(),
                ],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        // Add Cohere provider
        config.providers.insert(
            "cohere".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.cohere.ai/v1".to_string(),
                api_key: Some("cohere-test-key".to_string()),
                models: vec!["embed-english-v3.0".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config
    }

    #[test]
    fn test_similar_model_resolution_from_database() {
        let config = create_test_config_for_similarity();

        // Create database with specific model
        let db_name = format!("model_resolution_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let stored_model = "text-embedding-3-large";
        let stored_provider = "openai";

        // Add vector to establish model info
        let vector = vec![0.1, 0.2, 0.3];
        db.add_vector("Test text", &vector, stored_model, stored_provider)
            .unwrap();

        // Get model info from database
        let model_info = db.get_model_info().unwrap();
        assert!(model_info.is_some());

        let (db_model, db_provider) = model_info.unwrap();
        assert_eq!(db_model, stored_model);
        assert_eq!(db_provider, stored_provider);

        // Test model resolution when no explicit model provided
        let result =
            lc::utils::resolve_model_and_provider(&config, Some(db_provider), Some(db_model));
        assert!(result.is_ok());

        let (resolved_provider, resolved_model) = result.unwrap();
        assert_eq!(resolved_provider, stored_provider);
        assert_eq!(resolved_model, stored_model);

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similar_model_override() {
        let config = create_test_config_for_similarity();

        // Create database with one model
        let db_name = format!("model_override_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        db.add_vector("Test", &[0.1, 0.2, 0.3], "text-embedding-3-small", "openai")
            .unwrap();

        // Test explicit model override
        let result = lc::utils::resolve_model_and_provider(
            &config,
            Some("cohere".to_string()),
            Some("embed-english-v3.0".to_string()),
        );
        assert!(result.is_ok());

        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similar_with_provider_model_format() {
        let config = create_test_config_for_similarity();

        // Test provider:model format
        let result = lc::utils::resolve_model_and_provider(
            &config,
            None,
            Some("cohere:embed-english-v3.0".to_string()),
        );
        assert!(result.is_ok());

        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }
}

#[cfg(test)]
mod similar_parameter_validation_tests {
    use super::*;

    #[test]
    fn test_similarity_limit_validation() {
        let db_name = format!("limit_validation_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add some test vectors
        for i in 0..5 {
            let vector = vec![i as f64, 0.0, 0.0];
            db.add_vector(&format!("Text {}", i), &vector, model, provider)
                .unwrap();
        }

        let query = vec![0.0, 0.0, 0.0];

        // Test various limit values
        let test_limits = vec![0, 1, 3, 5, 10, 100];

        for limit in test_limits {
            let result = db.find_similar(&query, limit);
            assert!(result.is_ok(), "Failed with limit: {}", limit);

            let similar = result.unwrap();
            let expected_len = std::cmp::min(limit, 5); // We have 5 vectors
            assert_eq!(
                similar.len(),
                expected_len,
                "Wrong result count for limit: {}",
                limit
            );
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similarity_query_vector_validation() {
        let db_name = format!("query_validation_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add test vector
        let stored_vector = vec![1.0, 0.0, 0.0];
        db.add_vector("Test vector", &stored_vector, model, provider)
            .unwrap();

        // Test various query vectors
        let test_queries = [
            vec![1.0, 0.0, 0.0],      // Same dimensions
            vec![0.5, 0.5, 0.5],      // Same dimensions, different values
            vec![1.0, 0.0],           // Different dimensions (fewer)
            vec![1.0, 0.0, 0.0, 0.0], // Different dimensions (more)
            vec![],                   // Empty vector
        ];

        for (i, query) in test_queries.iter().enumerate() {
            let result = db.find_similar(query, 1);
            // The behavior depends on implementation
            // Some might handle dimension mismatches, others might error
            match result {
                Ok(similar) => {
                    // If it succeeds, should return valid results
                    assert!(similar.len() <= 1, "Query {} returned too many results", i);
                }
                Err(_) => {
                    // Error is acceptable for invalid queries
                }
            }
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similarity_with_special_values() {
        let db_name = format!("special_values_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add normal vector
        let normal_vector = vec![0.5, 0.5, 0.5];
        db.add_vector("Normal vector", &normal_vector, model, provider)
            .unwrap();

        // Test queries with special values
        let special_queries = vec![
            vec![0.0, 0.0, 0.0],      // All zeros
            vec![1.0, 1.0, 1.0],      // All ones
            vec![-1.0, -1.0, -1.0],   // Negative values
            vec![f64::MAX, 0.0, 0.0], // Very large value
            vec![f64::MIN, 0.0, 0.0], // Very small value
        ];

        for query in special_queries {
            let result = db.find_similar(&query, 1);
            // Should handle special values gracefully
            match result {
                Ok(similar) => {
                    assert!(similar.len() <= 1);
                    if !similar.is_empty() {
                        // Similarity score should be finite
                        assert!(
                            similar[0].1.is_finite(),
                            "Similarity score should be finite for query: {:?}",
                            query
                        );
                    }
                }
                Err(_) => {
                    // Error handling is also acceptable
                }
            }
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }
}

#[cfg(test)]
mod similar_error_handling_tests {
    use super::*;

    #[test]
    fn test_similar_with_nonexistent_database() {
        // Try to search in a database that doesn't exist
        let db_name = format!("nonexistent_similar_db_{}", std::process::id());
        let result = VectorDatabase::new(&db_name);

        // This might succeed (creating the database) or fail
        match result {
            Ok(db) => {
                // If it creates the database, it should be empty
                let query = vec![1.0, 0.0, 0.0];
                let similar_result = db.find_similar(&query, 5);
                assert!(similar_result.is_ok());

                let similar = similar_result.unwrap();
                assert!(similar.is_empty());

                // Cleanup
                VectorDatabase::delete_database(&db_name).unwrap();
            }
            Err(_) => {
                // Error is also acceptable
            }
        }
    }

    #[test]
    fn test_similar_with_corrupted_data() {
        // This test would be implementation-specific
        // Testing how the system handles corrupted vector data
        let db_name = format!("corruption_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add some normal data
        db.add_vector("Normal text", &[0.1, 0.2, 0.3], model, provider)
            .unwrap();

        // Normal query should work
        let query = vec![0.1, 0.2, 0.3];
        let result = db.find_similar(&query, 1);
        assert!(result.is_ok());

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similar_with_invalid_model_info() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Test with empty config (no providers)
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_err());

        // Test with invalid provider
        let result = lc::utils::resolve_model_and_provider(
            &config,
            Some("invalid".to_string()),
            Some("model".to_string()),
        );
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod similar_performance_tests {
    use super::*;

    #[test]
    fn test_similarity_search_performance() {
        let db_name = format!("performance_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add many vectors
        let vector_count = 100;
        for i in 0..vector_count {
            let vector: Vec<f64> = (0..10).map(|j| (i * 10 + j) as f64 * 0.01).collect();
            db.add_vector(&format!("Vector {}", i), &vector, model, provider)
                .unwrap();
        }

        // Test search performance
        let query: Vec<f64> = (0..10).map(|i| i as f64 * 0.01).collect();

        let start = std::time::Instant::now();
        let result = db.find_similar(&query, 10);
        let duration = start.elapsed();

        assert!(result.is_ok());
        let similar = result.unwrap();
        assert_eq!(similar.len(), 10);

        // Performance should be reasonable (less than 1 second for 100 vectors)
        assert!(
            duration.as_secs() < 1,
            "Search took too long: {:?}",
            duration
        );

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similarity_search_with_large_vectors() {
        let db_name = format!("large_vector_performance_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();
        let model = "text-embedding-3-large";
        let provider = "openai";

        // Add vectors with realistic embedding dimensions
        let dimension = 1536; // OpenAI text-embedding-3-small dimension
        let vector_count = 10;

        for i in 0..vector_count {
            let vector: Vec<f64> = (0..dimension)
                .map(|j| ((i * dimension + j) as f64) * 0.0001)
                .collect();
            db.add_vector(&format!("Large vector {}", i), &vector, model, provider)
                .unwrap();
        }

        // Test search with large query vector
        let query: Vec<f64> = (0..dimension).map(|i| (i as f64) * 0.0001).collect();

        let result = db.find_similar(&query, 5);
        assert!(result.is_ok());

        let similar = result.unwrap();
        assert_eq!(similar.len(), 5);

        // Verify vector dimensions are preserved
        for (entry, _) in &similar {
            assert_eq!(entry.vector.len(), dimension);
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }
}

#[cfg(test)]
mod similar_integration_tests {
    use super::*;

    #[test]
    fn test_complete_similarity_workflow() {
        let _config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("text-embedding-3-small".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        let db_name = format!("similarity_workflow_test_{}", std::process::id());
        let model = "text-embedding-3-small";
        let provider = "openai";

        // 1. Create database and add vectors
        let db = VectorDatabase::new(&db_name).unwrap();

        let test_vectors = vec![
            (
                "Artificial intelligence research",
                vec![0.9, 0.1, 0.0, 0.0, 0.0],
            ),
            ("Machine learning algorithms", vec![0.8, 0.2, 0.0, 0.0, 0.0]),
            (
                "Web development with JavaScript",
                vec![0.0, 0.0, 0.9, 0.1, 0.0],
            ),
            ("Database design principles", vec![0.0, 0.0, 0.1, 0.9, 0.0]),
            ("Cooking Italian cuisine", vec![0.0, 0.0, 0.0, 0.0, 1.0]),
        ];

        for (text, vector) in &test_vectors {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        // 2. Test model resolution from database
        let model_info = db.get_model_info().unwrap().unwrap();
        assert_eq!(model_info.0, model);
        assert_eq!(model_info.1, provider);

        // 3. Test similarity search with AI-related query
        let ai_query = vec![0.85, 0.15, 0.0, 0.0, 0.0];
        let similar = db.find_similar(&ai_query, 3).unwrap();

        assert_eq!(similar.len(), 3);

        // Top results should be AI-related
        assert!(
            similar[0].0.text.to_lowercase().contains("artificial")
                || similar[0].0.text.to_lowercase().contains("intelligence")
        );
        assert!(
            similar[1].0.text.to_lowercase().contains("machine")
                || similar[1].0.text.to_lowercase().contains("learning")
        );

        // 4. Test similarity search with different query
        let web_query = vec![0.0, 0.0, 0.8, 0.2, 0.0];
        let web_similar = db.find_similar(&web_query, 2).unwrap();

        assert_eq!(web_similar.len(), 2);
        assert!(
            web_similar[0].0.text.to_lowercase().contains("web")
                || web_similar[0].0.text.to_lowercase().contains("javascript")
        );

        // 5. Test with limit larger than available vectors
        let all_similar = db.find_similar(&ai_query, 10).unwrap();
        assert_eq!(all_similar.len(), 5); // Should return all 5 vectors

        // 6. Verify similarity ordering
        for i in 1..all_similar.len() {
            assert!(all_similar[i - 1].1 >= all_similar[i].1);
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }

    #[test]
    fn test_similarity_with_model_consistency() {
        let db_name = format!("model_consistency_test_{}", std::process::id());
        let db = VectorDatabase::new(&db_name).unwrap();

        // Add vectors with consistent model
        let model = "text-embedding-3-small";
        let provider = "openai";

        db.add_vector("First text", &[0.1, 0.2, 0.3], model, provider)
            .unwrap();
        db.add_vector("Second text", &[0.2, 0.3, 0.4], model, provider)
            .unwrap();

        // Verify model consistency
        let model_info = db.get_model_info().unwrap().unwrap();
        assert_eq!(model_info.0, model);
        assert_eq!(model_info.1, provider);

        // Search should work with consistent dimensions
        let query = vec![0.15, 0.25, 0.35];
        let similar = db.find_similar(&query, 2).unwrap();
        assert_eq!(similar.len(), 2);

        // Both results should have the same model info
        for (entry, _) in &similar {
            assert_eq!(entry.model, model);
            assert_eq!(entry.provider, provider);
        }

        // Cleanup
        VectorDatabase::delete_database(&db_name).unwrap();
    }
}
