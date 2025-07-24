use anyhow::Result;
use crate::provider::{OpenAIClient, ChatRequest, Message};
use crate::database::ChatEntry;

pub async fn send_chat_request(
    client: &OpenAIClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
) -> Result<String> {
    let mut messages = Vec::new();
    
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
        max_tokens: Some(1024),
        temperature: Some(0.7),
    };
    
    client.chat(&request).await
}