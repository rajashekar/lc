//! Tests for template pattern matching functionality

use lc::config::ProviderConfig;
use lc::template_processor::TemplateConfig;
use std::collections::HashMap;

#[test]
fn test_regex_pattern_matching() {
    let mut provider_config = ProviderConfig {
        endpoint: "https://api.openai.com/v1".to_string(),
        api_key: Some("test-key".to_string()),
        models: vec!["gpt-5".to_string(), "gpt-5-mini".to_string(), "gpt-5-turbo".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat/completions".to_string(),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
        auth_type: None,
        vars: HashMap::new(),
        images_path: None,
        embeddings_path: None,
        chat_templates: None,
        images_templates: None,
        embeddings_templates: None,
        models_templates: None,
    };

    // Create chat endpoint templates
    let mut chat_templates = HashMap::new();

    // Add a pattern-based template
    let gpt5_template = TemplateConfig {
        request: Some(r#"{"model": "{{ model }}", "max_completion_tokens": {{ max_tokens }}}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert("gpt-5.*".to_string(), gpt5_template.clone());

    // Add a specific model template (should take precedence)
    let gpt5_nano_template = TemplateConfig {
        request: Some(r#"{"model": "{{ model }}", "nano_tokens": {{ max_tokens }}}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert("gpt-5-nano".to_string(), gpt5_nano_template.clone());

    provider_config.chat_templates = Some(chat_templates);

    // Test exact match takes precedence
    let template = provider_config.get_endpoint_template("chat", "gpt-5-nano");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"model": "{{ model }}", "nano_tokens": {{ max_tokens }}}"#);

    // Test pattern matching for gpt-5
    let template = provider_config.get_endpoint_template("chat", "gpt-5");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"model": "{{ model }}", "max_completion_tokens": {{ max_tokens }}}"#);

    // Test pattern matching for gpt-5-mini
    let template = provider_config.get_endpoint_template("chat", "gpt-5-mini");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"model": "{{ model }}", "max_completion_tokens": {{ max_tokens }}}"#);

    // Test pattern matching for gpt-5-turbo
    let template = provider_config.get_endpoint_template("chat", "gpt-5-turbo");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"model": "{{ model }}", "max_completion_tokens": {{ max_tokens }}}"#);

    // Test no match
    let template = provider_config.get_endpoint_template("chat", "gpt-4");
    assert!(template.is_none());
}

#[test]
fn test_multiple_patterns() {
    let mut provider_config = ProviderConfig {
        endpoint: "https://api.example.com".to_string(),
        api_key: Some("test-key".to_string()),
        models: vec!["model-v1".to_string(), "model-v2".to_string(), "other-model".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat".to_string(),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
        auth_type: None,
        vars: HashMap::new(),
        images_path: None,
        embeddings_path: None,
        chat_templates: None,
        images_templates: None,
        embeddings_templates: None,
        models_templates: None,
    };

    // Create chat endpoint templates
    let mut chat_templates = HashMap::new();

    // Add multiple patterns
    let v1_template = TemplateConfig {
        request: Some(r#"{"version": "v1"}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert(".*-v1$".to_string(), v1_template.clone());

    let v2_template = TemplateConfig {
        request: Some(r#"{"version": "v2"}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert(".*-v2$".to_string(), v2_template.clone());

    let other_template = TemplateConfig {
        request: Some(r#"{"version": "other"}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert("other-.*".to_string(), other_template.clone());

    provider_config.chat_templates = Some(chat_templates);

    // Test v1 pattern
    let template = provider_config.get_endpoint_template("chat", "model-v1");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"version": "v1"}"#);

    // Test v2 pattern
    let template = provider_config.get_endpoint_template("chat", "model-v2");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"version": "v2"}"#);

    // Test other pattern
    let template = provider_config.get_endpoint_template("chat", "other-model");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"version": "other"}"#);

    // Test no match
    let template = provider_config.get_endpoint_template("chat", "different-model");
    assert!(template.is_none());
}

#[test]
fn test_default_template_fallback() {
    let mut provider_config = ProviderConfig {
        endpoint: "https://api.example.com".to_string(),
        api_key: Some("test-key".to_string()),
        models: vec!["test-model".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat".to_string(),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
        auth_type: None,
        vars: HashMap::new(),
        images_path: None,
        embeddings_path: None,
        chat_templates: None,
        images_templates: None,
        embeddings_templates: None,
        models_templates: None,
    };

    // Create chat endpoint templates with default
    let mut chat_templates = HashMap::new();

    // Add default template (empty key)
    let default_template = TemplateConfig {
        request: Some(r#"{"type": "default"}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert("".to_string(), default_template);

    // Add a specific pattern
    let specific_template = TemplateConfig {
        request: Some(r#"{"type": "specific"}"#.to_string()),
        response: None,
        stream_response: None,
    };
    chat_templates.insert("specific-.*".to_string(), specific_template);

    provider_config.chat_templates = Some(chat_templates);

    // Test specific pattern match
    let template = provider_config.get_endpoint_template("chat", "specific-model");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"type": "specific"}"#);

    // Test fallback to default
    let template = provider_config.get_endpoint_template("chat", "other-model");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"type": "default"}"#);
}

#[test]
fn test_endpoint_specific_templates() {
    let mut provider_config = ProviderConfig {
        endpoint: "https://api.example.com".to_string(),
        api_key: Some("test-key".to_string()),
        models: vec!["test-model".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat".to_string(),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
        auth_type: None,
        vars: HashMap::new(),
        images_path: Some("/images".to_string()),
        embeddings_path: Some("/embeddings".to_string()),
        chat_templates: None,
        images_templates: None,
        embeddings_templates: None,
        models_templates: None,
    };

    // Create different templates for different endpoints
    let mut chat_templates = HashMap::new();
    chat_templates.insert("".to_string(), TemplateConfig {
        request: Some(r#"{"endpoint": "chat"}"#.to_string()),
        response: None,
        stream_response: None,
    });

    let mut images_templates = HashMap::new();
    images_templates.insert("".to_string(), TemplateConfig {
        request: Some(r#"{"endpoint": "images"}"#.to_string()),
        response: None,
        stream_response: None,
    });

    provider_config.chat_templates = Some(chat_templates);
    provider_config.images_templates = Some(images_templates);

    // Test chat endpoint
    let template = provider_config.get_endpoint_template("chat", "test-model");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"endpoint": "chat"}"#);

    // Test images endpoint
    let template = provider_config.get_endpoint_template("images", "test-model");
    assert!(template.is_some());
    assert_eq!(template.unwrap(), r#"{"endpoint": "images"}"#);

    // Test missing endpoint
    let template = provider_config.get_endpoint_template("embeddings", "test-model");
    assert!(template.is_none());
}