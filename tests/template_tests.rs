use lc::provider::{ChatRequest, Message};
use lc::template_processor::TemplateProcessor;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_basic_request_template() {
    let mut processor = TemplateProcessor::new().unwrap();

    let template = r#"{
  "model": "{{ model }}",
  "messages": {{ messages | json }},
  "max_tokens": {{ max_tokens | default(value=1024) }}
}"#;

    processor.register_template(template).unwrap();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Hello, world!".to_string())],
        max_tokens: Some(500),
        temperature: None,
        tools: None,
        stream: None,
    };

    let vars = HashMap::new();
    let result = processor
        .process_request(&request, template, &vars)
        .unwrap();

    assert_eq!(result["model"], "gpt-4");
    assert_eq!(result["max_tokens"], 500);
    assert!(result["messages"].is_array());
}

#[test]
fn test_gpt5_template_override() {
    let mut processor = TemplateProcessor::new().unwrap();

    // GPT-5 template that uses max_completion_tokens
    let template = r#"{
  "model": "{{ model }}",
  "messages": {{ messages | json }},
  {% if max_tokens %}"max_completion_tokens": {{ max_tokens }}{% endif %}
}"#;

    processor.register_template(template).unwrap();

    let request = ChatRequest {
        model: "gpt-5-nano".to_string(),
        messages: vec![Message::user("Test message".to_string())],
        max_tokens: Some(2000),
        temperature: None,
        tools: None,
        stream: None,
    };

    let vars = HashMap::new();
    let result = processor
        .process_request(&request, template, &vars)
        .unwrap();

    // Should have max_completion_tokens instead of max_tokens
    assert!(result.get("max_completion_tokens").is_some());
    assert_eq!(result["max_completion_tokens"], 2000);
    assert!(result.get("max_tokens").is_none());
}

#[test]
fn test_response_template() {
    let mut processor = TemplateProcessor::new().unwrap();

    let template = r#"{
  "content": "{{ choices[0].message.content }}",
  "tool_calls": {{ choices[0].message.tool_calls | default(value=[]) | json }}
}"#;

    processor.register_template(template).unwrap();

    let response = json!({
        "choices": [{
            "message": {
                "content": "Hello from the AI!",
                "tool_calls": null
            }
        }]
    });

    let result = processor.process_response(&response, template).unwrap();

    assert_eq!(result["content"], "Hello from the AI!");
    assert_eq!(result["tool_calls"], json!([]));
}

#[test]
fn test_gemini_role_filter() {
    let mut processor = TemplateProcessor::new().unwrap();

    let template = r#"{
  "role": "{{ role | gemini_role }}"
}"#;

    processor.register_template(template).unwrap();

    let _request = ChatRequest {
        model: "gemini-pro".to_string(),
        messages: vec![Message::assistant("I am an assistant".to_string())],
        max_tokens: None,
        temperature: None,
        tools: None,
        stream: None,
    };

    let _vars: HashMap<String, String> = HashMap::new();

    // Process with assistant role
    let mut context = tera::Context::new();
    context.insert("role", "assistant");

    // Use the test-only render_template method
    let result = processor.render_template(template, &context).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // Assistant should be converted to "model" for Gemini
    assert_eq!(parsed["role"], "model");
}

#[test]
fn test_template_with_vars() {
    let mut processor = TemplateProcessor::new().unwrap();

    let template = r#"{
  "project": "{{ project }}",
  "location": "{{ location }}",
  "model": "{{ model }}"
}"#;

    processor.register_template(template).unwrap();

    let request = ChatRequest {
        model: "text-bison".to_string(),
        messages: vec![],
        max_tokens: None,
        temperature: None,
        tools: None,
        stream: None,
    };

    let mut vars = HashMap::new();
    vars.insert("project".to_string(), "my-gcp-project".to_string());
    vars.insert("location".to_string(), "us-central1".to_string());

    let result = processor
        .process_request(&request, template, &vars)
        .unwrap();

    assert_eq!(result["project"], "my-gcp-project");
    assert_eq!(result["location"], "us-central1");
    assert_eq!(result["model"], "text-bison");
}

#[test]
fn test_conditional_fields() {
    let mut processor = TemplateProcessor::new().unwrap();

    let template = r#"{
  "model": "{{ model }}",
  {% if temperature %}"temperature": {{ temperature }},{% endif %}
  {% if tools %}"tools": {{ tools | json }},{% endif %}
  "messages": {{ messages | json }}
}"#;

    processor.register_template(template).unwrap();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message::user("Test".to_string())],
        max_tokens: None,
        temperature: Some(0.7),
        tools: None,
        stream: None,
    };

    let vars = HashMap::new();
    let result = processor
        .process_request(&request, template, &vars)
        .unwrap();

    // Temperature should be present, tools should not
    assert!(result["temperature"].is_f64());
    assert!((result["temperature"].as_f64().unwrap() - 0.7).abs() < 0.0001);
    assert!(result.get("tools").is_none());
}
