use anyhow::Result;
use crate::provider::{OpenAIClient, ChatRequest, Message};
use crate::database::ChatEntry;

pub async fn send_chat_request(
    client: &OpenAIClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<String> {
    let mut messages = Vec::new();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content: sys_prompt.to_string(),
        });
    }
    
    // Add conversation history
    for entry in history {
        messages.push(Message {
            role: "user".to_string(),
            content: entry.question.clone(),
        });
        messages.push(Message {
            role: "assistant".to_string(),
            content: entry.response.clone(),
        });
    }
    
    // Add current prompt
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    });
    
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
    };
    
    client.chat(&request).await
}