//! Integration tests for RAG (Retrieval-Augmented Generation) commands
//! 
//! This module contains comprehensive integration tests for all RAG-related
//! CLI functionality, testing both direct prompt and interactive chat RAG features.

mod common;

use lc::vector_db::VectorDatabase;
use lc::config::Config;
use lc::provider::EmbeddingRequest;
use std::collections::HashMap;

#[cfg(test)]
mod rag_context_retrieval_tests {
    use super::*;

    fn setup_rag_test_database(db_name: &str) -> VectorDatabase {
        // Clean up any existing database first
        let _ = VectorDatabase::delete_database(db_name);
        
        let db = VectorDatabase::new(db_name).unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add knowledge base content
        let knowledge_base = vec![
            ("Machine learning is a subset of artificial intelligence that enables computers to learn and improve from experience without being explicitly programmed.",
             vec![0.8, 0.6, 0.2, 0.1, 0.0, 0.1]),
            ("Deep learning is a subset of machine learning that uses neural networks with multiple layers to model and understand complex patterns in data.",
             vec![0.7, 0.7, 0.3, 0.1, 0.0, 0.1]),
            ("Neural networks are computing systems inspired by biological neural networks. They consist of interconnected nodes that process information.",
             vec![0.6, 0.8, 0.4, 0.1, 0.0, 0.1]),
            ("Natural language processing (NLP) is a branch of AI that helps computers understand, interpret and manipulate human language.",
             vec![0.5, 0.4, 0.8, 0.2, 0.0, 0.1]),
            ("Computer vision is a field of AI that trains computers to interpret and understand visual information from the world.",
             vec![0.4, 0.3, 0.2, 0.8, 0.2, 0.1]),
            ("Python is a high-level programming language known for its simplicity and readability. It's widely used in data science and AI.",
             vec![0.2, 0.1, 0.3, 0.1, 0.8, 0.5]),
        ];

        for (text, vector) in knowledge_base {
            db.add_vector(text, &vector, model, provider).unwrap();
        }

        db
    }

    #[test]
    fn test_rag_context_retrieval_basic() {
        let db_name = "rag_context_basic";
        let db = setup_rag_test_database(db_name);
        
        // Query about machine learning
        let ml_query = vec![0.75, 0.65, 0.25, 0.1, 0.0, 0.1];
        let similar = db.find_similar(&ml_query, 3).unwrap();
        
        assert_eq!(similar.len(), 3);
        
        // Should retrieve ML-related content
        let context_texts: Vec<&str> = similar.iter().map(|(entry, _)| entry.text.as_str()).collect();
        assert!(context_texts.iter().any(|text| text.contains("machine learning") || text.contains("Machine learning")));
        
        // Results should be ordered by relevance
        assert!(similar[0].1 >= similar[1].1);
        assert!(similar[1].1 >= similar[2].1);

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_rag_context_filtering_by_similarity() {
        let db_name = "rag_context_filtering";
        let db = setup_rag_test_database(db_name);
        
        // Query that should match some content well and others poorly
        let specific_query = vec![0.8, 0.6, 0.2, 0.1, 0.0, 0.1]; // Very similar to first entry
        let similar = db.find_similar(&specific_query, 6).unwrap();
        
        assert_eq!(similar.len(), 6);
        
        // Test similarity threshold filtering (>0.3 as used in RAG implementation)
        let high_similarity_results: Vec<_> = similar.iter()
            .filter(|(_, similarity)| *similarity > 0.3)
            .collect();
        
        // Should have some high-similarity results
        assert!(!high_similarity_results.is_empty());
        
        // First result should have very high similarity
        assert!(similar[0].1 > 0.8, "First result similarity: {}", similar[0].1);

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_rag_context_with_different_topics() {
        let db_name = "rag_context_topics";
        let db = setup_rag_test_database(db_name);
        
        // Test queries for different topics
        let test_queries = vec![
            (vec![0.7, 0.7, 0.3, 0.1, 0.0, 0.1], "deep learning"), // Deep learning query
            (vec![0.5, 0.4, 0.8, 0.2, 0.0, 0.1], "natural language"), // NLP query
            (vec![0.4, 0.3, 0.2, 0.8, 0.2, 0.1], "computer vision"), // Vision query
            (vec![0.2, 0.1, 0.3, 0.1, 0.8, 0.5], "python"), // Programming query
        ];

        for (query_vector, expected_topic) in test_queries {
            let similar = db.find_similar(&query_vector, 2).unwrap();
            assert!(!similar.is_empty());
            
            // Top result should be related to the expected topic
            let top_result = &similar[0].0.text.to_lowercase();
            assert!(top_result.contains(expected_topic),
                   "Query for '{}' didn't return relevant content. Got: '{}'",
                   expected_topic, similar[0].0.text);
        }

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_rag_context_empty_database() {
        let empty_db = VectorDatabase::new("empty_rag_test").unwrap();
        
        let query = vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        let similar = empty_db.find_similar(&query, 3).unwrap();
        
        assert!(similar.is_empty());

        // Cleanup
        VectorDatabase::delete_database("empty_rag_test").unwrap();
    }

    #[test]
    fn test_rag_context_formatting() {
        let db_name = "rag_context_formatting";
        let db = setup_rag_test_database(db_name);
        
        let query = vec![0.8, 0.6, 0.2, 0.1, 0.0, 0.1];
        let similar = db.find_similar(&query, 3).unwrap();
        
        // Simulate context formatting as done in RAG implementation
        let mut context = String::new();
        let mut included_count = 0;
        
        for (entry, similarity) in similar {
            if similarity > 0.3 { // Same threshold as in implementation
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }
        
        assert!(!context.is_empty());
        assert!(included_count > 0);
        
        // Context should be formatted with bullet points
        assert!(context.contains("- "));
        assert!(context.ends_with('\n'));
        
        // Should contain relevant information
        assert!(context.to_lowercase().contains("machine learning") ||
                context.to_lowercase().contains("artificial intelligence"));

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }
}

#[cfg(test)]
mod rag_model_consistency_tests {
    use super::*;

    #[test]
    fn test_rag_model_info_retrieval() {
        let db = VectorDatabase::new("rag_model_test").unwrap();
        let stored_model = "text-embedding-3-small";
        let stored_provider = "openai";
        
        // Add vector with specific model info
        let vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        db.add_vector("Test content", &vector, stored_model, stored_provider).unwrap();
        
        // Retrieve model info as RAG system would
        let model_info = db.get_model_info().unwrap();
        assert!(model_info.is_some());
        
        let (db_model, db_provider) = model_info.unwrap();
        assert_eq!(db_model, stored_model);
        assert_eq!(db_provider, stored_provider);

        // Cleanup
        VectorDatabase::delete_database("rag_model_test").unwrap();
    }

    #[test]
    fn test_rag_embedding_request_creation() {
        let model = "text-embedding-3-small";
        let query = "What is machine learning?";
        
        // Create embedding request as RAG system would
        let embedding_request = EmbeddingRequest {
            model: model.to_string(),
            input: query.to_string(),
            encoding_format: Some("float".to_string()),
        };
        
        assert_eq!(embedding_request.model, model);
        assert_eq!(embedding_request.input, query);
        assert_eq!(embedding_request.encoding_format, Some("float".to_string()));
    }

    #[test]
    fn test_rag_dimension_consistency_check() {
        let db = VectorDatabase::new("rag_dimension_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add vector with specific dimensions
        let stored_vector = vec![0.1; 1536]; // OpenAI text-embedding-3-small dimension
        db.add_vector("Stored content", &stored_vector, model, provider).unwrap();
        
        // Query with same dimensions (should work)
        let matching_query = vec![0.2; 1536];
        let result = db.find_similar(&matching_query, 1);
        assert!(result.is_ok());
        let similar = result.unwrap();
        assert_eq!(similar.len(), 1);
        
        // Query with different dimensions (behavior depends on implementation)
        let mismatched_query = vec![0.2; 1024];
        let result = db.find_similar(&mismatched_query, 1);
        // Implementation should handle this gracefully (error or dimension handling)
        let _ = result;

        // Cleanup
        VectorDatabase::delete_database("rag_dimension_test").unwrap();
    }

    #[test]
    fn test_rag_provider_client_separation() {
        // Test that RAG can use different providers for embedding vs chat
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("venice".to_string()), // Chat provider
            default_model: Some("llama-3.3-70b".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Add chat provider (Venice)
        config.providers.insert("venice".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.venice.ai/api/v1".to_string(),
            api_key: Some("venice-key".to_string()),
            models: vec!["llama-3.3-70b".to_string()],
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Add embedding provider (OpenAI)
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: Some("openai-key".to_string()),
            models: vec!["text-embedding-3-small".to_string()],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Test chat model resolution
        let chat_result = lc::cli::resolve_model_and_provider(&config, None, None);
        assert!(chat_result.is_ok());
        let (chat_provider, chat_model) = chat_result.unwrap();
        assert_eq!(chat_provider, "venice");
        assert_eq!(chat_model, "llama-3.3-70b");

        // Test embedding model resolution (different provider)
        let embed_result = lc::cli::resolve_model_and_provider(&config, Some("openai".to_string()), Some("text-embedding-3-small".to_string()));
        assert!(embed_result.is_ok());
        let (embed_provider, embed_model) = embed_result.unwrap();
        assert_eq!(embed_provider, "openai");
        assert_eq!(embed_model, "text-embedding-3-small");
    }
}

#[cfg(test)]
mod rag_integration_tests {
    use super::*;

    #[test]
    fn test_rag_workflow_simulation() {
        let db_name = "rag_workflow_test";
        let embedding_model = "text-embedding-3-small";
        let embedding_provider = "openai";
        let _chat_model = "gpt-4o-mini";
        let _chat_provider = "openai";

        // 1. Setup: Create database with knowledge
        let db = VectorDatabase::new(db_name).unwrap();
        
        let knowledge_entries = vec![
            ("Rust is a systems programming language focused on safety and performance.", vec![0.8, 0.2, 0.1, 0.0]),
            ("Python is popular for data science and machine learning applications.", vec![0.2, 0.8, 0.1, 0.0]),
            ("JavaScript is the language of the web, running in browsers and servers.", vec![0.1, 0.2, 0.8, 0.0]),
            ("Machine learning models require large datasets for training.", vec![0.1, 0.9, 0.0, 0.1]),
        ];

        for (text, vector) in &knowledge_entries {
            db.add_vector(text, vector, embedding_model, embedding_provider).unwrap();
        }

        // 2. RAG Query: User asks about Python
        let user_query = "Tell me about Python programming";
        let query_vector = vec![0.25, 0.75, 0.15, 0.05]; // Similar to Python entries

        // 3. Context Retrieval: Find relevant information
        let similar = db.find_similar(&query_vector, 3).unwrap();
        assert!(!similar.is_empty());

        // 4. Context Formatting: Format for LLM
        let mut context = String::new();
        let mut relevant_count = 0;
        
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
                relevant_count += 1;
            }
        }

        assert!(!context.is_empty());
        assert!(relevant_count > 0);
        
        // Context should contain Python-related information
        assert!(context.to_lowercase().contains("python"));

        // 5. Enhanced Prompt: Combine context with user query
        let enhanced_prompt = format!(
            "Context from knowledge base:\n{}\n\nUser question: {}",
            context, user_query
        );

        assert!(enhanced_prompt.contains("Context from knowledge base:"));
        assert!(enhanced_prompt.contains("User question:"));
        assert!(enhanced_prompt.contains(user_query));
        assert!(enhanced_prompt.to_lowercase().contains("python"));

        // 6. Model Info Verification: Ensure consistent embedding model
        let model_info = db.get_model_info().unwrap().unwrap();
        assert_eq!(model_info.0, embedding_model);
        assert_eq!(model_info.1, embedding_provider);

        // Cleanup
        VectorDatabase::delete_database(db_name).unwrap();
    }

    #[test]
    fn test_rag_with_multiple_relevant_contexts() {
        let db = VectorDatabase::new("multi_context_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add related content that should all be relevant
        let related_content = vec![
            ("Deep learning is a subset of machine learning using neural networks.", vec![0.9, 0.8, 0.1, 0.0]),
            ("Neural networks consist of layers of interconnected nodes.", vec![0.8, 0.9, 0.1, 0.0]),
            ("Backpropagation is the algorithm used to train neural networks.", vec![0.7, 0.8, 0.2, 0.0]),
            ("Convolutional neural networks are used for image processing.", vec![0.6, 0.7, 0.3, 0.1]),
            ("Recurrent neural networks handle sequential data.", vec![0.5, 0.6, 0.4, 0.1]),
        ];

        for (text, vector) in &related_content {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        // Query about neural networks (should match multiple entries)
        let nn_query = vec![0.8, 0.85, 0.15, 0.05];
        let similar = db.find_similar(&nn_query, 5).unwrap();
        
        assert_eq!(similar.len(), 5);

        // Multiple entries should have high similarity
        let high_similarity_count = similar.iter()
            .filter(|(_, sim)| *sim > 0.5)
            .count();
        
        assert!(high_similarity_count >= 3, "Should have multiple highly relevant results");

        // Format context with multiple relevant entries
        let mut context = String::new();
        for (entry, similarity) in &similar {
            if *similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
            }
        }

        // Should contain information about neural networks from multiple sources
        let context_lower = context.to_lowercase();
        assert!(context_lower.contains("neural"));
        assert!(context_lower.contains("deep learning") || context_lower.contains("backpropagation"));

        // Cleanup
        VectorDatabase::delete_database("multi_context_test").unwrap();
    }

    #[test]
    fn test_rag_with_no_relevant_context() {
        let db = VectorDatabase::new("no_context_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add content unrelated to the query
        let unrelated_content = vec![
            ("Cooking pasta requires boiling water and salt.", vec![0.0, 0.0, 0.0, 1.0]),
            ("Gardening tips for growing tomatoes in summer.", vec![0.0, 0.0, 0.1, 0.9]),
            ("Travel guide to visiting Paris museums.", vec![0.0, 0.1, 0.0, 0.8]),
        ];

        for (text, vector) in &unrelated_content {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        // Query about programming (very different from stored content)
        let programming_query = vec![1.0, 0.0, 0.0, 0.0];
        let similar = db.find_similar(&programming_query, 3).unwrap();
        
        assert_eq!(similar.len(), 3);

        // All similarities should be low
        for (_, similarity) in &similar {
            assert!(*similarity < 0.5, "Similarity should be low for unrelated content: {}", similarity);
        }

        // Context filtering should exclude low-similarity results
        let mut context = String::new();
        let mut included_count = 0;
        
        for (entry, similarity) in similar {
            if similarity > 0.3 { // Same threshold as RAG implementation
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }

        // Should have little or no context due to low similarity
        assert!(included_count <= 1, "Should include few or no results due to low similarity");

        // Cleanup
        VectorDatabase::delete_database("no_context_test").unwrap();
    }

    #[test]
    fn test_rag_context_length_management() {
        let db = VectorDatabase::new("context_length_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";

        // Add content with varying lengths
        let content_entries = vec![
            ("Short text.", vec![0.8, 0.2, 0.0, 0.0]),
            ("This is a medium-length text that contains more information about the topic and provides additional context.", vec![0.7, 0.3, 0.0, 0.0]),
            ("This is a very long text entry that contains extensive information about the topic, providing detailed explanations, examples, and comprehensive coverage of various aspects. It includes multiple sentences and covers different subtopics within the main subject area. This type of content might be typical of documentation, articles, or detailed explanations that would be stored in a knowledge base for retrieval-augmented generation systems.", vec![0.6, 0.4, 0.0, 0.0]),
        ];

        for (text, vector) in &content_entries {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        let query = vec![0.75, 0.25, 0.0, 0.0];
        let similar = db.find_similar(&query, 3).unwrap();
        
        // Format context and measure length
        let mut context = String::new();
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
            }
        }

        assert!(!context.is_empty());
        
        // Context should contain all relevant entries regardless of length
        assert!(context.contains("Short text"));
        assert!(context.contains("medium-length"));
        assert!(context.contains("very long text"));

        // In a real implementation, you might want to limit context length
        // This test verifies that the system can handle varying content lengths

        // Cleanup
        VectorDatabase::delete_database("context_length_test").unwrap();
    }
}

#[cfg(test)]
mod rag_error_handling_tests {
    use super::*;

    #[test]
    fn test_rag_with_nonexistent_database() {
        // Simulate RAG trying to use a database that doesn't exist
        let result = VectorDatabase::new("nonexistent_rag_db");
        
        match result {
            Ok(db) => {
                // If database is created, it should be empty
                let query = vec![0.5, 0.5, 0.5];
                let similar = db.find_similar(&query, 3).unwrap();
                assert!(similar.is_empty());
                
                // Context should be empty
                let mut context = String::new();
                for (entry, similarity) in similar {
                    if similarity > 0.3 {
                        context.push_str(&format!("- {}\n", entry.text));
                    }
                }
                assert!(context.is_empty());
                
                VectorDatabase::delete_database("nonexistent_rag_db").unwrap();
            }
            Err(_) => {
                // Error is also acceptable
            }
        }
    }

    #[test]
    fn test_rag_with_empty_database() {
        let db = VectorDatabase::new("empty_rag_db").unwrap();
        
        // Empty database should return no results
        let query = vec![0.5, 0.5, 0.5];
        let similar = db.find_similar(&query, 3).unwrap();
        assert!(similar.is_empty());
        
        // Context should be empty
        let mut context = String::new();
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
            }
        }
        assert!(context.is_empty());

        // Cleanup
        VectorDatabase::delete_database("empty_rag_db").unwrap();
    }

    #[test]
    fn test_rag_with_invalid_model_info() {
        let db = VectorDatabase::new("invalid_model_rag_test").unwrap();
        
        // Database with no model info
        let model_info = db.get_model_info().unwrap();
        assert!(model_info.is_none());
        
        // RAG system should handle missing model info gracefully
        // In the actual implementation, this would return early with empty context

        // Cleanup
        VectorDatabase::delete_database("invalid_model_rag_test").unwrap();
    }

    #[test]
    fn test_rag_with_dimension_mismatch() {
        let db = VectorDatabase::new("dimension_mismatch_rag_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add vector with specific dimensions
        let stored_vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        db.add_vector("Test content", &stored_vector, model, provider).unwrap();
        
        // Query with different dimensions
        let mismatched_query = vec![0.1, 0.2, 0.3]; // Different dimension count
        let result = db.find_similar(&mismatched_query, 1);
        
        // System should handle dimension mismatch gracefully
        match result {
            Ok(similar) => {
                // If it succeeds, results should be valid
                for (_, similarity) in similar {
                    assert!(similarity.is_finite());
                }
            }
            Err(_) => {
                // Error is also acceptable for dimension mismatch
            }
        }

        // Cleanup
        VectorDatabase::delete_database("dimension_mismatch_rag_test").unwrap();
    }

    #[test]
    fn test_rag_context_formatting_edge_cases() {
        let db = VectorDatabase::new("formatting_edge_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add content with edge cases
        let edge_cases = vec![
            ("", vec![0.8, 0.2, 0.0]), // Empty text
            ("Single word", vec![0.7, 0.3, 0.0]), // Very short text
            ("Text\nwith\nnewlines", vec![0.6, 0.4, 0.0]), // Text with newlines
            ("Text with special chars: !@#$%^&*()", vec![0.5, 0.5, 0.0]), // Special characters
        ];

        for (text, vector) in &edge_cases {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        let query = vec![0.75, 0.25, 0.0];
        let similar = db.find_similar(&query, 4).unwrap();
        
        // Format context
        let mut context = String::new();
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
            }
        }

        // Context should be properly formatted even with edge cases
        if !context.is_empty() {
            assert!(context.contains("- "));
            // Should handle empty text, newlines, and special characters gracefully
        }

        // Cleanup
        VectorDatabase::delete_database("formatting_edge_test").unwrap();
    }
}

#[cfg(test)]
mod rag_performance_tests {
    use super::*;

    #[test]
    fn test_rag_retrieval_performance() {
        let db = VectorDatabase::new("rag_performance_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add many knowledge base entries
        let entry_count = 100;
        for i in 0..entry_count {
            let vector: Vec<f64> = (0..10).map(|j| ((i * 10 + j) as f64) * 0.01).collect();
            let text = format!("Knowledge entry {} with information about topic {}", i, i % 10);
            db.add_vector(&text, &vector, model, provider).unwrap();
        }

        // Test RAG retrieval performance
        let query: Vec<f64> = (0..10).map(|i| (i as f64) * 0.01).collect();
        
        let start = std::time::Instant::now();
        let similar = db.find_similar(&query, 5).unwrap();
        let retrieval_duration = start.elapsed();

        assert_eq!(similar.len(), 5);
        
        // Context formatting performance
        let start = std::time::Instant::now();
        let mut context = String::new();
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
            }
        }
        let formatting_duration = start.elapsed();

        // Performance should be reasonable
        assert!(retrieval_duration.as_millis() < 100, "Retrieval too slow: {:?}", retrieval_duration);
        assert!(formatting_duration.as_millis() < 10, "Formatting too slow: {:?}", formatting_duration);

        // Cleanup
        VectorDatabase::delete_database("rag_performance_test").unwrap();
    }

    #[test]
    fn test_rag_with_large_context() {
        let db = VectorDatabase::new("large_context_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add entries that will all be relevant (high similarity)
        let base_vector = vec![0.8, 0.2, 0.0, 0.0, 0.0];
        for i in 0..20 {
            let mut vector = base_vector.clone();
            vector[0] += (i as f64) * 0.001; // Slight variations to maintain high similarity
            
            let text = format!("This is knowledge entry number {} containing detailed information about the main topic. It includes comprehensive explanations and examples that would be useful for answering user questions.", i);
            db.add_vector(&text, &vector, model, provider).unwrap();
        }

        // Query that should match all entries
        let query = vec![0.8, 0.2, 0.0, 0.0, 0.0];
        let similar = db.find_similar(&query, 20).unwrap();
        
        // Format large context
        let mut context = String::new();
        let mut included_count = 0;
        
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }

        // Should include many relevant entries
        assert!(included_count >= 10, "Should include many relevant entries: {}", included_count);
        
        // Context should be substantial but manageable
        assert!(context.len() > 1000, "Context should be substantial");
        assert!(context.len() < 10000, "Context should not be excessive"); // Reasonable upper bound

        // Cleanup
        VectorDatabase::delete_database("large_context_test").unwrap();
    }

    #[test]
    fn test_rag_similarity_threshold_performance() {
        let db = VectorDatabase::new("threshold_performance_test").unwrap();
        let model = "text-embedding-3-small";
        let provider = "openai";
        
        // Add mix of relevant and irrelevant content
        let mixed_content = vec![
            // Highly relevant (similarity > 0.8)
            ("Machine learning algorithms", vec![0.9, 0.1, 0.0, 0.0]),
            ("Deep learning networks", vec![0.85, 0.15, 0.0, 0.0]),
            ("Neural network training", vec![0.8, 0.2, 0.0, 0.0]),
            
            // Moderately relevant (similarity 0.3-0.8)
            ("Data science methods", vec![0.6, 0.4, 0.0, 0.0]),
            ("Statistical analysis", vec![0.5, 0.5, 0.0, 0.0]),
            ("Computer algorithms", vec![0.4, 0.6, 0.0, 0.0]),
            
            // Low relevance (similarity < 0.3)
            ("Cooking recipes", vec![0.1, 0.0, 0.9, 0.0]),
            ("Travel destinations", vec![0.0, 0.1, 0.0, 0.9]),
            ("Sports statistics", vec![0.0, 0.0, 0.1, 0.9]),
        ];

        for (text, vector) in &mixed_content {
            db.add_vector(text, vector, model, provider).unwrap();
        }

        // Query similar to highly relevant content
        let query = vec![0.9, 0.1, 0.0, 0.0];
        let similar = db.find_similar(&query, 9).unwrap();
        
        // Test threshold filtering efficiency
        let mut high_relevance_count = 0;
        let mut moderate_relevance_count = 0;
        let mut low_relevance_count = 0;
        
        for (_, similarity) in &similar {
            if *similarity > 0.8 {
                high_relevance_count += 1;
            } else if *similarity > 0.3 {
                moderate_relevance_count += 1;
            } else {
                low_relevance_count += 1;
            }
        }

        // Should have clear separation
        assert!(high_relevance_count >= 2, "Should have high relevance results");
        assert!(low_relevance_count >= 2, "Should have low relevance results");
        
        // Context filtering should exclude low relevance
        let mut context = String::new();
        let mut included_count = 0;
        
        for (entry, similarity) in similar {
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }

        // Should include only relevant content
        assert_eq!(included_count, high_relevance_count + moderate_relevance_count);
        assert!(context.to_lowercase().contains("machine learning") ||
                context.to_lowercase().contains("neural"));
        assert!(!context.to_lowercase().contains("cooking") &&
                !context.to_lowercase().contains("travel"));

        // Cleanup
        VectorDatabase::delete_database("threshold_performance_test").unwrap();
    }
}