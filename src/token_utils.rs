use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};

/// Token counter for various models with caching
pub struct TokenCounter {
    encoder: CoreBPE,
    // LRU cache for token counts to avoid repeated tokenization
    token_cache: Arc<Mutex<LruCache<String, usize>>>,
    // Cache for truncated text to avoid repeated truncation
    truncation_cache: Arc<Mutex<LruCache<(String, usize), String>>>,
}

// Global cache for encoder instances to avoid repeated creation
lazy_static::lazy_static! {
    static ref ENCODER_CACHE: Arc<Mutex<LruCache<String, CoreBPE>>> =
        Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(10).unwrap())));
}

impl TokenCounter {
    /// Create a new token counter for the given model with caching
    pub fn new(model_name: &str) -> Result<Self> {
        // Map model names to tiktoken model names
        let tiktoken_model = map_model_to_tiktoken(model_name);

        // Try to get encoder from cache first
        let encoder = {
            let mut cache = ENCODER_CACHE.lock();
            if let Some(cached_encoder) = cache.get(&tiktoken_model) {
                cached_encoder.clone()
            } else {
                let new_encoder = get_bpe_from_model(&tiktoken_model).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to create token encoder for model '{}': {}",
                        model_name,
                        e
                    )
                })?;
                cache.put(tiktoken_model, new_encoder.clone());
                new_encoder
            }
        };

        Ok(Self {
            encoder,
            token_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
            truncation_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
        })
    }

    /// Count tokens in the given text with caching
    pub fn count_tokens(&self, text: &str) -> usize {
        // Check cache first
        {
            let mut cache = self.token_cache.lock();
            if let Some(&cached_count) = cache.get(text) {
                return cached_count;
            }
        }

        // Calculate token count
        let count = self.encoder.encode_with_special_tokens(text).len();

        // Store in cache
        {
            let mut cache = self.token_cache.lock();
            cache.put(text.to_string(), count);
        }

        count
    }

    /// Estimate tokens for a chat request including system prompt and history
    pub fn estimate_chat_tokens(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        history: &[crate::database::ChatEntry],
    ) -> usize {
        let mut total_tokens = 0;

        // Count system prompt tokens
        if let Some(sys_prompt) = system_prompt {
            total_tokens += self.count_tokens(sys_prompt);
            total_tokens += 4; // Overhead for system message formatting
        }

        // Count history tokens
        for entry in history {
            total_tokens += self.count_tokens(&entry.question);
            total_tokens += self.count_tokens(&entry.response);
            total_tokens += 8; // Overhead for message formatting (role, etc.)
        }

        // Count current prompt tokens
        total_tokens += self.count_tokens(prompt);
        total_tokens += 4; // Overhead for user message formatting

        // Add some buffer for response generation
        total_tokens += 100; // Reserve space for response start

        total_tokens
    }

    /// Check if the estimated tokens exceed the context limit
    pub fn exceeds_context_limit(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        history: &[crate::database::ChatEntry],
        context_limit: u32,
    ) -> bool {
        let estimated_tokens = self.estimate_chat_tokens(prompt, system_prompt, history);
        estimated_tokens > context_limit as usize
    }

    /// Truncate input to fit within context limit
    pub fn truncate_to_fit(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        history: &[crate::database::ChatEntry],
        context_limit: u32,
        max_output_tokens: Option<u32>,
    ) -> (String, Vec<crate::database::ChatEntry>) {
        let max_output = max_output_tokens.unwrap_or(4096) as usize;
        let available_tokens = (context_limit as usize).saturating_sub(max_output);

        // Always preserve the current prompt and system prompt
        let mut used_tokens = self.count_tokens(prompt) + 4; // User message overhead
        if let Some(sys_prompt) = system_prompt {
            used_tokens += self.count_tokens(sys_prompt) + 4; // System message overhead
        }

        if used_tokens >= available_tokens {
            // Even the prompt alone is too large, truncate it
            let max_prompt_tokens = available_tokens.saturating_sub(100); // Leave some buffer
            let truncated_prompt = self.truncate_text(prompt, max_prompt_tokens);
            return (truncated_prompt, Vec::new());
        }

        // Include as much history as possible
        let mut truncated_history = Vec::new();
        let remaining_tokens = available_tokens - used_tokens;
        let mut history_tokens = 0;

        // Include history from most recent to oldest
        for entry in history.iter().rev() {
            let entry_tokens =
                self.count_tokens(&entry.question) + self.count_tokens(&entry.response) + 8;
            if history_tokens + entry_tokens <= remaining_tokens {
                history_tokens += entry_tokens;
                truncated_history.insert(0, entry.clone());
            } else {
                break;
            }
        }

        (prompt.to_string(), truncated_history)
    }

    /// Truncate text to fit within token limit with caching
    fn truncate_text(&self, text: &str, max_tokens: usize) -> String {
        let cache_key = (text.to_string(), max_tokens);

        // Check cache first
        {
            let mut cache = self.truncation_cache.lock();
            if let Some(cached_result) = cache.get(&cache_key) {
                return cached_result.clone();
            }
        }

        let tokens = self.encoder.encode_with_special_tokens(text);
        if tokens.len() <= max_tokens {
            return text.to_string();
        }

        let result = {
            let truncated_tokens = &tokens[..max_tokens];
            match self.encoder.decode(truncated_tokens.to_vec()) {
                Ok(decoded) => decoded,
                Err(_) => {
                    // Fallback: truncate by characters (rough approximation)
                    let chars: Vec<char> = text.chars().collect();
                    let estimated_chars = max_tokens * 3; // Rough estimate: 1 token ≈ 3-4 chars
                    if chars.len() > estimated_chars {
                        chars[..estimated_chars].iter().collect()
                    } else {
                        text.to_string()
                    }
                }
            }
        };

        // Store in cache
        {
            let mut cache = self.truncation_cache.lock();
            cache.put(cache_key, result.clone());
        }

        result
    }
}

/// Map model names to tiktoken-compatible model names
/// This is a simplified fallback approach - ideally tokenizer mappings should be
/// configured per provider in configuration files for accuracy
fn map_model_to_tiktoken(model_name: &str) -> String {
    let lower_name = model_name.to_lowercase();

    // Only handle actual OpenAI models with their correct tokenizers
    if lower_name.contains("gpt-4") {
        "gpt-4".to_string()
    } else if lower_name.contains("gpt-3.5") {
        "gpt-3.5-turbo".to_string()
    } else {
        // For all non-OpenAI models, use GPT-4 as a rough approximation
        // NOTE: This is inaccurate but necessary since tiktoken only supports OpenAI models
        // TODO: Move to provider-specific tokenizer configuration or disable token counting
        // for non-OpenAI models to avoid misleading estimates
        "gpt-4".to_string()
    }
}
