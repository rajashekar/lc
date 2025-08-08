use lc::provider::{ChatRequest, Message, MessageContent};
use lc::template_processor::TemplateProcessor;
use std::collections::HashMap;

#[test]
fn test_bedrock_request_template() {
    // Create a sample ChatRequest
    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content_type: MessageContent::Text {
                    content: Some("Hello, how are you?".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content_type: MessageContent::Text {
                    content: Some("I'm doing well, thank you!".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        max_tokens: Some(1000),
        temperature: Some(0.7),
        tools: None,
        stream: None,
    };

    // Bedrock request template
    let template = r#"
{
  "messages": [
    {% for message in messages %}
    {
      "role": "{{ message.role | bedrock_role }}",
      "content": [
        {
          "text": "{{ message.content | default(value="") }}"
        }
      ]
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]{% if max_tokens %},
  "max_tokens": {{ max_tokens }}{% endif %}{% if temperature %},
  "temperature": {{ temperature }}{% endif %}{% if tools %},
  "tools": {{ tools | json }}{% endif %}{% if stream %},
  "stream": {{ stream }}{% endif %}
}
"#;

    let mut processor = TemplateProcessor::new().expect("Failed to create template processor");
    let vars: HashMap<String, String> = HashMap::new();
    
    let result = processor.process_request(&request, template, &vars)
        .expect("Failed to process request template");

    // Verify the structure
    assert!(result.get("messages").is_some());
    let messages = result.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 2);

    // Check first message
    let first_message = &messages[0];
    assert_eq!(first_message.get("role").unwrap().as_str().unwrap(), "user");
    let content = first_message.get("content").unwrap().as_array().unwrap();
    assert_eq!(content[0].get("text").unwrap().as_str().unwrap(), "Hello, how are you?");

    // Check second message (system role should be converted to user for Bedrock)
    let second_message = &messages[1];
    assert_eq!(second_message.get("role").unwrap().as_str().unwrap(), "assistant");

    // Check parameters
    assert_eq!(result.get("max_tokens").unwrap().as_u64().unwrap(), 1000);
    assert!((result.get("temperature").unwrap().as_f64().unwrap() - 0.7).abs() < 0.0001);
}

#[test]
fn test_bedrock_response_template() {
    // Sample Bedrock response
    let bedrock_response = serde_json::json!({
        "output": {
            "message": {
                "content": [
                    {
                        "text": "Hello! I'm Claude, an AI assistant created by Anthropic."
                    }
                ],
                "role": "assistant"
            }
        },
        "stopReason": "end_turn",
        "usage": {
            "inputTokens": 10,
            "outputTokens": 15,
            "totalTokens": 25
        },
        "metrics": {
            "latencyMs": 500
        }
    });

    let template = r#"
{
  "content": "{{ output.message.content[0].text }}"
}
"#;

    let mut processor = TemplateProcessor::new().expect("Failed to create template processor");
    
    let result = processor.process_response(&bedrock_response, template)
        .expect("Failed to process response template");

    // Verify content extraction
    assert_eq!(
        result.get("content").unwrap().as_str().unwrap(),
        "Hello! I'm Claude, an AI assistant created by Anthropic."
    );
}

#[test]
fn test_bedrock_role_filter() {
    // Test the bedrock_role filter through a complete request template
    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content_type: MessageContent::Text {
                    content: Some("You are a helpful assistant.".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content_type: MessageContent::Text {
                    content: Some("Hello!".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content_type: MessageContent::Text {
                    content: Some("Hi there!".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        max_tokens: None,
        temperature: None,
        tools: None,
        stream: None,
    };

    let template = r#"
{
  "messages": [
    {% for message in messages %}
    {
      "role": "{{ message.role | bedrock_role }}",
      "content": [
        {
          "text": "{% if message.content_type.Text %}{{ message.content_type.Text.content | default(value="") }}{% endif %}"
        }
      ]
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]
}
"#;

    let mut processor = TemplateProcessor::new().expect("Failed to create template processor");
    let vars: HashMap<String, String> = HashMap::new();
    
    let result = processor.process_request(&request, template, &vars)
        .expect("Failed to process template with role filter");

    let messages = result.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 3);
    
    // System message should be converted to user
    assert_eq!(messages[0].get("role").unwrap().as_str().unwrap(), "user");
    // User message should remain user
    assert_eq!(messages[1].get("role").unwrap().as_str().unwrap(), "user");
    // Assistant message should remain assistant
    assert_eq!(messages[2].get("role").unwrap().as_str().unwrap(), "assistant");
}

#[test]
fn test_bedrock_claude_specific_template() {
    // Test the Claude-specific template with inferenceConfig
    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content_type: MessageContent::Text {
                    content: Some("What is the capital of France?".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        max_tokens: Some(500),
        temperature: Some(0.5),
        tools: None,
        stream: None,
    };

    let template = r#"
{
  "messages": [
    {% for message in messages %}
    {
      "role": "{{ message.role | bedrock_role }}",
      "content": [
        {
          "text": "{% if message.content_type.Text %}{{ message.content_type.Text.content | default(value="") }}{% elif message.content_type.Multimodal %}{% for part in message.content_type.Multimodal.content %}{% if part.Text %}{{ part.Text.text }}{% if not loop.last %} {% endif %}{% endif %}{% endfor %}{% endif %}"
        }
      ]
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ],
  "inferenceConfig": {
    {% if max_tokens %}"maxTokens": {{ max_tokens }}{% if temperature %},{% endif %}{% endif %}
    {% if temperature %}"temperature": {{ temperature }}{% endif %}
  }{% if tools %},
  "toolConfig": {
    "tools": {{ tools | json }}
  }{% endif %}
}
"#;

    let mut processor = TemplateProcessor::new().expect("Failed to create template processor");
    let vars: HashMap<String, String> = HashMap::new();
    
    let result = processor.process_request(&request, template, &vars)
        .expect("Failed to process Claude-specific template");

    // Verify inferenceConfig structure
    let inference_config = result.get("inferenceConfig").unwrap();
    assert_eq!(inference_config.get("maxTokens").unwrap().as_u64().unwrap(), 500);
    assert_eq!(inference_config.get("temperature").unwrap().as_f64().unwrap(), 0.5);
    
    // Verify messages structure
    let messages = result.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].get("role").unwrap().as_str().unwrap(), "user");
}

#[test]
fn test_bedrock_template_with_system_message() {
    // Test that system messages are converted to user messages for Bedrock
    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content_type: MessageContent::Text {
                    content: Some("You are a helpful assistant.".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content_type: MessageContent::Text {
                    content: Some("Hello!".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        max_tokens: None,
        temperature: None,
        tools: None,
        stream: None,
    };

    let template = r#"
{
  "messages": [
    {% for message in messages %}
    {
      "role": "{{ message.role | bedrock_role }}",
      "content": [
        {
          "text": "{% if message.content_type.Text %}{{ message.content_type.Text.content | default(value="") }}{% elif message.content_type.Multimodal %}{% for part in message.content_type.Multimodal.content %}{% if part.Text %}{{ part.Text.text }}{% if not loop.last %} {% endif %}{% endif %}{% endfor %}{% endif %}"
        }
      ]
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]
}
"#;

    let mut processor = TemplateProcessor::new().expect("Failed to create template processor");
    let vars: HashMap<String, String> = HashMap::new();
    
    let result = processor.process_request(&request, template, &vars)
        .expect("Failed to process template with system message");

    let messages = result.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 2);
    
    // System message should be converted to user
    assert_eq!(messages[0].get("role").unwrap().as_str().unwrap(), "user");
    assert_eq!(messages[1].get("role").unwrap().as_str().unwrap(), "user");
}