use crate::{chat, config, database, input::MultiLineInput, readers};
use crate::provider_installer::{AuthType, ProviderInstaller};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use rpassword::read_password;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

// Global debug flag
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

// Debug logging macro
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::cli::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            use colored::Colorize;
            eprintln!("{} {}", "[DEBUG]".dimmed(), format!($($arg)*));
        }
    };
}

// Set debug mode
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

#[derive(Parser)]
#[command(name = "lc")]
#[command(
    about = "LLM Client - A fast Rust-based LLM CLI tool with PDF support and RAG capabilities"
)]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Direct prompt to send to the default model
    #[arg(value_name = "PROMPT")]
    pub prompt: Vec<String>,

    /// Provider to use for the prompt
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,

    /// Model to use for the prompt
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// System prompt to use (when used with direct prompt)
    #[arg(short = 's', long = "system")]
    pub system_prompt: Option<String>,

    /// Max tokens override (supports 'k' suffix, e.g., '2k' for 2000)
    #[arg(long = "max-tokens")]
    pub max_tokens: Option<String>,

    /// Temperature override (0.0 to 2.0)
    #[arg(long = "temperature")]
    pub temperature: Option<String>,

    /// Attach file(s) to the prompt (supports text files, PDFs with 'pdf' feature)
    #[arg(short = 'a', long = "attach")]
    pub attachments: Vec<String>,

    /// Attach image(s) to the prompt (supports jpg, png, gif, webp, or URLs)
    #[arg(short = 'i', long = "image")]
    pub images: Vec<String>,

    /// Attach audio file(s) for transcription (supports mp3, wav, flac, etc.)
    #[arg(short = 'u', long = "audio")]
    pub audio_files: Vec<String>,

    /// Include tools from MCP server(s) (comma-separated server names)
    #[arg(short = 't', long = "tools")]
    pub tools: Option<String>,

    /// Vector database name for RAG (Retrieval-Augmented Generation)
    #[arg(short = 'v', long = "vectordb")]
    pub vectordb: Option<String>,

    /// Enable debug/verbose logging
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Continue the current session (use existing session ID)
    #[arg(short = 'c', long = "continue")]
    pub continue_session: bool,

    /// Chat ID to use or continue (alternative to --continue)
    #[arg(long = "cid")]
    pub chat_id: Option<String>,

    /// Use search results as context (format: provider or provider:query)
    #[arg(long = "use-search")]
    pub use_search: Option<String>,

    /// Enable streaming output for prompt responses
    #[arg(long = "stream")]
    pub stream: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CompletionShell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
    /// Elvish shell
    Elvish,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Provider management (alias: p)
    #[command(alias = "p")]
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
    /// API key management (alias: k)
    #[command(alias = "k")]
    Keys {
        #[command(subcommand)]
        command: KeyCommands,
    },
    /// Log management (alias: l)
    #[command(alias = "l")]
    Logs {
        #[command(subcommand)]
        command: LogCommands,
    },
    /// Usage statistics and analytics (alias: u)
    #[command(alias = "u")]
    Usage {
        #[command(subcommand)]
        command: Option<UsageCommands>,
        /// Show usage for the last N days
        #[arg(short = 'd', long = "days")]
        days: Option<u32>,
        /// Show only token usage (default shows both tokens and requests)
        #[arg(short = 't', long = "tokens")]
        tokens_only: bool,
        /// Show only request counts
        #[arg(short = 'r', long = "requests")]
        requests_only: bool,
        /// Maximum number of items to show in charts
        #[arg(short = 'n', long = "limit", default_value = "10")]
        limit: usize,
    },
    /// Configuration management (alias: co)
    #[command(alias = "co")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    /// Interactive chat mode (alias: c)
    #[command(alias = "c")]
    Chat {
        /// Model to use for the chat
        #[arg(short, long)]
        model: Option<String>,
        /// Provider to use for the chat
        #[arg(short, long)]
        provider: Option<String>,
        /// Chat ID to use or continue
        #[arg(long)]
        cid: Option<String>,
        /// Include tools from MCP server(s) (comma-separated server names)
        #[arg(short = 't', long = "tools")]
        tools: Option<String>,
        /// Vector database name for RAG (Retrieval-Augmented Generation)
        #[arg(short = 'v', long = "vectordb")]
        database: Option<String>,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
        /// Attach image(s) to the chat (supports jpg, png, gif, webp, or URLs)
        #[arg(short = 'i', long = "image")]
        images: Vec<String>,
    },
    /// Global models management (alias: m)
    #[command(alias = "m")]
    Models {
        #[command(subcommand)]
        command: Option<ModelsCommands>,
        /// Search query for models (case-insensitive)
        #[arg(short = 'q', long = "query")]
        query: Option<String>,
        /// Filter models that support tools/function calling
        #[arg(long = "tools")]
        tools: bool,
        /// Filter models that support reasoning
        #[arg(long = "reasoning")]
        reasoning: bool,
        /// Filter models that support vision
        #[arg(long = "vision")]
        vision: bool,
        /// Filter models that support audio
        #[arg(long = "audio")]
        audio: bool,
        /// Filter models that support code generation
        #[arg(long = "code")]
        code: bool,
        /// Filter models with minimum context length (e.g., 128k)
        #[arg(long = "ctx")]
        context_length: Option<String>,
        /// Filter models with minimum input token length (e.g., 128k)
        #[arg(long = "input")]
        input_length: Option<String>,
        /// Filter models with minimum output token length (e.g., 128k)
        #[arg(long = "output")]
        output_length: Option<String>,
        /// Filter models with maximum input price per million tokens
        #[arg(long = "input-price")]
        input_price: Option<f64>,
        /// Filter models with maximum output price per million tokens
        #[arg(long = "output-price")]
        output_price: Option<f64>,
    },
    /// Model alias management (alias: a)
    #[command(alias = "a")]
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },
    /// Template management (alias: t)
    #[command(alias = "t")]
    Templates {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// Proxy server (alias: pr)
    #[command(alias = "pr")]
    Proxy {
        /// Port to listen on
        #[arg(short = 'p', long = "port", default_value = "6789")]
        port: u16,
        /// Host to bind to
        #[arg(long = "host", default_value = "127.0.0.1")]
        host: String,
        /// Filter by provider
        #[arg(long = "provider")]
        provider: Option<String>,
        /// Filter by specific model (can be provider:model or alias)
        #[arg(short = 'm', long = "model")]
        model: Option<String>,
        /// API key for authentication
        #[arg(short = 'k', long = "key")]
        api_key: Option<String>,
        /// Generate a random API key
        #[arg(short = 'g', long = "generate-key")]
        generate_key: bool,
    },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Generate embeddings for text (alias: e)
    #[command(alias = "e")]
    Embed {
        /// Model to use for embeddings
        #[arg(short, long)]
        model: String,
        /// Provider to use for embeddings
        #[arg(short, long)]
        provider: Option<String>,
        /// Vector database name to store embeddings
        #[arg(short = 'v', long = "vectordb")]
        database: Option<String>,
        /// Files to embed (supports glob patterns, including PDFs with 'pdf' feature)
        #[arg(short = 'f', long = "files")]
        files: Vec<String>,
        /// Text to embed (optional if files are provided)
        text: Option<String>,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },
    /// Find similar text using vector similarity (alias: s)
    #[command(alias = "s")]
    Similar {
        /// Model to use for embeddings (optional if database has existing model)
        #[arg(short, long)]
        model: Option<String>,
        /// Provider to use for embeddings (optional if database has existing model)
        #[arg(short, long)]
        provider: Option<String>,
        /// Vector database name to search
        #[arg(short = 'v', long = "vectordb")]
        database: String,
        /// Number of similar results to return
        #[arg(short, long, default_value = "5")]
        limit: usize,
        /// Query text to find similar content
        query: String,
    },
    /// Vector database management (alias: v)
    #[command(alias = "v")]
    Vectors {
        #[command(subcommand)]
        command: VectorCommands,
    },
    /// Web chat proxy for non-OpenAI compatible services (alias: w)
    #[command(alias = "w")]
    WebChatProxy {
        #[command(subcommand)]
        command: WebChatProxyCommands,
    },
    /// Sync configuration files to/from cloud providers (alias: sy)
    #[command(alias = "sy")]
    Sync {
        #[command(subcommand)]
        command: SyncCommands,
    },
    /// Search provider management (alias: se)
    #[command(alias = "se")]
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },
    /// Generate images from text prompts (alias: img)
    #[command(alias = "img")]
    Image {
        /// Text prompt for image generation
        prompt: String,
        /// Model to use for image generation
        #[arg(short, long)]
        model: Option<String>,
        /// Provider to use for image generation
        #[arg(short, long)]
        provider: Option<String>,
        /// Image size (e.g., "1024x1024", "512x512")
        #[arg(short, long, default_value = "1024x1024")]
        size: String,
        /// Number of images to generate
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Output directory for generated images
        #[arg(short, long)]
        output: Option<String>,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },
    /// Transcribe audio to text (alias: tr)
    #[command(alias = "tr")]
    Transcribe {
        /// Audio file(s) to transcribe (supports mp3, wav, flac, etc.)
        audio_files: Vec<String>,
        /// Model to use for transcription
        #[arg(short, long)]
        model: Option<String>,
        /// Provider to use for transcription
        #[arg(short, long)]
        provider: Option<String>,
        /// Language of the audio (ISO-639-1 format, e.g., "en", "es")
        #[arg(short = 'l', long)]
        language: Option<String>,
        /// Optional prompt to guide the transcription
        #[arg(long)]
        prompt: Option<String>,
        /// Response format (json, text, srt, verbose_json, vtt)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
        /// Temperature for transcription (0.0 to 1.0)
        #[arg(long)]
        temperature: Option<f32>,
        /// Output file for transcription (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },
    /// Convert text to speech
    TTS {
        /// Text to convert to speech
        text: String,
        /// Model to use for TTS
        #[arg(short, long)]
        model: Option<String>,
        /// Provider to use for TTS
        #[arg(short, long)]
        provider: Option<String>,
        /// Voice to use (e.g., alloy, echo, fable, onyx, nova, shimmer)
        #[arg(short = 'v', long, default_value = "alloy")]
        voice: String,
        /// Output audio format (mp3, opus, aac, flac, wav, pcm)
        #[arg(short = 'f', long, default_value = "mp3")]
        format: String,
        /// Speech speed (0.25 to 4.0)
        #[arg(short = 's', long)]
        speed: Option<f32>,
        /// Output file for audio (required)
        #[arg(short, long)]
        output: String,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },
    /// Dump metadata JSON from models cache (alias: dump)
    #[command(alias = "dump")]
    DumpMetadata {
        /// Specific provider to dump (optional - dumps all if not specified)
        provider: Option<String>,
        /// List available cached metadata files
        #[arg(short, long)]
        list: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// Refresh the models cache (alias: r)
    #[command(alias = "r")]
    Refresh,
    /// Show cache information (alias: i)
    #[command(alias = "i")]
    Info,
    /// Dump raw /models responses to JSON files (alias: d)
    #[command(alias = "d")]
    Dump,
    /// List embedding models (alias: e)
    #[command(alias = "e")]
    Embed,
    /// Manage model paths for extraction (alias: p)
    #[command(alias = "p")]
    Path {
        #[command(subcommand)]
        command: ModelsPathCommands,
    },
    /// Manage model tags and extraction rules (alias: t)
    #[command(alias = "t")]
    Tags {
        #[command(subcommand)]
        command: ModelsTagsCommands,
    },
    /// Filter models by tags (alias: f)
    #[command(alias = "f")]
    Filter {
        /// Tags to filter by (comma-separated)
        #[arg(short = 't', long = "tag")]
        tags: String,
    },
}

#[derive(Subcommand)]
pub enum ModelsPathCommands {
    /// List all model extraction paths (alias: l)
    #[command(alias = "l")]
    List,
    /// Add a new model extraction path (alias: a)
    #[command(alias = "a")]
    Add {
        /// JQ-style path to add
        path: String,
    },
    /// Delete a model extraction path (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Path to delete
        path: String,
    },
}

#[derive(Subcommand)]
pub enum ModelsTagsCommands {
    /// List all tags and their rules (alias: l)
    #[command(alias = "l")]
    List,
    /// Add a rule to a tag (alias: a)
    #[command(alias = "a")]
    Add {
        /// Tag name
        tag: String,
        /// Extraction rule (JQ-style path or search pattern)
        rule: String,
    },
}

#[derive(Subcommand)]
pub enum AliasCommands {
    /// Add a new alias (alias: a)
    #[command(alias = "a")]
    Add {
        /// Alias name
        name: String,
        /// Provider and model in format provider:model
        target: String,
    },
    /// Remove an alias (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Alias name to remove
        name: String,
    },
    /// List all aliases (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Add a new template (alias: a)
    #[command(alias = "a")]
    Add {
        /// Template name
        name: String,
        /// Template prompt content
        prompt: String,
    },
    /// Remove a template (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Template name to remove
        name: String,
    },
    /// List all templates (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// Install a provider from the registry (alias: i)
    #[command(alias = "i")]
    Install {
        /// Provider name to install
        name: String,
        /// Force reinstall even if already installed
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
    /// Update installed providers (alias: up)
    #[command(alias = "up")]
    Upgrade {
        /// Provider name to update (updates all if not specified)
        name: Option<String>,
    },
    /// Uninstall a provider (alias: un)
    #[command(alias = "un")]
    Uninstall {
        /// Provider name to uninstall
        name: String,
    },
    /// List available providers from registry (alias: av)
    #[command(alias = "av")]
    Available {
        /// Show only official providers
        #[arg(long = "official")]
        official: bool,
        /// Filter by tag
        #[arg(short = 't', long = "tag")]
        tag: Option<String>,
    },
    /// Add a new provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
        /// Provider endpoint URL
        url: String,
        /// Custom models endpoint path (default: /models)
        #[arg(short = 'm', long = "models-path")]
        models_path: Option<String>,
        /// Custom chat completions endpoint path (default: /chat/completions)
        #[arg(short = 'c', long = "chat-path")]
        chat_path: Option<String>,
    },
    /// Update an existing provider (alias: u)
    #[command(alias = "u")]
    Update {
        /// Provider name
        name: String,
        /// Provider endpoint URL
        url: String,
    },
    /// Remove a provider (alias: r)
    #[command(alias = "r")]
    Remove {
        /// Provider name
        name: String,
    },
    /// List all providers (alias: l)
    #[command(alias = "l")]
    List,
    /// List available models for a provider (alias: m)
    #[command(alias = "m")]
    Models {
        /// Provider name
        name: String,
        /// Refresh the models cache for this provider (alias: r)
        #[arg(short = 'r', long = "refresh")]
        refresh: bool,
    },
    /// Manage custom headers for a provider (alias: h)
    #[command(alias = "h")]
    Headers {
        /// Provider name
        provider: String,
        #[command(subcommand)]
        command: HeaderCommands,
    },
    /// Manage provider variables for path templating (alias: v)
    #[command(alias = "v")]
    Vars {
        /// Provider name
        provider: String,
        #[command(subcommand)]
        command: ProviderVarsCommands,
    },
    /// Set token URL for a provider (alias: t)
    #[command(alias = "t")]
    TokenUrl {
        /// Provider name
        provider: String,
        /// Token URL for dynamic token retrieval
        url: String,
    },
    /// Manage provider API paths (alias: path)
    #[command(alias = "path")]
    Paths {
        /// Provider name
        provider: String,
        #[command(subcommand)]
        command: ProviderPathCommands,
    },
}

#[derive(Subcommand)]
pub enum ProviderVarsCommands {
    /// Set a provider variable (alias: s)
    #[command(alias = "s")]
    Set {
        /// Variable key (e.g., project, location)
        key: String,
        /// Variable value
        value: String,
    },
    /// Get a provider variable (alias: g)
    #[command(alias = "g")]
    Get {
        /// Variable key
        key: String,
    },
    /// List all provider variables (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum ProviderPathCommands {
    /// Add or update a provider path (alias: a)
    #[command(alias = "a")]
    Add {
        /// Models path
        #[arg(short = 'm', long = "models")]
        models_path: Option<String>,
        /// Chat completions path
        #[arg(short = 'c', long = "chat")]
        chat_path: Option<String>,
        /// Image generations path
        #[arg(short = 'i', long = "images")]
        images_path: Option<String>,
        /// Embeddings path
        #[arg(short = 'e', long = "embeddings")]
        embeddings_path: Option<String>,
    },
    /// Delete a provider path (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Delete models path
        #[arg(short = 'm', long = "models")]
        models: bool,
        /// Delete chat completions path
        #[arg(short = 'c', long = "chat")]
        chat: bool,
        /// Delete image generations path
        #[arg(short = 'i', long = "images")]
        images: bool,
        /// Delete embeddings path
        #[arg(short = 'e', long = "embeddings")]
        embeddings: bool,
    },
    /// List all provider paths (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum HeaderCommands {
    /// Add a custom header (alias: a)
    #[command(alias = "a")]
    Add {
        /// Header name
        name: String,
        /// Header value
        value: String,
    },
    /// Remove a custom header (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Header name
        name: String,
    },
    /// List all custom headers (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Add API key for a provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
    },
    /// List providers with API keys (alias: l)
    #[command(alias = "l")]
    List,
    /// Get API key for a provider (alias: g)
    #[command(alias = "g")]
    Get {
        /// Provider name
        name: String,
    },
    /// Remove API key for a provider (alias: r)
    #[command(alias = "r")]
    Remove {
        /// Provider name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum LogCommands {
    /// Show all logs (alias: sh)
    #[command(alias = "sh")]
    Show {
        /// Show minimal table format
        #[arg(long)]
        minimal: bool,
    },
    /// Show recent logs (alias: r)
    #[command(alias = "r")]
    Recent {
        #[command(subcommand)]
        command: Option<RecentCommands>,
        /// Number of recent entries to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Show current session logs (alias: c)
    #[command(alias = "c")]
    Current,
    /// Show database statistics (alias: s)
    #[command(alias = "s")]
    Stats,
    /// Purge all logs (alias: p)
    #[command(alias = "p")]
    Purge {
        /// Confirm purge without prompt
        #[arg(long)]
        yes: bool,
        /// Purge logs older than N days
        #[arg(long)]
        older_than_days: Option<u32>,
        /// Keep only the most recent N entries
        #[arg(long)]
        keep_recent: Option<usize>,
        /// Purge when database exceeds N MB
        #[arg(long)]
        max_size_mb: Option<u64>,
    },
}

#[derive(Subcommand)]
pub enum RecentCommands {
    /// Get last answer from LLM (alias: a)
    #[command(alias = "a")]
    Answer {
        #[command(subcommand)]
        command: Option<AnswerCommands>,
    },
    /// Get last question/prompt asked to LLM (alias: q)
    #[command(alias = "q")]
    Question,
    /// Get model used in last interaction (alias: m)
    #[command(alias = "m")]
    Model,
    /// Get session ID of last interaction (alias: s)
    #[command(alias = "s")]
    Session,
}

#[derive(Subcommand)]
pub enum UsageCommands {
    /// Show daily usage statistics (alias: d)
    #[command(alias = "d")]
    Daily {
        /// Number of days to show
        #[arg(short = 'n', long = "count", default_value = "30")]
        count: usize,
    },
    /// Show weekly usage statistics (alias: w)
    #[command(alias = "w")]
    Weekly {
        /// Number of weeks to show
        #[arg(short = 'n', long = "count", default_value = "12")]
        count: usize,
    },
    /// Show monthly usage statistics (alias: m)
    #[command(alias = "m")]
    Monthly {
        /// Number of months to show
        #[arg(short = 'n', long = "count", default_value = "12")]
        count: usize,
    },
    /// Show yearly usage statistics (alias: y)
    #[command(alias = "y")]
    Yearly {
        /// Number of years to show
        #[arg(short = 'n', long = "count", default_value = "5")]
        count: usize,
    },
    /// Show top models by usage (alias: models)
    #[command(alias = "models")]
    Models {
        /// Number of models to show
        #[arg(short = 'n', long = "count", default_value = "10")]
        count: usize,
    },
}

#[derive(Subcommand)]
pub enum AnswerCommands {
    /// Extract code blocks from last answer (alias: c)
    #[command(alias = "c")]
    Code,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set configuration values (alias: s)
    #[command(alias = "s")]
    Set {
        #[command(subcommand)]
        command: SetCommands,
    },
    /// Get configuration values (alias: g)
    #[command(alias = "g")]
    Get {
        #[command(subcommand)]
        command: GetCommands,
    },
    /// Delete/unset configuration values (alias: d)
    #[command(alias = "d")]
    Delete {
        #[command(subcommand)]
        command: DeleteCommands,
    },
    /// Show configuration directory path (alias: p)
    #[command(alias = "p")]
    Path,
}

#[derive(Subcommand)]
pub enum SetCommands {
    /// Set default provider (alias: p)
    #[command(alias = "p")]
    Provider {
        /// Provider name
        name: String,
    },
    /// Set default model (alias: m)
    #[command(alias = "m")]
    Model {
        /// Model name
        name: String,
    },
    /// Set system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt {
        /// System prompt text
        prompt: String,
    },
    /// Set max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens {
        /// Max tokens value (supports 'k' suffix, e.g., '2k' for 2000)
        value: String,
    },
    /// Set temperature (alias: te)
    #[command(alias = "te")]
    Temperature {
        /// Temperature value (0.0 to 2.0)
        value: String,
    },
    /// Set default search provider (alias: se)
    #[command(alias = "se")]
    Search {
        /// Search provider name
        name: String,
    },
    /// Set streaming mode (alias: st)
    #[command(alias = "st")]
    Stream {
        /// Enable or disable streaming (true/false)
        value: String,
    },
}

#[derive(Subcommand)]
pub enum GetCommands {
    /// Get default provider (alias: p)
    #[command(alias = "p")]
    Provider,
    /// Get default model (alias: m)
    #[command(alias = "m")]
    Model,
    /// Get system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt,
    /// Get max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens,
    /// Get temperature (alias: te)
    #[command(alias = "te")]
    Temperature,
    /// Get default search provider (alias: se)
    #[command(alias = "se")]
    Search,
    /// Get streaming mode (alias: st)
    #[command(alias = "st")]
    Stream,
}

#[derive(Subcommand)]
pub enum DeleteCommands {
    /// Delete default provider (alias: p)
    #[command(alias = "p")]
    Provider,
    /// Delete default model (alias: m)
    #[command(alias = "m")]
    Model,
    /// Delete system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt,
    /// Delete max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens,
    /// Delete temperature (alias: te)
    #[command(alias = "te")]
    Temperature,
    /// Delete default search provider (alias: se)
    #[command(alias = "se")]
    Search,
    /// Delete streaming mode (alias: st)
    #[command(alias = "st")]
    Stream,
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// Add a new MCP server (alias: a)
    #[command(alias = "a")]
    Add {
        /// Server name
        name: String,
        /// Command or URL for the MCP server
        command_or_url: String,
        /// MCP server type
        #[arg(long = "type", value_enum)]
        server_type: McpServerType,
        /// Environment variables (can be specified multiple times as KEY=VALUE)
        #[arg(short = 'e', long = "env", value_parser = parse_env_var)]
        env: Vec<(String, String)>,
    },
    /// Delete an MCP server configuration (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Server name
        name: String,
    },
    /// List all configured MCP servers (alias: l)
    #[command(alias = "l")]
    List,
    /// Stop an MCP server connection (alias: st)
    #[command(alias = "st")]
    Stop {
        /// Server name
        name: String,
    },
    /// List functions exposed by a running MCP server (alias: f)
    #[command(alias = "f")]
    Functions {
        /// Server name
        name: String,
    },
    /// Invoke a function from a running MCP server (alias: i)
    #[command(alias = "i")]
    Invoke {
        /// Server name
        name: String,
        /// Function name
        function: String,
        /// Function arguments
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum VectorCommands {
    /// List all vector databases (alias: l)
    #[command(alias = "l")]
    List,
    /// Delete a vector database (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Name of the vector database to delete
        name: String,
    },
    /// Show information about a vector database (alias: i)
    #[command(alias = "i")]
    Info {
        /// Name of the vector database
        name: String,
    },
}

#[derive(Subcommand)]
pub enum WebChatProxyCommands {
    /// List supported providers (alias: p)
    #[command(alias = "p")]
    Providers {
        #[command(subcommand)]
        command: Option<WebChatProxyProviderCommands>,
    },
    /// Start proxy server for a provider (alias: s)
    #[command(alias = "s")]
    Start {
        /// Provider name
        provider: String,
        /// Port to listen on
        #[arg(short = 'p', long = "port", default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long = "host", default_value = "127.0.0.1")]
        host: String,
        /// API key for authentication
        #[arg(short = 'k', long = "key")]
        key: Option<String>,
        /// Generate a random API key
        #[arg(short = 'g', long = "generate-key")]
        generate_key: bool,
        /// Run in daemon mode (background)
        #[arg(short = 'd', long = "daemon")]
        daemon: bool,
    },
    /// Stop proxy server for a provider
    Stop {
        /// Provider name
        provider: String,
    },
    /// List running proxy servers (alias: ps)
    #[command(alias = "ps")]
    List,
}

#[derive(Subcommand)]
pub enum WebChatProxyProviderCommands {
    /// List all supported providers (alias: l)
    #[command(alias = "l")]
    List,
    /// Set authentication for Kagi provider
    Kagi {
        #[command(subcommand)]
        command: WebChatProxyKagiCommands,
    },
}

#[derive(Subcommand)]
pub enum WebChatProxyKagiCommands {
    /// Set authentication token
    Auth {
        /// Authentication token
        token: Option<String>,
    },
    /// List available Kagi models (alias: m)
    #[command(alias = "m")]
    Models,
}

#[derive(Subcommand)]
pub enum SyncCommands {
    /// List supported cloud providers (alias: p)
    #[command(alias = "p")]
    Providers,
    /// Configure cloud provider settings (alias: c)
    #[command(alias = "c")]
    Configure {
        /// Cloud provider name (e.g., s3)
        provider: String,
        #[command(subcommand)]
        command: Option<ConfigureCommands>,
    },
    /// Sync configuration to cloud provider
    To {
        /// Cloud provider name (e.g., s3)
        provider: String,
        /// Encrypt files before uploading
        #[arg(short = 'e', long = "encrypted")]
        encrypted: bool,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Sync configuration from cloud provider
    From {
        /// Cloud provider name (e.g., s3)
        provider: String,
        /// Decrypt files after downloading
        #[arg(short = 'e', long = "encrypted")]
        encrypted: bool,
        /// Enable debug/verbose logging
        #[arg(short = 'd', long = "debug")]
        debug: bool,
        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigureCommands {
    /// Set up provider configuration (alias: s)
    #[command(alias = "s")]
    Setup,
    /// Show current provider configuration (alias: sh)
    #[command(alias = "sh")]
    Show,
    /// Remove provider configuration (alias: r)
    #[command(alias = "r")]
    Remove,
}

#[derive(Subcommand)]
pub enum SearchCommands {
    /// Manage search providers (alias: p)
    #[command(alias = "p")]
    Provider {
        #[command(subcommand)]
        command: SearchProviderCommands,
    },
    /// Perform a search query
    Query {
        /// Search provider to use
        provider: String,
        /// Search query
        query: String,
        /// Output format (json or md/markdown)
        #[arg(short = 'f', long = "format", default_value = "md")]
        format: String,
        /// Number of results to return
        #[arg(short = 'n', long = "count", default_value = "5")]
        count: usize,
    },
}

#[derive(Subcommand)]
pub enum SearchProviderCommands {
    /// Add a new search provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
        /// Provider API endpoint URL
        url: String,
    },
    /// Delete a search provider (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Provider name
        name: String,
    },
    /// Set header for a search provider (alias: s)
    #[command(alias = "s")]
    Set {
        /// Provider name
        provider: String,
        /// Header name
        header_name: String,
        /// Header value
        header_value: String,
    },
    /// List all search providers (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum McpServerType {
    /// Standard I/O based MCP server
    Stdio,
    /// Server-Sent Events MCP server
    Sse,
    /// Streamable HTTP MCP server
    Streamable,
}


// Helper function to parse environment variable KEY=VALUE pairs
fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid environment variable format: '{}'. Expected 'KEY=VALUE'",
            s
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

// Helper function to extract code blocks from markdown text
fn extract_code_blocks(text: &str) -> Vec<String> {
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_block = String::new();

    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                if !current_block.trim().is_empty() {
                    code_blocks.push(current_block.trim().to_string());
                }
                current_block.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
            }
        } else if in_code_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    // Handle case where code block doesn't end properly
    if in_code_block && !current_block.trim().is_empty() {
        code_blocks.push(current_block.trim().to_string());
    }

    code_blocks
}

// Provider command handlers
pub async fn handle_provider_command(command: ProviderCommands) -> Result<()> {
    match command {
        ProviderCommands::Install { name, force } => {
            let installer = ProviderInstaller::new()?;
            installer.install_provider(&name, force).await?;
        }
        ProviderCommands::Upgrade { name } => {
            let installer = ProviderInstaller::new()?;
            if let Some(provider_name) = name {
                installer.update_provider(&provider_name).await?;
            } else {
                installer.update_all_providers().await?;
            }
        }
        ProviderCommands::Uninstall { name } => {
            let installer = ProviderInstaller::new()?;
            installer.uninstall_provider(&name)?;
        }
        ProviderCommands::Available { official, tag } => {
            let installer = ProviderInstaller::new()?;
            let providers = installer.list_available().await?;
            
            println!("\n{}", "Available Providers:".bold().blue());
            
            let mut displayed_count = 0;
            for (id, metadata) in providers {
                // Apply filters
                if official && !metadata.official {
                    continue;
                }
                if let Some(ref filter_tag) = tag {
                    if !metadata.tags.contains(filter_tag) {
                        continue;
                    }
                }
                
                displayed_count += 1;
                
                print!("  {} {} - {}", "•".blue(), id.bold(), metadata.name);
                
                if metadata.official {
                    print!(" {}", "✓ official".green());
                }
                
                if !metadata.tags.is_empty() {
                    print!(" [{}]", metadata.tags.join(", ").dimmed());
                }
                
                println!("\n    {}", metadata.description.dimmed());
                
                // Show auth type
                let auth_str = match metadata.auth_type {
                    AuthType::ApiKey => "API Key",
                    AuthType::ServiceAccount => "Service Account",
                    AuthType::OAuth => "OAuth",
                    AuthType::Token => "Token",
                    AuthType::Headers => "Custom Headers",
                    AuthType::None => "None",
                };
                println!("    Auth: {}", auth_str.yellow());
            }
            
            if displayed_count == 0 {
                if official {
                    println!("No official providers found.");
                } else if tag.is_some() {
                    println!("No providers found with the specified tag.");
                } else {
                    println!("No providers available.");
                }
            } else {
                println!("\n{} Use 'lc providers install <name>' to install a provider", "💡".yellow());
            }
        }
        ProviderCommands::Add {
            name,
            url,
            models_path,
            chat_path,
        } => {
            let mut config = config::Config::load()?;
            config.add_provider_with_paths(name.clone(), url, models_path, chat_path)?;
            config.save()?;
            println!("{} Provider '{}' added successfully", "✓".green(), name);
        }
        ProviderCommands::Update { name, url } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            config.add_provider(name.clone(), url)?; // add_provider also updates
            config.save()?;
            println!("{} Provider '{}' updated successfully", "✓".green(), name);
        }
        ProviderCommands::Remove { name } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            config.providers.remove(&name);
            config.save()?;
            println!("{} Provider '{}' removed successfully", "✓".green(), name);
        }
        ProviderCommands::List => {
            let config = config::Config::load()?;
            if config.providers.is_empty() {
                println!("No providers configured.");
                return Ok(());
            }

            println!("\n{}", "Configured Providers:".bold().blue());

            // Load keys config to check authentication status
            let keys = crate::keys::KeysConfig::load().unwrap_or_else(|_| crate::keys::KeysConfig::new());

            // Sort providers by name for easier lookup
            let mut sorted_providers: Vec<_> = config.providers.iter().collect();
            sorted_providers.sort_by(|a, b| a.0.cmp(b.0));

            for (name, provider_config) in sorted_providers {
                // Check if provider has authentication in keys.toml
                let has_key = keys.has_auth(name);
                let key_status = if has_key { "✓".green() } else { "✗".red() };
                println!(
                    "  {} {} - {} (API Key: {})",
                    "•".blue(),
                    name.bold(),
                    provider_config.endpoint,
                    key_status
                );
            }
        }
        ProviderCommands::Models { name, refresh } => {
            debug_log!(
                "Handling provider models command for '{}', refresh: {}",
                name,
                refresh
            );

            let config = config::Config::load()?;
            let _provider_config = config.get_provider(&name)?;

            debug_log!("Provider '{}' found in config", name);

            // Use unified cache system
            match crate::unified_cache::UnifiedCache::fetch_and_cache_provider_models(
                &name, refresh,
            )
            .await
            {
                Ok(models) => {
                    debug_log!(
                        "Successfully fetched {} models for provider '{}'",
                        models.len(),
                        name
                    );
                    println!("\n{} Available models:", "Models:".bold());
                    display_provider_models(&models)?;
                }
                Err(e) => {
                    debug_log!("Unified cache failed for provider '{}': {}", name, e);
                    eprintln!("Error fetching models from provider '{}': {}", name, e);

                    // Fallback to basic listing if unified cache fails
                    debug_log!(
                        "Attempting fallback to basic client listing for provider '{}'",
                        name
                    );
                    let mut config_mut = config.clone();
                    match chat::create_authenticated_client(&mut config_mut, &name).await {
                        Ok(client) => {
                            debug_log!("Created fallback client for provider '{}'", name);
                            // Save config if tokens were updated
                            if config_mut.get_cached_token(&name) != config.get_cached_token(&name)
                            {
                                debug_log!("Tokens updated for provider '{}', saving config", name);
                                config_mut.save()?;
                            }

                            match client.list_models().await {
                                Ok(models) => {
                                    debug_log!(
                                        "Fallback client returned {} models for provider '{}'",
                                        models.len(),
                                        name
                                    );
                                    println!(
                                        "\n{} Available models (basic listing):",
                                        "Models:".bold()
                                    );
                                    for model in models {
                                        println!("  • {}", model.id);
                                    }
                                }
                                Err(e2) => {
                                    debug_log!(
                                        "Fallback client failed for provider '{}': {}",
                                        name,
                                        e2
                                    );
                                    anyhow::bail!("Failed to fetch models: {}", e2);
                                }
                            }
                        }
                        Err(e2) => {
                            debug_log!(
                                "Failed to create fallback client for provider '{}': {}",
                                name,
                                e2
                            );
                            anyhow::bail!("Failed to create client: {}", e2);
                        }
                    }
                }
            }
        }
        ProviderCommands::Headers { provider, command } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }

            match command {
                HeaderCommands::Add { name, value } => {
                    config.add_header(provider.clone(), name.clone(), value.clone())?;
                    config.save()?;
                    println!(
                        "{} Header '{}' added to provider '{}'",
                        "✓".green(),
                        name,
                        provider
                    );
                }
                HeaderCommands::Delete { name } => {
                    config.remove_header(provider.clone(), name.clone())?;
                    config.save()?;
                    println!(
                        "{} Header '{}' removed from provider '{}'",
                        "✓".green(),
                        name,
                        provider
                    );
                }
                HeaderCommands::List => {
                    let headers = config.list_headers(&provider)?;
                    if headers.is_empty() {
                        println!("No custom headers configured for provider '{}'", provider);
                    } else {
                        println!(
                            "\n{} Custom headers for provider '{}':",
                            "Headers:".bold().blue(),
                            provider
                        );
                        for (name, value) in headers {
                            println!("  {} {}: {}", "•".blue(), name.bold(), value);
                        }
                    }
                }
            }
        }
        ProviderCommands::TokenUrl { provider, url } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }

            config.set_token_url(provider.clone(), url.clone())?;
            config.save()?;
            println!("{} Token URL set for provider '{}'", "✓".green(), provider);
        }
        ProviderCommands::Vars { provider, command } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }
            match command {
                ProviderVarsCommands::Set { key, value } => {
                    config.set_provider_var(&provider, &key, &value)?;
                    config.save()?;
                    println!(
                        "{} Set var '{}'='{}' for provider '{}'",
                        "✓".green(),
                        key,
                        value,
                        provider
                    );
                }
                ProviderVarsCommands::Get { key } => {
                    match config.get_provider_var(&provider, &key) {
                        Some(val) => println!("{}", val),
                        None => anyhow::bail!("Var '{}' not set for provider '{}'", key, provider),
                    }
                }
                ProviderVarsCommands::List => {
                    let vars = config.list_provider_vars(&provider)?;
                    if vars.is_empty() {
                        println!("No vars set for provider '{}'", provider);
                    } else {
                        println!(
                            "\n{} Vars for provider '{}':",
                            "Vars:".bold().blue(),
                            provider
                        );
                        for (k, v) in vars {
                            println!("  {} {} = {}", "•".blue(), k.bold(), v);
                        }
                    }
                }
            }
        }
        ProviderCommands::Paths { provider, command } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }
            match command {
                ProviderPathCommands::Add {
                    models_path,
                    chat_path,
                    images_path,
                    embeddings_path,
                } => {
                    let mut updated = false;
                    if let Some(path) = models_path {
                        config.set_provider_models_path(&provider, &path)?;
                        println!(
                            "{} Models path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = chat_path {
                        config.set_provider_chat_path(&provider, &path)?;
                        println!(
                            "{} Chat path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = images_path {
                        config.set_provider_images_path(&provider, &path)?;
                        println!(
                            "{} Images path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = embeddings_path {
                        config.set_provider_embeddings_path(&provider, &path)?;
                        println!(
                            "{} Embeddings path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if !updated {
                        anyhow::bail!("No paths specified. Use -m, -c, -i, or -e to set paths.");
                    }
                    config.save()?;
                }
                ProviderPathCommands::Delete {
                    models,
                    chat,
                    images,
                    embeddings,
                } => {
                    let mut updated = false;
                    if models {
                        config.reset_provider_models_path(&provider)?;
                        println!(
                            "{} Models path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if chat {
                        config.reset_provider_chat_path(&provider)?;
                        println!(
                            "{} Chat path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if images {
                        config.reset_provider_images_path(&provider)?;
                        println!(
                            "{} Images path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if embeddings {
                        config.reset_provider_embeddings_path(&provider)?;
                        println!(
                            "{} Embeddings path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if !updated {
                        anyhow::bail!("No paths specified for deletion. Use -m, -c, -i, or -e to delete paths.");
                    }
                    config.save()?;
                }
                ProviderPathCommands::List => {
                    let paths = config.list_provider_paths(&provider)?;
                    println!(
                        "\n{} API paths for provider '{}':",
                        "Paths:".bold().blue(),
                        provider
                    );
                    println!("  {} Models: {}", "•".blue(), paths.models_path.bold());
                    println!("  {} Chat: {}", "•".blue(), paths.chat_path.bold());
                    if let Some(ref images_path) = paths.images_path {
                        println!("  {} Images: {}", "•".blue(), images_path.bold());
                    } else {
                        println!("  {} Images: {}", "•".blue(), "not set".dimmed());
                    }
                    if let Some(ref embeddings_path) = paths.embeddings_path {
                        println!("  {} Embeddings: {}", "•".blue(), embeddings_path.bold());
                    } else {
                        println!("  {} Embeddings: {}", "•".blue(), "not set".dimmed());
                    }
                }
            }
        }
    }
    Ok(())
}

// Key command handlers
pub async fn handle_key_command(command: KeyCommands) -> Result<()> {
    match command {
        KeyCommands::Add { name } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&name) {
                anyhow::bail!(
                    "Provider '{}' not found. Add it first with 'lc providers add'",
                    name
                );
            }

            // Detect Google SA JWT providers and prompt for Service Account JSON
            let provider_cfg = config.get_provider(&name)?;
            let is_google_sa = provider_cfg.auth_type.as_deref() == Some("google_sa_jwt")
                || provider_cfg.endpoint.contains("aiplatform.googleapis.com");

            if is_google_sa {
                println!(
                    "Detected Google Vertex AI provider. Please provide the Service Account JSON."
                );
                println!("Options:");
                println!("  1. Paste the base64 version directly (ex: cat sa.json | base64)");
                println!("  2. Provide the path to the JSON file (ex: /path/to/sa.json)");
                print!("Base64 Service Account JSON or file path for {}: ", name);
                io::stdout().flush()?;

                // Use regular stdin reading instead of rpassword for large inputs
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();

                let sa_json = if input.starts_with('/') || input.ends_with(".json") {
                    // Treat as file path
                    match std::fs::read_to_string(input) {
                        Ok(file_content) => file_content,
                        Err(e) => {
                            anyhow::bail!("Failed to read service account file '{}': {}", input, e)
                        }
                    }
                } else {
                    // Treat as base64 input - clean whitespace and newlines
                    let sa_json_b64 = input
                        .trim()
                        .replace("\n", "")
                        .replace("\r", "")
                        .replace(" ", "");

                    // Decode base64
                    use base64::{engine::general_purpose, Engine as _};
                    match general_purpose::STANDARD.decode(&sa_json_b64) {
                        Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                            Ok(json_str) => json_str,
                            Err(_) => anyhow::bail!("Invalid UTF-8 in decoded base64 data"),
                        },
                        Err(_) => anyhow::bail!("Invalid base64 format"),
                    }
                };

                // Minimal validation
                let parsed: serde_json::Value = serde_json::from_str(&sa_json)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
                let sa_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let client_email = parsed
                    .get("client_email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let private_key = parsed
                    .get("private_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if sa_type != "service_account" {
                    anyhow::bail!("Service Account JSON must have \"type\": \"service_account\"");
                }
                if client_email.is_empty() {
                    anyhow::bail!("Service Account JSON missing 'client_email'");
                }
                if private_key.is_empty() {
                    anyhow::bail!("Service Account JSON missing 'private_key'");
                }

                // Store full JSON string in api_key field (used by JWT mint flow)
                config.set_api_key(name.clone(), sa_json)?;
                config.save()?;
                println!(
                    "{} Service Account stored for provider '{}'",
                    "✓".green(),
                    name
                );
            } else {
                print!("Enter API key for {}: ", name);
                io::stdout().flush()?;
                let key = read_password()?;

                config.set_api_key(name.clone(), key)?;
                config.save()?;
                println!("{} API key set for provider '{}'", "✓".green(), name);
            }
        }
        KeyCommands::Get { name } => {
            let config = config::Config::load()?;

            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }

            let provider_config = config.get_provider(&name)?;
            if let Some(api_key) = &provider_config.api_key {
                println!("{}", api_key);
            } else {
                anyhow::bail!("No API key configured for provider '{}'", name);
            }
        }
        KeyCommands::List => {
            let config = config::Config::load()?;
            if config.providers.is_empty() {
                println!("No providers configured.");
                return Ok(());
            }

            println!("\n{}", "API Key Status:".bold().blue());
            for (name, provider_config) in &config.providers {
                let status = if provider_config.api_key.is_some() {
                    "✓ Configured".green()
                } else {
                    "✗ Missing".red()
                };
                println!("  {} {} - {}", "•".blue(), name.bold(), status);
            }
        }
        KeyCommands::Remove { name } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }

            if let Some(provider_config) = config.providers.get_mut(&name) {
                provider_config.api_key = None;
            }
            config.save()?;
            println!("{} API key removed for provider '{}'", "✓".green(), name);
        }
    }
    Ok(())
}

// Log command handlers
pub async fn handle_log_command(command: LogCommands) -> Result<()> {
    let db = database::Database::new()?;

    match command {
        LogCommands::Show { minimal } => {
            let entries = db.get_all_logs()?;

            if entries.is_empty() {
                println!("No chat logs found.");
                return Ok(());
            }

            if minimal {
                use tabled::{Table, Tabled};

                #[derive(Tabled)]
                struct LogEntry {
                    #[tabled(rename = "Chat ID")]
                    chat_id: String,
                    #[tabled(rename = "Model")]
                    model: String,
                    #[tabled(rename = "Question")]
                    question: String,
                    #[tabled(rename = "Time")]
                    time: String,
                }

                let table_data: Vec<LogEntry> = entries
                    .into_iter()
                    .map(|entry| LogEntry {
                        chat_id: entry.chat_id[..8].to_string(),
                        model: entry.model,
                        question: if entry.question.len() > 50 {
                            format!("{}...", &entry.question[..50])
                        } else {
                            entry.question
                        },
                        time: entry.timestamp.format("%m-%d %H:%M").to_string(),
                    })
                    .collect();

                let table = Table::new(table_data);
                println!("{}", table);
            } else {
                println!("\n{}", "Chat Logs:".bold().blue());

                for entry in entries {
                    println!(
                        "\n{} {} ({})",
                        "Session:".bold(),
                        &entry.chat_id[..8],
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!("{} {}", "Model:".bold(), entry.model);

                    // Show token usage if available
                    if let (Some(input_tokens), Some(output_tokens)) =
                        (entry.input_tokens, entry.output_tokens)
                    {
                        println!(
                            "{} {} input + {} output = {} total tokens",
                            "Tokens:".bold(),
                            input_tokens,
                            output_tokens,
                            input_tokens + output_tokens
                        );
                    }

                    println!("{} {}", "Q:".yellow(), entry.question);
                    println!(
                        "{} {}",
                        "A:".green(),
                        if entry.response.len() > 200 {
                            format!("{}...", &entry.response[..200])
                        } else {
                            entry.response
                        }
                    );
                    println!("{}", "─".repeat(80).dimmed());
                }
            }
        }
        LogCommands::Recent { command, count } => {
            match command {
                Some(RecentCommands::Answer { command }) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        match command {
                            Some(AnswerCommands::Code) => {
                                let code_blocks = extract_code_blocks(&entry.response);
                                if code_blocks.is_empty() {
                                    anyhow::bail!("No code blocks found in the last answer");
                                } else {
                                    for block in code_blocks {
                                        println!("{}", block);
                                    }
                                }
                            }
                            None => {
                                println!("{}", entry.response);
                            }
                        }
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Question) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.question);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Model) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.model);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Session) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.chat_id);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                None => {
                    // Default behavior - show recent logs
                    let mut entries = db.get_all_logs()?;
                    entries.truncate(count);

                    if entries.is_empty() {
                        println!("No recent logs found.");
                        return Ok(());
                    }

                    println!(
                        "\n{} (showing {} entries)",
                        "Recent Logs:".bold().blue(),
                        entries.len()
                    );

                    for entry in entries {
                        println!(
                            "\n{} {} ({})",
                            "Session:".bold(),
                            &entry.chat_id[..8],
                            entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!("{} {}", "Model:".bold(), entry.model);

                        // Show token usage if available
                        if let (Some(input_tokens), Some(output_tokens)) =
                            (entry.input_tokens, entry.output_tokens)
                        {
                            println!(
                                "{} {} input + {} output = {} total tokens",
                                "Tokens:".bold(),
                                input_tokens,
                                output_tokens,
                                input_tokens + output_tokens
                            );
                        }

                        println!("{} {}", "Q:".yellow(), entry.question);
                        println!(
                            "{} {}",
                            "A:".green(),
                            if entry.response.len() > 150 {
                                format!("{}...", &entry.response[..150])
                            } else {
                                entry.response
                            }
                        );
                        println!("{}", "─".repeat(60).dimmed());
                    }
                }
            }
        }
        LogCommands::Current => {
            if let Some(session_id) = db.get_current_session_id()? {
                let history = db.get_chat_history(&session_id)?;

                println!("\n{} {}", "Current Session:".bold().blue(), session_id);
                println!("{} {} messages", "Messages:".bold(), history.len());

                for (i, entry) in history.iter().enumerate() {
                    println!(
                        "\n{} {} ({})",
                        format!("Message {}:", i + 1).bold(),
                        entry.model,
                        entry.timestamp.format("%H:%M:%S")
                    );
                    println!("{} {}", "Q:".yellow(), entry.question);
                    println!(
                        "{} {}",
                        "A:".green(),
                        if entry.response.len() > 100 {
                            format!("{}...", &entry.response[..100])
                        } else {
                            entry.response.clone()
                        }
                    );
                }
            } else {
                println!("No current session found.");
            }
        }
        LogCommands::Stats => {
            let stats = db.get_stats()?;

            println!("\n{}", "Database Statistics:".bold().blue());
            println!();

            // Basic stats
            println!("{} {}", "Total Entries:".bold(), stats.total_entries);
            println!("{} {}", "Unique Sessions:".bold(), stats.unique_sessions);

            // File size formatting
            let file_size_str = if stats.file_size_bytes < 1024 {
                format!("{} bytes", stats.file_size_bytes)
            } else if stats.file_size_bytes < 1024 * 1024 {
                format!("{:.1} KB", stats.file_size_bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", stats.file_size_bytes as f64 / (1024.0 * 1024.0))
            };
            println!("{} {}", "Database Size:".bold(), file_size_str);

            // Date range
            if let Some((earliest, latest)) = stats.date_range {
                println!(
                    "{} {} to {}",
                    "Date Range:".bold(),
                    earliest.format("%Y-%m-%d %H:%M:%S"),
                    latest.format("%Y-%m-%d %H:%M:%S")
                );
            } else {
                println!("{} {}", "Date Range:".bold(), "No entries".dimmed());
            }

            // Model usage
            if !stats.model_usage.is_empty() {
                println!("\n{}", "Model Usage:".bold().blue());
                for (model, count) in stats.model_usage {
                    let percentage = if stats.total_entries > 0 {
                        (count as f64 / stats.total_entries as f64) * 100.0
                    } else {
                        0.0
                    };
                    println!(
                        "  {} {} ({} - {:.1}%)",
                        "•".blue(),
                        model.bold(),
                        count,
                        percentage
                    );
                }
            }
        }
        LogCommands::Purge {
            yes,
            older_than_days,
            keep_recent,
            max_size_mb,
        } => {
            // Check if any specific purge options are provided
            let has_specific_options =
                older_than_days.is_some() || keep_recent.is_some() || max_size_mb.is_some();

            if has_specific_options {
                // Smart purge with specific options
                let deleted_count = db.smart_purge(older_than_days, keep_recent, max_size_mb)?;

                if deleted_count > 0 {
                    println!("{} Purged {} log entries", "✓".green(), deleted_count);

                    if let Some(days) = older_than_days {
                        println!("  - Removed entries older than {} days", days);
                    }
                    if let Some(count) = keep_recent {
                        println!("  - Kept only the {} most recent entries", count);
                    }
                    if let Some(size) = max_size_mb {
                        println!("  - Enforced maximum database size of {} MB", size);
                    }
                } else {
                    println!("{} No logs needed to be purged", "ℹ️".blue());
                }
            } else {
                // Full purge (existing behavior)
                if !yes {
                    print!(
                        "Are you sure you want to purge all logs? This cannot be undone. (y/N): "
                    );
                    // Deliberately flush stdout to ensure prompt appears before user input
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    if !input.trim().to_lowercase().starts_with('y') {
                        println!("Purge cancelled.");
                        return Ok(());
                    }
                }

                db.purge_all_logs()?;
                println!("{} All logs purged successfully", "✓".green());
            }
        }
    }
    Ok(())
}

// Config command handlers
pub async fn handle_config_command(command: Option<ConfigCommands>) -> Result<()> {
    match command {
        Some(ConfigCommands::Set { command }) => match command {
            SetCommands::Provider { name } => {
                let mut config = config::Config::load()?;

                if !config.has_provider(&name) {
                    anyhow::bail!(
                        "Provider '{}' not found. Add it first with 'lc providers add'",
                        name
                    );
                }

                config.default_provider = Some(name.clone());
                config.save()?;
                println!("{} Default provider set to '{}'", "✓".green(), name);
            }
            SetCommands::Model { name } => {
                let mut config = config::Config::load()?;
                config.default_model = Some(name.clone());
                config.save()?;
                println!("{} Default model set to '{}'", "✓".green(), name);
            }
            SetCommands::SystemPrompt { prompt } => {
                let mut config = config::Config::load()?;
                let resolved_prompt = config.resolve_template_or_prompt(&prompt);
                config.system_prompt = Some(resolved_prompt);
                config.save()?;
                println!("{} System prompt set", "✓".green());
            }
            SetCommands::MaxTokens { value } => {
                let mut config = config::Config::load()?;
                let parsed_value = config::Config::parse_max_tokens(&value)?;
                config.max_tokens = Some(parsed_value);
                config.save()?;
                println!("{} Max tokens set to {}", "✓".green(), parsed_value);
            }
            SetCommands::Temperature { value } => {
                let mut config = config::Config::load()?;
                let parsed_value = config::Config::parse_temperature(&value)?;
                config.temperature = Some(parsed_value);
                config.save()?;
                println!("{} Temperature set to {}", "✓".green(), parsed_value);
            }
            SetCommands::Search { name } => {
                let mut search_config = crate::search::SearchConfig::load()?;

                if !search_config.has_provider(&name) {
                    anyhow::bail!("Search provider '{}' not found. Add it first with 'lc search provider add'", name);
                }

                search_config.set_default_provider(name.clone())?;
                search_config.save()?;
                println!("{} Default search provider set to '{}'", "✓".green(), name);
            }
            SetCommands::Stream { value } => {
                let mut config = config::Config::load()?;
                let stream_value = match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => anyhow::bail!("Invalid stream value '{}'. Use 'true' or 'false'", value),
                };
                config.stream = Some(stream_value);
                config.save()?;
                println!("{} Streaming mode set to {}", "✓".green(), stream_value);
            }
        },
        Some(ConfigCommands::Get { command }) => {
            let config = config::Config::load()?;
            match command {
                GetCommands::Provider => {
                    if let Some(provider) = &config.default_provider {
                        println!("{}", provider);
                    } else {
                        anyhow::bail!("No default provider configured");
                    }
                }
                GetCommands::Model => {
                    if let Some(model) = &config.default_model {
                        println!("{}", model);
                    } else {
                        anyhow::bail!("No default model configured");
                    }
                }
                GetCommands::SystemPrompt => {
                    if let Some(system_prompt) = &config.system_prompt {
                        println!("{}", system_prompt);
                    } else {
                        anyhow::bail!("No system prompt configured");
                    }
                }
                GetCommands::MaxTokens => {
                    if let Some(max_tokens) = &config.max_tokens {
                        println!("{}", max_tokens);
                    } else {
                        anyhow::bail!("No max tokens configured");
                    }
                }
                GetCommands::Temperature => {
                    if let Some(temperature) = &config.temperature {
                        println!("{}", temperature);
                    } else {
                        anyhow::bail!("No temperature configured");
                    }
                }
                GetCommands::Search => {
                    let search_config = crate::search::SearchConfig::load()?;
                    if let Some(provider) = search_config.get_default_provider() {
                        println!("{}", provider);
                    } else {
                        anyhow::bail!("No default search provider configured");
                    }
                }
                GetCommands::Stream => {
                    if let Some(stream) = &config.stream {
                        println!("{}", stream);
                    } else {
                        anyhow::bail!("No streaming mode configured");
                    }
                }
            }
        }
        Some(ConfigCommands::Delete { command }) => {
            let mut config = config::Config::load()?;
            match command {
                DeleteCommands::Provider => {
                    if config.default_provider.is_some() {
                        config.default_provider = None;
                        config.save()?;
                        println!("{} Default provider deleted", "✓".green());
                    } else {
                        anyhow::bail!("No default provider configured to delete");
                    }
                }
                DeleteCommands::Model => {
                    if config.default_model.is_some() {
                        config.default_model = None;
                        config.save()?;
                        println!("{} Default model deleted", "✓".green());
                    } else {
                        anyhow::bail!("No default model configured to delete");
                    }
                }
                DeleteCommands::SystemPrompt => {
                    if config.system_prompt.is_some() {
                        config.system_prompt = None;
                        config.save()?;
                        println!("{} System prompt deleted", "✓".green());
                    } else {
                        anyhow::bail!("No system prompt configured to delete");
                    }
                }
                DeleteCommands::MaxTokens => {
                    if config.max_tokens.is_some() {
                        config.max_tokens = None;
                        config.save()?;
                        println!("{} Max tokens deleted", "✓".green());
                    } else {
                        anyhow::bail!("No max tokens configured to delete");
                    }
                }
                DeleteCommands::Temperature => {
                    if config.temperature.is_some() {
                        config.temperature = None;
                        config.save()?;
                        println!("{} Temperature deleted", "✓".green());
                    } else {
                        anyhow::bail!("No temperature configured to delete");
                    }
                }
                DeleteCommands::Search => {
                    let mut search_config = crate::search::SearchConfig::load()?;
                    if search_config.get_default_provider().is_some() {
                        search_config.set_default_provider(String::new())?;
                        search_config.save()?;
                        println!("{} Default search provider deleted", "✓".green());
                    } else {
                        anyhow::bail!("No default search provider configured to delete");
                    }
                }
                DeleteCommands::Stream => {
                    let mut config = config::Config::load()?;
                    if config.stream.is_some() {
                        config.stream = None;
                        config.save()?;
                        println!("{} Streaming mode deleted", "✓".green());
                    } else {
                        anyhow::bail!("No streaming mode configured to delete");
                    }
                }
            }
        }
        Some(ConfigCommands::Path) => {
            let config_dir = config::Config::config_dir()?;
            println!("\n{}", "Configuration Directory:".bold().blue());
            println!("{}", config_dir.display());
            println!("\n{}", "Files:".bold().blue());
            println!("  {} config.toml", "•".blue());
            println!("  {} logs.db (synced to cloud)", "•".blue());
            println!("\n{}", "Database Management:".bold().blue());
            println!(
                "  {} Purge old logs: {}",
                "•".blue(),
                "lc logs purge --older-than-days 30".dimmed()
            );
            println!(
                "  {} Keep recent logs: {}",
                "•".blue(),
                "lc logs purge --keep-recent 1000".dimmed()
            );
            println!(
                "  {} Size-based purge: {}",
                "•".blue(),
                "lc logs purge --max-size-mb 50".dimmed()
            );
        }
        None => {
            // Show current configuration with enhanced model metadata
            let config = config::Config::load()?;
            println!("\n{}", "Current Configuration:".bold().blue());

            if let Some(provider) = &config.default_provider {
                println!("provider {}", provider);
            } else {
                println!("provider {}", "not set".dimmed());
            }

            if let Some(model) = &config.default_model {
                // Try to find model metadata to display rich information
                if let Some(provider) = &config.default_provider {
                    match load_provider_enhanced_models(provider).await {
                        Ok(models) => {
                            // Find the specific model
                            if let Some(model_metadata) = models.iter().find(|m| m.id == *model) {
                                // Display model with metadata
                                let _model_info = vec![model.clone()];

                                // Build capability indicators
                                let mut capabilities = Vec::new();
                                if model_metadata.supports_tools
                                    || model_metadata.supports_function_calling
                                {
                                    capabilities.push("🔧 tools".blue());
                                }
                                if model_metadata.supports_vision {
                                    capabilities.push("👁 vision".magenta());
                                }
                                if model_metadata.supports_audio {
                                    capabilities.push("🔊 audio".yellow());
                                }
                                if model_metadata.supports_reasoning {
                                    capabilities.push("🧠 reasoning".cyan());
                                }
                                if model_metadata.supports_code {
                                    capabilities.push("💻 code".green());
                                }

                                // Build context and pricing info
                                let mut info_parts = Vec::new();
                                if let Some(ctx) = model_metadata.context_length {
                                    if ctx >= 1000000 {
                                        info_parts.push(format!("{}m ctx", ctx / 1000000));
                                    } else if ctx >= 1000 {
                                        info_parts.push(format!("{}k ctx", ctx / 1000));
                                    } else {
                                        info_parts.push(format!("{} ctx", ctx));
                                    }
                                }
                                if let Some(input_price) = model_metadata.input_price_per_m {
                                    info_parts.push(format!("${:.2}/M in", input_price));
                                }
                                if let Some(output_price) = model_metadata.output_price_per_m {
                                    info_parts.push(format!("${:.2}/M out", output_price));
                                }

                                // Display model name with metadata
                                let model_display =
                                    if let Some(ref display_name) = model_metadata.display_name {
                                        if display_name != &model_metadata.id {
                                            format!("{} ({})", model, display_name)
                                        } else {
                                            model.clone()
                                        }
                                    } else {
                                        model.clone()
                                    };

                                print!("model {}", model_display);

                                if !capabilities.is_empty() {
                                    let capability_strings: Vec<String> =
                                        capabilities.iter().map(|c| c.to_string()).collect();
                                    print!(" [{}]", capability_strings.join(" "));
                                }

                                if !info_parts.is_empty() {
                                    print!(" ({})", info_parts.join(", ").dimmed());
                                }

                                println!();
                            } else {
                                // Model not found in metadata, show basic info
                                println!("model {}", model);
                            }
                        }
                        Err(_) => {
                            // Failed to load metadata, show basic info
                            println!("model {}", model);
                        }
                    }
                } else {
                    // No provider set, show basic info
                    println!("model {}", model);
                }
            } else {
                println!("model {}", "not set".dimmed());
            }

            if let Some(system_prompt) = &config.system_prompt {
                println!("system_prompt {}", system_prompt);
            } else {
                println!("system_prompt {}", "not set".dimmed());
            }

            if let Some(max_tokens) = &config.max_tokens {
                println!("max_tokens {}", max_tokens);
            } else {
                println!("max_tokens {}", "not set".dimmed());
            }

            if let Some(temperature) = &config.temperature {
                println!("temperature {}", temperature);
            } else {
                println!("temperature {}", "not set".dimmed());
            }

            if let Some(stream) = &config.stream {
                println!("stream {}", stream);
            } else {
                println!("stream {}", "not set".dimmed());
            }
        }
    }
    Ok(())
}

// Helper function to resolve model and provider from various inputs
pub fn resolve_model_and_provider(
    config: &config::Config,
    provider_override: Option<String>,
    model_override: Option<String>,
) -> Result<(String, String)> {
    // Parse provider and model from model_override if it contains ":" or resolve alias
    // BUT if provider_override is already provided, treat model_override as literal
    let (final_provider_override, final_model_override) = if let Some(model) = &model_override {
        if provider_override.is_some() {
            // Provider is explicitly provided, treat model as literal (don't parse colons)
            (provider_override, model_override)
        } else if model.contains(':') {
            // No explicit provider, try to parse provider:model format
            let parts: Vec<&str> = model.splitn(2, ':').collect();
            if parts.len() == 2 {
                (Some(parts[0].to_string()), Some(parts[1].to_string()))
            } else {
                (provider_override, model_override)
            }
        } else {
            // Check if it's an alias
            if let Some(alias_target) = config.get_alias(model) {
                // Alias found, parse the target
                if alias_target.contains(':') {
                    let parts: Vec<&str> = alias_target.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        (Some(parts[0].to_string()), Some(parts[1].to_string()))
                    } else {
                        anyhow::bail!(
                            "Invalid alias target format: '{}'. Expected 'provider:model'",
                            alias_target
                        );
                    }
                } else {
                    anyhow::bail!(
                        "Invalid alias target format: '{}'. Expected 'provider:model'",
                        alias_target
                    );
                }
            } else {
                // Not an alias, treat as regular model name
                (provider_override, model_override)
            }
        }
    } else {
        (provider_override, model_override)
    };

    // Determine provider and model to use
    let provider_name = if let Some(provider) = final_provider_override {
        // Validate that the provider exists
        if !config.has_provider(&provider) {
            anyhow::bail!(
                "Provider '{}' not found. Add it first with 'lc providers add'",
                provider
            );
        }
        provider
    } else {
        config.default_provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default provider configured. Set one with 'lc config set provider <name>' or use -p flag"))?
            .clone()
    };

    let model_name = if let Some(model) = final_model_override {
        model
    } else {
        config.default_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default model configured. Set one with 'lc config set model <name>' or use -m flag"))?
            .clone()
    };

    Ok((provider_name, model_name))
}

// Helper function to read and format file contents
pub fn read_and_format_attachments(attachments: &[String]) -> Result<String> {
    if attachments.is_empty() {
        return Ok(String::new());
    }

    let mut formatted_content = String::new();

    for (i, file_path) in attachments.iter().enumerate() {
        if i > 0 {
            formatted_content.push_str("\n\n");
        }

        // Determine file extension for better formatting
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        formatted_content.push_str(&format!("=== File: {} ===\n", file_path));

        // Check if we have a specialized reader for this file type
        if let Some(reader) = readers::get_reader_for_extension(extension) {
            match reader.read_as_text(file_path) {
                Ok(content) => {
                    formatted_content.push_str(&content);
                }
                Err(e) => {
                    anyhow::bail!(
                        "Failed to read file '{}' with specialized reader: {}",
                        file_path,
                        e
                    );
                }
            }
        } else {
            // Fallback to regular text file reading for unsupported types
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    // Add language hint for code files
                    if !extension.is_empty() && is_code_file(extension) {
                        formatted_content.push_str(&format!("```{}\n{}\n```", extension, content));
                    } else {
                        formatted_content.push_str(&content);
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to read file '{}': {}", file_path, e);
                }
            }
        }
    }

    Ok(formatted_content)
}

// Helper function to determine if a file extension represents code
pub fn is_code_file(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "rs" | "py"
            | "js"
            | "ts"
            | "java"
            | "cpp"
            | "c"
            | "h"
            | "hpp"
            | "go"
            | "rb"
            | "php"
            | "swift"
            | "kt"
            | "scala"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "bat"
            | "cmd"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "xml"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "cfg"
            | "conf"
            | "sql"
            | "r"
            | "m"
            | "mm"
            | "pl"
            | "pm"
            | "lua"
            | "vim"
            | "dockerfile"
            | "makefile"
            | "cmake"
            | "gradle"
            | "maven"
    )
}

// Direct prompt handler
pub async fn handle_direct_prompt(
    prompt: String,
    provider_override: Option<String>,
    model_override: Option<String>,
    system_prompt_override: Option<String>,
    max_tokens_override: Option<String>,
    temperature_override: Option<String>,
    attachments: Vec<String>,
    images: Vec<String>,
    audio_files: Vec<String>,
    tools: Option<String>,
    vectordb: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    let config = config::Config::load()?;
    let db = database::Database::new()?;

    // Note: We don't enforce vision capability checks here as model metadata may be incomplete.
    // Let the API/model handle vision support validation and return appropriate errors if needed.

    // Read and format attachments
    let attachment_content = read_and_format_attachments(&attachments)?;

    // Process images if provided
    let processed_images = if !images.is_empty() {
        crate::image_utils::process_images(&images)?
    } else {
        Vec::new()
    };

    // Process audio files if provided - transcribe them and add to context
    let mut audio_transcriptions = String::new();
    if !audio_files.is_empty() {
        println!("{} Transcribing {} audio file(s)...", "🎤".blue(), audio_files.len());
        
        // Use the default whisper model for transcription
        let transcription_model = "whisper-1".to_string();
        
        // Find a provider that supports whisper (usually OpenAI)
        let transcription_provider = config.providers.iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("whisper")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| provider_override.clone().unwrap_or_else(|| "openai".to_string()));
        
        // Create a client for transcription
        let mut transcription_config = config.clone();
        let transcription_client = chat::create_authenticated_client(&mut transcription_config, &transcription_provider).await?;
        
        for (i, audio_file) in audio_files.iter().enumerate() {
            println!("  Processing audio file {}/{}: {}", i + 1, audio_files.len(), audio_file);
            
            // Process audio file
            let audio_data = if audio_file.starts_with("http://") || audio_file.starts_with("https://") {
                crate::audio_utils::process_audio_url(audio_file)?
            } else {
                crate::audio_utils::process_audio_file(std::path::Path::new(audio_file))?
            };
            
            // Create transcription request
            let transcription_request = crate::provider::AudioTranscriptionRequest {
                file: audio_data,
                model: transcription_model.clone(),
                language: None,
                prompt: None,
                response_format: Some("text".to_string()),
                temperature: None,
            };
            
            // Transcribe audio
            match transcription_client.transcribe_audio(&transcription_request).await {
                Ok(response) => {
                    if !audio_transcriptions.is_empty() {
                        audio_transcriptions.push_str("\n\n");
                    }
                    audio_transcriptions.push_str(&format!("=== Audio Transcription: {} ===\n{}", audio_file, response.text));
                    println!("  ✅ Transcribed successfully");
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to transcribe audio file '{}': {}", audio_file, e);
                }
            }
        }
        
        if !audio_transcriptions.is_empty() {
            println!("{} Audio transcription complete", "✅".green());
        }
    }

    // Combine prompt with attachments and audio transcriptions
    let final_prompt = {
        let mut combined = prompt.clone();
        
        if !attachment_content.is_empty() {
            combined.push_str("\n\n");
            combined.push_str(&attachment_content);
        }
        
        if !audio_transcriptions.is_empty() {
            combined.push_str("\n\n");
            combined.push_str(&audio_transcriptions);
        }
        
        combined
    };

    // Determine system prompt to use (CLI override takes precedence over config)
    let system_prompt = if let Some(override_prompt) = &system_prompt_override {
        Some(config.resolve_template_or_prompt(override_prompt))
    } else if let Some(config_prompt) = &config.system_prompt {
        Some(config.resolve_template_or_prompt(config_prompt))
    } else {
        None
    };
    let system_prompt = system_prompt.as_deref();

    // Determine max_tokens to use (CLI override takes precedence over config)
    let max_tokens = if let Some(override_tokens) = &max_tokens_override {
        Some(config::Config::parse_max_tokens(override_tokens)?)
    } else {
        config.max_tokens
    };

    // Determine temperature to use (CLI override takes precedence over config)
    let temperature = if let Some(override_temp) = &temperature_override {
        Some(config::Config::parse_temperature(override_temp)?)
    } else {
        config.temperature
    };

    // Fetch MCP tools if specified
    let (mcp_tools, mcp_server_names) = if let Some(tools_str) = &tools {
        fetch_mcp_tools(tools_str).await?
    } else {
        (None, Vec::new())
    };

    // Resolve provider and model
    let (provider_name, model_name) =
        resolve_model_and_provider(&config, provider_override, model_override)?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Generate a session ID for this direct prompt
    let session_id = uuid::Uuid::new_v4().to_string();
    db.set_current_session_id(&session_id)?;

    // RAG: Retrieve relevant context if database is specified
    let mut enhanced_prompt = final_prompt.clone();
    if let Some(ref db_name) = vectordb {
        match retrieve_rag_context(db_name, &final_prompt, &client, &model_name, &provider_name)
            .await
        {
            Ok(context) => {
                if !context.is_empty() {
                    enhanced_prompt = format!(
                        "Context from knowledge base:\n{}\n\nUser question: {}",
                        context, final_prompt
                    );
                    println!(
                        "{} Retrieved {} relevant context entries from '{}'",
                        "🧠".blue(),
                        context.lines().filter(|l| l.starts_with("- ")).count(),
                        db_name
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to retrieve RAG context: {}", e);
            }
        }
    }

    // Search integration: Add search results as context if --use-search is specified
    if let Some(search_spec) = use_search {
        match integrate_search_context(&search_spec, &prompt, &mut enhanced_prompt).await {
            Ok(search_performed) => {
                if search_performed {
                    println!("{} Search results integrated into context", "🔍".blue());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to integrate search results: {}", e);
            }
        }
    }

    // Determine if streaming should be used (CLI flag takes precedence over config)
    let use_streaming = stream || config.stream.unwrap_or(false);

    // Create the appropriate message based on whether images are included
    let messages = if !processed_images.is_empty() {
        // Create multimodal message with text and images
        let mut content_parts = vec![crate::provider::ContentPart::Text {
            text: enhanced_prompt.clone(),
        }];

        // Add each image as a content part
        for image_url in processed_images {
            content_parts.push(crate::provider::ContentPart::ImageUrl {
                image_url: crate::provider::ImageUrl {
                    url: image_url,
                    detail: Some("auto".to_string()),
                },
            });
        }

        vec![crate::provider::Message {
            role: "user".to_string(),
            content_type: crate::provider::MessageContent::Multimodal {
                content: content_parts,
            },
            tool_calls: None,
            tool_call_id: None,
        }]
    } else {
        // Regular text message
        vec![]
    };

    // Send the prompt
    if use_streaming {
        // Use streaming
        if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // For now, tools don't support streaming, fall back to regular
            print!("{}", "Thinking...".dimmed());
            // Deliberately flush stdout to show thinking indicator immediately
            io::stdout().flush()?;
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();

            // Use messages if we have images, otherwise use the text prompt
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_tool_execution_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    mcp_tools,
                    &server_refs,
                )
                .await
            } else {
                chat::send_chat_request_with_tool_execution(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    mcp_tools,
                    &server_refs,
                )
                .await
            };

            match result {
                Ok((response, input_tokens, output_tokens)) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{}", response);

                    // Save to database with token counts
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &model_name,
                        &prompt,
                        &response,
                        input_tokens,
                        output_tokens,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }
                }
                Err(e) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    anyhow::bail!("Error: {}", e);
                }
            }
        } else {
            // Use streaming chat - content is streamed directly to stdout
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_streaming_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_streaming(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok(_) => {
                    // Streaming completed successfully, add a newline
                    println!();

                    // For streaming, we save a placeholder since the actual response was streamed
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &model_name,
                        &prompt,
                        "[Streamed Response]",
                        None,
                        None,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }
                }
                Err(e) => {
                    anyhow::bail!("Error: {}", e);
                }
            }
        }
    } else {
        // Use regular non-streaming
        print!("{}", "Thinking...".dimmed());
        // Deliberately flush stdout to show thinking indicator immediately
        io::stdout().flush()?;

        let result = if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // Use tool execution loop when tools are available
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();
            if !messages.is_empty() {
                chat::send_chat_request_with_tool_execution_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    mcp_tools,
                    &server_refs,
                )
                .await
            } else {
                chat::send_chat_request_with_tool_execution(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    mcp_tools,
                    &server_refs,
                )
                .await
            }
        } else {
            // Use regular chat when no tools
            if !messages.is_empty() {
                chat::send_chat_request_with_validation_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_validation(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            }
        };

        match result {
            Ok((response, input_tokens, output_tokens)) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
                println!("{}", response);

                // Save to database with token counts (save original prompt for cleaner logs)
                if let Err(e) = db.save_chat_entry_with_tokens(
                    &session_id,
                    &model_name,
                    &prompt,
                    &response,
                    input_tokens,
                    output_tokens,
                ) {
                    eprintln!("Warning: Failed to save chat entry: {}", e);
                }
            }
            Err(e) => {
                print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                anyhow::bail!("Error: {}", e);
            }
        }
    }

    Ok(())
}

// Direct prompt handler for piped input (treats piped content as attachment)
pub async fn handle_direct_prompt_with_piped_input(
    piped_content: String,
    provider_override: Option<String>,
    model_override: Option<String>,
    system_prompt_override: Option<String>,
    max_tokens_override: Option<String>,
    temperature_override: Option<String>,
    attachments: Vec<String>,
    images: Vec<String>,
    audio_files: Vec<String>,
    tools: Option<String>,
    vectordb: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    // For piped input, we need to determine if there's a prompt in the arguments
    // Since we're called from main.rs when there's no prompt argument, we'll treat the piped content as both prompt and attachment
    // But we should provide a way to specify a prompt when piping content

    // For now, let's treat piped content as an attachment and ask for clarification
    let prompt = "Please analyze the following content:".to_string();

    // Create a temporary "attachment" from piped content
    let all_attachments = attachments;

    // Format piped content as an attachment
    let piped_attachment = format!("=== Piped Input ===\n{}", piped_content);

    let config = config::Config::load()?;
    let db = database::Database::new()?;

    // Read and format file attachments
    let file_attachment_content = read_and_format_attachments(&all_attachments)?;

    // Process audio files if provided - transcribe them and add to context
    let mut audio_transcriptions = String::new();
    if !audio_files.is_empty() {
        println!("{} Transcribing {} audio file(s)...", "🎤".blue(), audio_files.len());
        
        // Use the default whisper model for transcription
        let transcription_model = "whisper-1".to_string();
        
        // Find a provider that supports whisper (usually OpenAI)
        let transcription_provider = config.providers.iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("whisper")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| provider_override.clone().unwrap_or_else(|| "openai".to_string()));
        
        // Create a client for transcription
        let mut transcription_config = config.clone();
        let transcription_client = chat::create_authenticated_client(&mut transcription_config, &transcription_provider).await?;
        
        for (i, audio_file) in audio_files.iter().enumerate() {
            println!("  Processing audio file {}/{}: {}", i + 1, audio_files.len(), audio_file);
            
            // Process audio file
            let audio_data = if audio_file.starts_with("http://") || audio_file.starts_with("https://") {
                crate::audio_utils::process_audio_url(audio_file)?
            } else {
                crate::audio_utils::process_audio_file(std::path::Path::new(audio_file))?
            };
            
            // Create transcription request
            let transcription_request = crate::provider::AudioTranscriptionRequest {
                file: audio_data,
                model: transcription_model.clone(),
                language: None,
                prompt: None,
                response_format: Some("text".to_string()),
                temperature: None,
            };
            
            // Transcribe audio
            match transcription_client.transcribe_audio(&transcription_request).await {
                Ok(response) => {
                    if !audio_transcriptions.is_empty() {
                        audio_transcriptions.push_str("\n\n");
                    }
                    audio_transcriptions.push_str(&format!("=== Audio Transcription: {} ===\n{}", audio_file, response.text));
                    println!("  ✅ Transcribed successfully");
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to transcribe audio file '{}': {}", audio_file, e);
                }
            }
        }
        
        if !audio_transcriptions.is_empty() {
            println!("{} Audio transcription complete", "✅".green());
        }
    }

    // Combine prompt with piped content, file attachments, and audio transcriptions
    let final_prompt = {
        let mut combined = format!("{}\n\n{}", prompt, piped_attachment);
        
        if !file_attachment_content.is_empty() {
            combined.push_str("\n\n");
            combined.push_str(&file_attachment_content);
        }
        
        if !audio_transcriptions.is_empty() {
            combined.push_str("\n\n");
            combined.push_str(&audio_transcriptions);
        }
        
        combined
    };

    // Determine system prompt to use (CLI override takes precedence over config)
    let system_prompt = if let Some(override_prompt) = &system_prompt_override {
        Some(config.resolve_template_or_prompt(override_prompt))
    } else if let Some(config_prompt) = &config.system_prompt {
        Some(config.resolve_template_or_prompt(config_prompt))
    } else {
        None
    };
    let system_prompt = system_prompt.as_deref();

    // Determine max_tokens to use (CLI override takes precedence over config)
    let max_tokens = if let Some(override_tokens) = &max_tokens_override {
        Some(config::Config::parse_max_tokens(override_tokens)?)
    } else {
        config.max_tokens
    };

    // Determine temperature to use (CLI override takes precedence over config)
    let temperature = if let Some(override_temp) = &temperature_override {
        Some(config::Config::parse_temperature(override_temp)?)
    } else {
        config.temperature
    };

    // Fetch MCP tools if specified
    let (mcp_tools, mcp_server_names) = if let Some(tools_str) = &tools {
        fetch_mcp_tools(tools_str).await?
    } else {
        (None, Vec::new())
    };

    // Resolve provider and model
    let (provider_name, model_name) =
        resolve_model_and_provider(&config, provider_override, model_override)?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Generate a session ID for this direct prompt
    let session_id = uuid::Uuid::new_v4().to_string();
    db.set_current_session_id(&session_id)?;

    // RAG: Retrieve relevant context if database is specified
    let mut enhanced_prompt = final_prompt.clone();
    if let Some(ref db_name) = vectordb {
        match retrieve_rag_context(db_name, &final_prompt, &client, &model_name, &provider_name)
            .await
        {
            Ok(context) => {
                if !context.is_empty() {
                    enhanced_prompt = format!(
                        "Context from knowledge base:\n{}\n\nUser question: {}",
                        context, final_prompt
                    );
                    println!(
                        "{} Retrieved {} relevant context entries from '{}'",
                        "🧠".blue(),
                        context.lines().filter(|l| l.starts_with("- ")).count(),
                        db_name
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to retrieve RAG context: {}", e);
            }
        }
    }

    // Search integration: Add search results as context if --use-search is specified
    if let Some(search_spec) = use_search {
        match integrate_search_context(&search_spec, &prompt, &mut enhanced_prompt).await {
            Ok(search_performed) => {
                if search_performed {
                    println!("{} Search results integrated into context", "🔍".blue());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to integrate search results: {}", e);
            }
        }
    }

    // Determine if streaming should be used (CLI flag takes precedence over config)
    let use_streaming = stream || config.stream.unwrap_or(false);

    // Process images if provided
    let processed_images = if !images.is_empty() {
        crate::image_utils::process_images(&images)?
    } else {
        Vec::new()
    };

    // Create the appropriate message based on whether images are included
    let messages = if !processed_images.is_empty() {
        // Create multimodal message with text and images
        let mut content_parts = vec![crate::provider::ContentPart::Text {
            text: enhanced_prompt.clone(),
        }];

        // Add each image as a content part
        for image_url in processed_images {
            content_parts.push(crate::provider::ContentPart::ImageUrl {
                image_url: crate::provider::ImageUrl {
                    url: image_url,
                    detail: Some("auto".to_string()),
                },
            });
        }

        vec![crate::provider::Message {
            role: "user".to_string(),
            content_type: crate::provider::MessageContent::Multimodal {
                content: content_parts,
            },
            tool_calls: None,
            tool_call_id: None,
        }]
    } else {
        // Regular text message
        vec![]
    };

    // Send the prompt
    if use_streaming {
        // Use streaming
        if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // For now, tools don't support streaming, fall back to regular
            print!("{}", "Thinking...".dimmed());
            // Deliberately flush stdout to show thinking indicator immediately
            io::stdout().flush()?;
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();
            match chat::send_chat_request_with_tool_execution(
                &client,
                &model_name,
                &enhanced_prompt,
                &[],
                system_prompt,
                max_tokens,
                temperature,
                &provider_name,
                mcp_tools,
                &server_refs,
            )
            .await
            {
                Ok((response, input_tokens, output_tokens)) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{}", response);

                    // Save to database with token counts (save a shortened version for cleaner logs)
                    let log_prompt = if piped_content.len() > 100 {
                        format!("{}... (piped content)", &piped_content[..100])
                    } else {
                        format!("{} (piped content)", piped_content)
                    };

                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &model_name,
                        &log_prompt,
                        &response,
                        input_tokens,
                        output_tokens,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }
                }
                Err(e) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    anyhow::bail!("Error: {}", e);
                }
            }
        } else {
            // Use streaming chat - content is streamed directly to stdout
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_streaming_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_streaming(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok(_) => {
                    // Streaming completed successfully, add a newline
                    println!();

                    // Save to database with token counts (save a shortened version for cleaner logs)
                    let log_prompt = if piped_content.len() > 100 {
                        format!("{}... (piped content)", &piped_content[..100])
                    } else {
                        format!("{} (piped content)", piped_content)
                    };

                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &model_name,
                        &log_prompt,
                        "[Streamed Response]",
                        None,
                        None,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }
                }
                Err(e) => {
                    anyhow::bail!("Error: {}", e);
                }
            }
        }
    } else {
        // Use regular non-streaming
        print!("{}", "Thinking...".dimmed());
        // Deliberately flush stdout to show thinking indicator immediately
        io::stdout().flush()?;

        let result = if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // Use tool execution loop when tools are available
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();
            chat::send_chat_request_with_tool_execution(
                &client,
                &model_name,
                &enhanced_prompt,
                &[],
                system_prompt,
                max_tokens,
                temperature,
                &provider_name,
                mcp_tools,
                &server_refs,
            )
            .await
        } else {
            // Use regular chat when no tools
            if !messages.is_empty() {
                chat::send_chat_request_with_validation_messages(
                    &client,
                    &model_name,
                    &messages,
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_validation(
                    &client,
                    &model_name,
                    &enhanced_prompt,
                    &[],
                    system_prompt,
                    max_tokens,
                    temperature,
                    &provider_name,
                    None,
                )
                .await
            }
        };

        match result {
            Ok((response, input_tokens, output_tokens)) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
                println!("{}", response);

                // Save to database with token counts (save a shortened version for cleaner logs)
                let log_prompt = if piped_content.len() > 100 {
                    format!("{}... (piped content)", &piped_content[..100])
                } else {
                    format!("{} (piped content)", piped_content)
                };

                if let Err(e) = db.save_chat_entry_with_tokens(
                    &session_id,
                    &model_name,
                    &log_prompt,
                    &response,
                    input_tokens,
                    output_tokens,
                ) {
                    eprintln!("Warning: Failed to save chat entry: {}", e);
                }
            }
            Err(e) => {
                print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                anyhow::bail!("Error: {}", e);
            }
        }
    }

    Ok(())
}

// Interactive chat mode
pub async fn handle_chat_command(
    model: Option<String>,
    provider: Option<String>,
    cid: Option<String>,
    tools: Option<String>,
    database: Option<String>,
    debug: bool,
    images: Vec<String>,
    stream: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }
    let config = config::Config::load()?;
    let db = database::Database::new()?;

    // Note: We don't enforce vision capability checks here as model metadata may be incomplete.
    // Let the API/model handle vision support validation and return appropriate errors if needed.

    // Determine session ID
    let session_id = cid.unwrap_or_else(|| {
        let new_id = uuid::Uuid::new_v4().to_string();
        db.set_current_session_id(&new_id).unwrap();
        new_id
    });

    // Resolve provider and model using the same logic as direct prompts
    let (provider_name, resolved_model) = resolve_model_and_provider(&config, provider, model)?;
    let _provider_config = config.get_provider(&provider_name)?;

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Fetch MCP tools if specified
    let (mcp_tools, mcp_server_names) = if let Some(tools_str) = &tools {
        fetch_mcp_tools(tools_str).await?
    } else {
        (None, Vec::new())
    };

    let mut current_model = resolved_model.clone();

    // Process initial images if provided
    let mut processed_images = if !images.is_empty() {
        println!(
            "{} Processing {} initial image(s)...",
            "🖼️".blue(),
            images.len()
        );
        crate::image_utils::process_images(&images)?
    } else {
        Vec::new()
    };

    println!("\n{} Interactive Chat Mode", "🚀".blue());
    println!("{} Session ID: {}", "📝".blue(), session_id);
    println!("{} Model: {}", "🤖".blue(), current_model);
    if !processed_images.is_empty() {
        println!("{} Initial images: {}", "🖼️".blue(), images.len());
    }
    if mcp_tools.is_some() && !mcp_server_names.is_empty() {
        println!(
            "{} Tools: {} (from MCP servers: {})",
            "🔧".blue(),
            mcp_tools.as_ref().unwrap().len(),
            mcp_server_names.join(", ")
        );
    }
    println!("{} Type /help for commands, /exit to quit", "💡".yellow());
    println!("{} Use Shift+Enter or Ctrl+J for multi-line input, Enter to send\n", "💡".yellow());

    // Create multi-line input handler
    let mut input_handler = MultiLineInput::new();

    loop {
        // Use multi-line input handler
        let input_string = match input_handler.read_input(&format!("{}", "You:".bold().green())) {
            Ok(input_text) => input_text.trim().to_string(),
            Err(_) => {
                // If there's an error with multi-line input, fall back to simple input
                print!("{} ", "You:".bold().green());
                io::stdout().flush()?;
                
                let mut fallback_input = String::new();
                let bytes_read = io::stdin().read_line(&mut fallback_input)?;
                
                // If we read 0 bytes, it means EOF (e.g., when input is piped)
                if bytes_read == 0 {
                    println!("Goodbye! 👋");
                    break;
                }
                
                fallback_input.trim().to_string()
            }
        };

        if input_string.is_empty() {
            continue;
        }

        let input = input_string.as_str();

        // Handle chat commands
        if input.starts_with('/') {
            match input {
                "/exit" => {
                    println!("Goodbye! 👋");
                    break;
                }
                "/clear" => {
                    db.clear_session(&session_id)?;
                    println!("{} Session cleared", "✓".green());
                    continue;
                }
                "/help" => {
                    println!("\n{}", "Available Commands:".bold().blue());
                    println!("  /exit          - Exit chat session");
                    println!("  /clear         - Clear current session");
                    println!("  /model <name>  - Change model");
                    println!("  /help          - Show this help");
                    println!("\n{}", "Input Controls:".bold().blue());
                    println!("  Enter          - Send message");
                    println!("  Shift+Enter    - New line (multi-line input)");
                    println!("  Ctrl+J         - New line (alternative)");
                    println!("  Ctrl+C         - Cancel current input\n");
                    continue;
                }
                _ if input.starts_with("/model ") => {
                    let new_model = input.strip_prefix("/model ").unwrap().trim();
                    if !new_model.is_empty() {
                        current_model = new_model.to_string();
                        println!("{} Model changed to: {}", "✓".green(), current_model);
                    } else {
                        println!("{} Please specify a model name", "✗".red());
                    }
                    continue;
                }
                _ => {
                    println!(
                        "{} Unknown command. Type /help for available commands",
                        "✗".red()
                    );
                    continue;
                }
            }
        }

        // Send chat message
        let history = db.get_chat_history(&session_id)?;

        // RAG: Retrieve relevant context if database is specified
        let mut enhanced_input = input.to_string();
        if let Some(ref db_name) = database {
            match retrieve_rag_context(db_name, &input, &client, &current_model, &provider_name)
                .await
            {
                Ok(context) => {
                    if !context.is_empty() {
                        enhanced_input = format!(
                            "Context from knowledge base:\n{}\n\nUser question: {}",
                            context, input
                        );
                        println!(
                            "{} Retrieved {} relevant context entries from '{}'",
                            "🧠".blue(),
                            context.lines().filter(|l| l.starts_with("- ")).count(),
                            db_name
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to retrieve RAG context: {}", e);
                }
            }
        }

        // Create messages with images if we have initial images
        let messages = if !processed_images.is_empty() {
            // Build history messages first
            let mut msgs: Vec<crate::provider::Message> = history
                .iter()
                .flat_map(|entry| {
                    vec![
                        crate::provider::Message::user(entry.question.clone()),
                        crate::provider::Message::assistant(entry.response.clone()),
                    ]
                })
                .collect();

            // Add current message with images
            let mut content_parts = vec![crate::provider::ContentPart::Text {
                text: enhanced_input.clone(),
            }];

            // Add each image as a content part
            for image_url in &processed_images {
                content_parts.push(crate::provider::ContentPart::ImageUrl {
                    image_url: crate::provider::ImageUrl {
                        url: image_url.clone(),
                        detail: Some("auto".to_string()),
                    },
                });
            }

            msgs.push(crate::provider::Message {
                role: "user".to_string(),
                content_type: crate::provider::MessageContent::Multimodal {
                    content: content_parts,
                },
                tool_calls: None,
                tool_call_id: None,
            });

            msgs
        } else {
            Vec::new()
        };

        // Add newline before "Thinking..." to ensure proper positioning after multi-line input
        println!();
        print!("{}", "Thinking...".dimmed());
        // Deliberately flush stdout to show thinking indicator immediately
        io::stdout().flush()?;

        let resolved_system_prompt = if let Some(system_prompt) = &config.system_prompt {
            Some(config.resolve_template_or_prompt(system_prompt))
        } else {
            None
        };

        // Determine if streaming should be used (default to true for interactive chat)
        // CLI flag takes precedence, then config, then default to true for chat mode
        let use_streaming = stream || config.stream.unwrap_or(true);

        if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // Use tool execution loop when tools are available (tools don't support streaming yet)
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_tool_execution_messages(
                    &client,
                    &current_model,
                    &messages,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    mcp_tools.clone(),
                    &server_refs,
                )
                .await
            } else {
                chat::send_chat_request_with_tool_execution(
                    &client,
                    &current_model,
                    &enhanced_input,
                    &history,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    mcp_tools.clone(),
                    &server_refs,
                )
                .await
            };

            match result {
                Ok((response, input_tokens, output_tokens)) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{} {}", "Assistant:".bold().blue(), response);

                    // Save to database with token counts
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &current_model,
                        &input,
                        &response,
                        input_tokens,
                        output_tokens,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }

                    // Clear processed images after first use
                    if !processed_images.is_empty() {
                        processed_images.clear();
                    }
                }
                Err(e) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{} Error: {}", "✗".red(), e);
                }
            }
        } else if use_streaming {
            // Use streaming chat when no tools and streaming is enabled - content is streamed directly to stdout
            print!("\r{}\r{} ", " ".repeat(12), "Assistant:".bold().blue()); // Clear "Thinking..." and show Assistant
            // Deliberately flush stdout to show assistant label before streaming
            io::stdout().flush()?;
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_streaming_messages(
                    &client,
                    &current_model,
                    &messages,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_streaming(
                    &client,
                    &current_model,
                    &enhanced_input,
                    &history,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok(_) => {
                    // Streaming completed successfully, add a newline
                    println!();

                    // Save to database with placeholder since the actual response was streamed
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &current_model,
                        &input,
                        "[Streamed Response]",
                        None,
                        None,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }

                    // Clear processed images after first use
                    if !processed_images.is_empty() {
                        processed_images.clear();
                    }
                }
                Err(e) => {
                    println!("\n{} Error: {}", "✗".red(), e);
                }
            }
        } else {
            // Use regular chat when no tools and streaming is disabled
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_validation_messages(
                    &client,
                    &current_model,
                    &messages,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_validation(
                    &client,
                    &current_model,
                    &enhanced_input,
                    &history,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok((response, input_tokens, output_tokens)) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{} {}", "Assistant:".bold().blue(), response);

                    // Save to database with token counts
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &current_model,
                        &input,
                        &response,
                        input_tokens,
                        output_tokens,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }

                    // Clear processed images after first use
                    if !processed_images.is_empty() {
                        processed_images.clear();
                    }
                }
                Err(e) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..." (12 chars)
                    println!("{} Error: {}", "✗".red(), e);
                }
            }
        }

        println!(); // Add spacing
    }

    Ok(())
}

// Models command handlers
pub async fn handle_models_command(
    command: Option<ModelsCommands>,
    query: Option<String>,
    tags: Option<String>,
    context_length: Option<String>,
    input_length: Option<String>,
    output_length: Option<String>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<()> {
    use colored::Colorize;

    match command {
        Some(ModelsCommands::Refresh) => {
            crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
        }
        Some(ModelsCommands::Info) => {
            debug_log!("Handling models info command");

            let models_dir = crate::unified_cache::UnifiedCache::models_dir()?;
            debug_log!("Models cache directory: {}", models_dir.display());

            println!("\n{}", "Models Cache Information:".bold().blue());
            println!("Cache Directory: {}", models_dir.display());

            if !models_dir.exists() {
                debug_log!("Cache directory does not exist");
                println!("Status: No cache directory found");
                return Ok(());
            }

            let entries = std::fs::read_dir(&models_dir)?;
            let mut provider_count = 0;
            let mut total_models = 0;

            debug_log!("Reading cache directory entries");

            // Collect provider information first
            let mut provider_info = Vec::new();
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                            debug_log!("Processing cache file for provider: {}", provider_name);
                            provider_count += 1;
                            match crate::unified_cache::UnifiedCache::load_provider_models(
                                provider_name,
                            )
                            .await
                            {
                                Ok(models) => {
                                    let count = models.len();
                                    total_models += count;
                                    debug_log!(
                                        "Provider '{}' has {} cached models",
                                        provider_name,
                                        count
                                    );

                                    let age_display =
                                        crate::unified_cache::UnifiedCache::get_cache_age_display(
                                            provider_name,
                                        )
                                        .await
                                        .unwrap_or_else(|_| "Unknown".to_string());
                                    let is_fresh =
                                        crate::unified_cache::UnifiedCache::is_cache_fresh(
                                            provider_name,
                                        )
                                        .await
                                        .unwrap_or(false);
                                    debug_log!(
                                        "Provider '{}' cache age: {}, fresh: {}",
                                        provider_name,
                                        age_display,
                                        is_fresh
                                    );

                                    let status = if is_fresh {
                                        age_display.green()
                                    } else {
                                        format!("{} (expired)", age_display).red()
                                    };
                                    provider_info.push((provider_name.to_string(), count, status));
                                }
                                Err(e) => {
                                    debug_log!(
                                        "Error loading cache for provider '{}': {}",
                                        provider_name,
                                        e
                                    );
                                    provider_info.push((
                                        provider_name.to_string(),
                                        0,
                                        "Error loading cache".red(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            debug_log!("Sorting {} providers alphabetically", provider_info.len());

            // Sort providers alphabetically by name
            provider_info.sort_by(|a, b| a.0.cmp(&b.0));

            println!("\nCached Providers:");
            for (provider_name, count, status) in provider_info {
                if count > 0 {
                    println!(
                        "  {} {} - {} models ({})",
                        "•".blue(),
                        provider_name.bold(),
                        count,
                        status
                    );
                } else {
                    println!("  {} {} - {}", "•".blue(), provider_name.bold(), status);
                }
            }

            debug_log!(
                "Cache summary: {} providers, {} total models",
                provider_count,
                total_models
            );

            println!("\nSummary:");
            println!("  Providers: {}", provider_count);
            println!("  Total Models: {}", total_models);
        }
        Some(ModelsCommands::Dump) => {
            dump_models_data().await?;
        }
        Some(ModelsCommands::Embed) => {
            debug_log!("Handling embedding models command");

            // Use unified cache for embedding models command
            debug_log!("Loading all cached models from unified cache");
            let enhanced_models =
                crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            debug_log!("Loaded {} models from cache", enhanced_models.len());

            // If no cached models found, refresh all providers
            if enhanced_models.is_empty() {
                debug_log!("No cached models found, refreshing all providers");
                println!("No cached models found. Refreshing all providers...");
                crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
                let enhanced_models =
                    crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

                debug_log!("After refresh, loaded {} models", enhanced_models.len());

                if enhanced_models.is_empty() {
                    debug_log!("Still no models found after refresh");
                    println!("No models found after refresh.");
                    return Ok(());
                }
            }

            debug_log!("Filtering for embedding models");

            // Filter for embedding models only
            let embedding_models: Vec<_> = enhanced_models
                .into_iter()
                .filter(|model| {
                    matches!(
                        model.model_type,
                        crate::model_metadata::ModelType::Embedding
                    )
                })
                .collect();

            debug_log!("Found {} embedding models", embedding_models.len());

            if embedding_models.is_empty() {
                println!("No embedding models found.");
                return Ok(());
            }

            // Display results
            debug_log!("Displaying {} embedding models", embedding_models.len());
            display_embedding_models(&embedding_models)?;
        }
        Some(ModelsCommands::Path { command }) => match command {
            ModelsPathCommands::List => {
                crate::model_metadata::list_model_paths()?;
            }
            ModelsPathCommands::Add { path } => {
                crate::model_metadata::add_model_path(path)?;
            }
            ModelsPathCommands::Delete { path } => {
                crate::model_metadata::remove_model_path(path)?;
            }
        },
        Some(ModelsCommands::Tags { command }) => {
            match command {
                ModelsTagsCommands::List => {
                    crate::model_metadata::list_tags()?;
                }
                ModelsTagsCommands::Add { tag, rule } => {
                    // For simplicity, we'll add a single path rule
                    crate::model_metadata::add_tag(tag, vec![rule], "string".to_string(), None)?;
                }
            }
        }
        Some(ModelsCommands::Filter { tags: filter_tags }) => {
            // Load all models
            let models = crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            // Parse tags
            let required_tags: Vec<&str> = filter_tags.split(',').map(|s| s.trim()).collect();

            // Filter models based on tags
            let filtered: Vec<_> = models
                .into_iter()
                .filter(|model| {
                    for tag in &required_tags {
                        match *tag {
                            "tools" => {
                                if !model.supports_tools && !model.supports_function_calling {
                                    return false;
                                }
                            }
                            "vision" => {
                                if !model.supports_vision {
                                    return false;
                                }
                            }
                            "audio" => {
                                if !model.supports_audio {
                                    return false;
                                }
                            }
                            "reasoning" => {
                                if !model.supports_reasoning {
                                    return false;
                                }
                            }
                            "code" => {
                                if !model.supports_code {
                                    return false;
                                }
                            }
                            _ => {
                                // Check for context length filters like "ctx>100k"
                                if tag.starts_with("ctx") {
                                    if let Some(ctx) = model.context_length {
                                        if tag.contains('>') {
                                            if let Some(min_str) = tag.split('>').nth(1) {
                                                if let Ok(min_ctx) = parse_token_count(min_str) {
                                                    if ctx < min_ctx {
                                                        return false;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                })
                .collect();

            if filtered.is_empty() {
                println!("No models found with tags: {}", filter_tags);
            } else {
                println!(
                    "\n{} Models with tags [{}] ({} found):",
                    "Filtered Results:".bold().blue(),
                    filter_tags,
                    filtered.len()
                );

                let mut current_provider = String::new();
                for model in filtered {
                    if model.provider != current_provider {
                        current_provider = model.provider.clone();
                        println!("\n{}", format!("{}:", current_provider).bold().green());
                    }

                    print!("  {} {}", "•".blue(), model.id.bold());

                    // Show capabilities
                    let mut capabilities = Vec::new();
                    if model.supports_tools || model.supports_function_calling {
                        capabilities.push("🔧 tools".blue());
                    }
                    if model.supports_vision {
                        capabilities.push("👁 vision".magenta());
                    }
                    if model.supports_audio {
                        capabilities.push("🔊 audio".yellow());
                    }
                    if model.supports_reasoning {
                        capabilities.push("🧠 reasoning".cyan());
                    }
                    if model.supports_code {
                        capabilities.push("💻 code".green());
                    }

                    if !capabilities.is_empty() {
                        let capability_strings: Vec<String> =
                            capabilities.iter().map(|c| c.to_string()).collect();
                        print!(" [{}]", capability_strings.join(" "));
                    }

                    // Show context info
                    if let Some(ctx) = model.context_length {
                        if ctx >= 1000 {
                            print!(" ({}k ctx)", ctx / 1000);
                        } else {
                            print!(" ({} ctx)", ctx);
                        }
                    }

                    println!();
                }
            }
        }
        None => {
            debug_log!("Handling global models command");

            // Use unified cache for global models command
            debug_log!("Loading all cached models from unified cache");
            let enhanced_models =
                crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            debug_log!("Loaded {} models from cache", enhanced_models.len());

            // If no cached models found, refresh all providers
            if enhanced_models.is_empty() {
                debug_log!("No cached models found, refreshing all providers");
                println!("No cached models found. Refreshing all providers...");
                crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
                let enhanced_models =
                    crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

                debug_log!("After refresh, loaded {} models", enhanced_models.len());

                if enhanced_models.is_empty() {
                    debug_log!("Still no models found after refresh");
                    println!("No models found after refresh.");
                    return Ok(());
                }
            }

            debug_log!("Applying filters to {} models", enhanced_models.len());

            // Parse tags if provided
            let tag_filters = if let Some(ref tag_str) = tags {
                let tags_vec: Vec<String> =
                    tag_str.split(',').map(|s| s.trim().to_string()).collect();
                Some(tags_vec)
            } else {
                None
            };

            // Apply filters
            let filtered_models = apply_model_filters_with_tags(
                enhanced_models,
                &query,
                tag_filters,
                &context_length,
                &input_length,
                &output_length,
                input_price,
                output_price,
            )?;

            debug_log!("After filtering, {} models remain", filtered_models.len());

            if filtered_models.is_empty() {
                debug_log!("No models match the specified criteria");
                println!("No models found matching the specified criteria.");
                return Ok(());
            }

            // Display results
            debug_log!("Displaying {} filtered models", filtered_models.len());
            display_enhanced_models(&filtered_models, &query)?;
        }
    }

    Ok(())
}

// Template command handlers
pub async fn handle_template_command(command: TemplateCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        TemplateCommands::Add { name, prompt } => {
            let mut config = config::Config::load()?;
            config.add_template(name.clone(), prompt.clone())?;
            config.save()?;
            println!("{} Template '{}' added", "✓".green(), name);
        }
        TemplateCommands::Delete { name } => {
            let mut config = config::Config::load()?;
            config.remove_template(name.clone())?;
            config.save()?;
            println!("{} Template '{}' removed", "✓".green(), name);
        }
        TemplateCommands::List => {
            let config = config::Config::load()?;
            let templates = config.list_templates();

            if templates.is_empty() {
                println!("No templates configured.");
            } else {
                println!("\n{}", "Templates:".bold().blue());
                for (name, prompt) in templates {
                    let display_prompt = if prompt.len() > 60 {
                        format!("{}...", &prompt[..60])
                    } else {
                        prompt.clone()
                    };
                    println!("  {} {} -> {}", "•".blue(), name.bold(), display_prompt);
                }
            }
        }
    }

    Ok(())
}

// Proxy command handler
pub async fn handle_proxy_command(
    port: u16,
    host: String,
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    generate_key: bool,
) -> Result<()> {
    use crate::proxy;

    // Handle API key generation
    let final_api_key = if generate_key {
        let generated_key = proxy::generate_api_key();
        println!(
            "{} Generated API key: {}",
            "🔑".green(),
            generated_key.bold()
        );
        Some(generated_key)
    } else {
        api_key
    };

    // Validate provider if specified
    if let Some(ref provider_name) = provider {
        let config = config::Config::load()?;
        if !config.has_provider(provider_name) {
            anyhow::bail!(
                "Provider '{}' not found. Add it first with 'lc providers add'",
                provider_name
            );
        }
    }

    // Validate model if specified (could be alias or provider:model format)
    if let Some(ref model_name) = model {
        let config = config::Config::load()?;

        // Check if it's an alias
        if let Some(_alias_target) = config.get_alias(model_name) {
            // Valid alias
        } else if model_name.contains(':') {
            // Check provider:model format
            let parts: Vec<&str> = model_name.splitn(2, ':').collect();
            if parts.len() == 2 {
                let provider_name = parts[0];
                if !config.has_provider(provider_name) {
                    anyhow::bail!(
                        "Provider '{}' not found in model specification '{}'",
                        provider_name,
                        model_name
                    );
                }
            }
        } else {
            // Assume it's a model name for the default or specified provider
            // This will be validated when requests come in
        }
    }

    // Show configuration summary
    println!("\n{}", "Proxy Server Configuration:".bold().blue());
    println!("  {} {}:{}", "Address:".bold(), host, port);

    if let Some(ref provider_filter) = provider {
        println!(
            "  {} {}",
            "Provider Filter:".bold(),
            provider_filter.green()
        );
    } else {
        println!(
            "  {} {}",
            "Provider Filter:".bold(),
            "All providers".dimmed()
        );
    }

    if let Some(ref model_filter) = model {
        println!("  {} {}", "Model Filter:".bold(), model_filter.green());
    } else {
        println!("  {} {}", "Model Filter:".bold(), "All models".dimmed());
    }

    if final_api_key.is_some() {
        println!("  {} {}", "Authentication:".bold(), "Enabled".green());
    } else {
        println!("  {} {}", "Authentication:".bold(), "Disabled".yellow());
    }

    println!("\n{}", "Available endpoints:".bold().blue());
    println!("  {} http://{}:{}/models", "•".blue(), host, port);
    println!("  {} http://{}:{}/v1/models", "•".blue(), host, port);
    println!("  {} http://{}:{}/chat/completions", "•".blue(), host, port);
    println!(
        "  {} http://{}:{}/v1/chat/completions",
        "•".blue(),
        host,
        port
    );

    println!("\n{} Press Ctrl+C to stop the server\n", "💡".yellow());

    // Start the proxy server
    proxy::start_proxy_server(host, port, provider, model, final_api_key).await?;

    Ok(())
}

// Dump models data function
async fn dump_models_data() -> Result<()> {
    use crate::{chat, config::Config};

    println!("{} Dumping /models for each provider...", "🔍".blue());

    // Load configuration
    let config = Config::load()?;

    // Create models directory if it doesn't exist
    std::fs::create_dir_all("models")?;

    let mut successful_dumps = 0;
    let mut total_providers = 0;

    for (provider_name, provider_config) in &config.providers {
        total_providers += 1;

        // Skip providers without API keys
        if provider_config.api_key.is_none() {
            println!("{} Skipping {} (no API key)", "⚠️".yellow(), provider_name);
            continue;
        }

        println!("{} Fetching models from {}...", "📡".blue(), provider_name);

        // Create authenticated client
        let mut config_mut = config.clone();
        match chat::create_authenticated_client(&mut config_mut, provider_name).await {
            Ok(client) => {
                // Make raw request to get full JSON response
                match fetch_raw_models_response(&client, provider_config).await {
                    Ok(raw_response) => {
                        // Save raw response to file
                        let filename = format!("models/{}.json", provider_name);
                        match std::fs::write(&filename, &raw_response) {
                            Ok(_) => {
                                println!(
                                    "{} Saved {} models data to {}",
                                    "✅".green(),
                                    provider_name,
                                    filename
                                );
                                successful_dumps += 1;
                            }
                            Err(e) => {
                                println!(
                                    "{} Failed to save {} models data: {}",
                                    "❌".red(),
                                    provider_name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to fetch models from {}: {}",
                            "❌".red(),
                            provider_name,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                println!(
                    "{} Failed to create client for {}: {}",
                    "❌".red(),
                    provider_name,
                    e
                );
            }
        }
    }

    println!("\n{} Summary:", "📊".blue());
    println!("   Total providers: {}", total_providers);
    println!("   Successful dumps: {}", successful_dumps);
    println!("   Models data saved to: ./models/");

    if successful_dumps > 0 {
        println!("\n{} Model data collection complete!", "🎉".green());
        println!("   Next step: Analyze the JSON files to extract metadata patterns");
    }

    Ok(())
}

fn apply_model_filters_with_tags(
    models: Vec<crate::model_metadata::ModelMetadata>,
    query: &Option<String>,
    tag_filters: Option<Vec<String>>,
    context_length: &Option<String>,
    input_length: &Option<String>,
    output_length: &Option<String>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<Vec<crate::model_metadata::ModelMetadata>> {
    let mut filtered = models;

    // Apply text search filter
    if let Some(ref search_query) = query {
        let query_lower = search_query.to_lowercase();
        filtered.retain(|model| {
            model.id.to_lowercase().contains(&query_lower)
                || model
                    .display_name
                    .as_ref()
                    .map_or(false, |name| name.to_lowercase().contains(&query_lower))
                || model
                    .description
                    .as_ref()
                    .map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
        });
    }

    // Apply tag filters if provided
    if let Some(tags) = tag_filters {
        for tag in tags {
            match tag.as_str() {
                "tools" => {
                    filtered
                        .retain(|model| model.supports_tools || model.supports_function_calling);
                }
                "reasoning" => {
                    filtered.retain(|model| model.supports_reasoning);
                }
                "vision" => {
                    filtered.retain(|model| model.supports_vision);
                }
                "audio" => {
                    filtered.retain(|model| model.supports_audio);
                }
                "code" => {
                    filtered.retain(|model| model.supports_code);
                }
                _ => {
                    // Ignore unknown tags
                }
            }
        }
    }

    // Apply context length filter
    if let Some(ref ctx_str) = context_length {
        let min_ctx = parse_token_count(ctx_str)?;
        filtered.retain(|model| model.context_length.map_or(false, |ctx| ctx >= min_ctx));
    }

    // Apply input length filter
    if let Some(ref input_str) = input_length {
        let min_input = parse_token_count(input_str)?;
        filtered.retain(|model| {
            model
                .max_input_tokens
                .map_or(false, |input| input >= min_input)
                || model.context_length.map_or(false, |ctx| ctx >= min_input)
        });
    }

    // Apply output length filter
    if let Some(ref output_str) = output_length {
        let min_output = parse_token_count(output_str)?;
        filtered.retain(|model| {
            model
                .max_output_tokens
                .map_or(false, |output| output >= min_output)
        });
    }

    // Apply price filters
    if let Some(max_input_price) = input_price {
        filtered.retain(|model| {
            model
                .input_price_per_m
                .map_or(true, |price| price <= max_input_price)
        });
    }

    if let Some(max_output_price) = output_price {
        filtered.retain(|model| {
            model
                .output_price_per_m
                .map_or(true, |price| price <= max_output_price)
        });
    }

    // Sort by provider, then by model name
    filtered.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.id.cmp(&b.id)));

    Ok(filtered)
}

fn parse_token_count(input: &str) -> Result<u32> {
    let input = input.to_lowercase();
    if let Some(num_str) = input.strip_suffix('k') {
        let num: f32 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000.0) as u32)
    } else if let Some(num_str) = input.strip_suffix('m') {
        let num: f32 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000000.0) as u32)
    } else {
        input
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))
    }
}

fn display_enhanced_models(
    models: &[crate::model_metadata::ModelMetadata],
    query: &Option<String>,
) -> Result<()> {
    use colored::Colorize;

    if let Some(ref search_query) = query {
        println!(
            "\n{} Models matching '{}' ({} found):",
            "Search Results:".bold().blue(),
            search_query,
            models.len()
        );
    } else {
        println!(
            "\n{} Available models ({} total):",
            "Models:".bold().blue(),
            models.len()
        );
    }

    let mut current_provider = String::new();
    for model in models {
        if model.provider != current_provider {
            current_provider = model.provider.clone();
            println!("\n{}", format!("{}:", current_provider).bold().green());
        }

        // Build capability indicators
        let mut capabilities = Vec::new();
        if model.supports_tools || model.supports_function_calling {
            capabilities.push("🔧 tools".blue());
        }
        if model.supports_vision {
            capabilities.push("👁 vision".magenta());
        }
        if model.supports_audio {
            capabilities.push("🔊 audio".yellow());
        }
        if model.supports_reasoning {
            capabilities.push("🧠 reasoning".cyan());
        }
        if model.supports_code {
            capabilities.push("💻 code".green());
        }

        // Build context info
        let mut context_info = Vec::new();
        if let Some(ctx) = model.context_length {
            context_info.push(format!("{}k ctx", ctx / 1000));
        }
        if let Some(max_out) = model.max_output_tokens {
            context_info.push(format!("{}k out", max_out / 1000));
        }

        // Display model with metadata
        let model_display = if let Some(ref display_name) = model.display_name {
            format!("{} ({})", model.id, display_name)
        } else {
            model.id.clone()
        };

        print!("  {} {}", "•".blue(), model_display.bold());

        if !capabilities.is_empty() {
            let capability_strings: Vec<String> =
                capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }

        if !context_info.is_empty() {
            print!(" ({})", context_info.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}

pub async fn fetch_raw_models_response(
    _client: &crate::chat::LLMClient,
    provider_config: &crate::config::ProviderConfig,
) -> Result<String> {
    use serde_json::Value;

    // Use the shared optimized HTTP client
    // Create optimized HTTP client with connection pooling and keep-alive settings
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = provider_config.get_models_url();

    debug_log!("Making API request to: {}", url);
    debug_log!("Request timeout: 60 seconds");

    let mut req = http_client
        .get(&url)
        .header("Content-Type", "application/json");

    debug_log!("Added Content-Type: application/json header");

    // Add custom headers first
    let mut has_custom_headers = false;
    for (name, value) in &provider_config.headers {
        debug_log!("Adding custom header: {}: {}", name, value);
        req = req.header(name, value);
        has_custom_headers = true;
    }

    // Only add Authorization header if no custom headers are present
    if !has_custom_headers {
        if let Some(api_key) = provider_config.api_key.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
            debug_log!("Added Authorization header with API key");
        } else {
            debug_log!("No API key configured and no custom headers provided; cannot add Authorization header");
            // Return a clear error instead of panicking
            anyhow::bail!("No API key configured and no custom headers set for models request");
        }
    } else {
        debug_log!("Skipping Authorization header due to custom headers present");
    }

    debug_log!("Sending HTTP GET request...");
    let response = req.send().await?;

    let status = response.status();
    debug_log!("Received response with status: {}", status);

    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        debug_log!("API request failed with error response: {}", text);
        anyhow::bail!("API request failed with status {}: {}", status, text);
    }

    let response_text = response.text().await?;
    debug_log!("Received response body ({} bytes)", response_text.len());

    // Pretty print the JSON for better readability
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => {
            debug_log!("Response is valid JSON, pretty-printing");
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        Err(_) => {
            debug_log!("Response is not valid JSON, returning as-is");
            // If it's not valid JSON, return as-is
            Ok(response_text)
        }
    }
}

// Alias command handlers
pub async fn handle_alias_command(command: AliasCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        AliasCommands::Add { name, target } => {
            let mut config = config::Config::load()?;
            config.add_alias(name.clone(), target.clone())?;
            config.save()?;
            println!("{} Alias '{}' added for '{}'", "✓".green(), name, target);
        }
        AliasCommands::Delete { name } => {
            let mut config = config::Config::load()?;
            config.remove_alias(name.clone())?;
            config.save()?;
            println!("{} Alias '{}' removed", "✓".green(), name);
        }
        AliasCommands::List => {
            let config = config::Config::load()?;
            let aliases = config.list_aliases();

            if aliases.is_empty() {
                println!("No aliases configured.");
            } else {
                println!("\n{}", "Model Aliases:".bold().blue());
                for (alias, target) in aliases {
                    println!("  {} {} -> {}", "•".blue(), alias.bold(), target);
                }
            }
        }
    }

    Ok(())
}

// Load enhanced models for a specific provider
async fn load_provider_enhanced_models(
    provider_name: &str,
) -> Result<Vec<crate::model_metadata::ModelMetadata>> {
    use crate::model_metadata::MetadataExtractor;
    use std::fs;

    let filename = format!("models/{}.json", provider_name);

    if !std::path::Path::new(&filename).exists() {
        return Ok(Vec::new());
    }

    match fs::read_to_string(&filename) {
        Ok(json_content) => {
            match MetadataExtractor::extract_from_provider(provider_name, &json_content) {
                Ok(models) => Ok(models),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to extract metadata from {}: {}",
                        provider_name, e
                    );
                    Ok(Vec::new())
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", filename, e);
            Ok(Vec::new())
        }
    }
}

// Display provider models with metadata
fn display_provider_models(models: &[crate::model_metadata::ModelMetadata]) -> Result<()> {
    use colored::Colorize;

    for model in models {
        // Safety check: log if all capability flags are false to catch defaulting bugs
        if !model.supports_tools
            && !model.supports_function_calling
            && !model.supports_vision
            && !model.supports_audio
            && !model.supports_reasoning
            && !model.supports_code
        {
            debug_log!("All capability flags are false for model '{}' - this might indicate a defaulting bug", model.id);
        }

        // Build capability indicators
        let mut capabilities = Vec::new();
        if model.supports_tools || model.supports_function_calling {
            capabilities.push("🔧 tools".blue());
        }
        if model.supports_vision {
            capabilities.push("👁 vision".magenta());
        }
        if model.supports_audio {
            capabilities.push("🔊 audio".yellow());
        }
        if model.supports_reasoning {
            capabilities.push("🧠 reasoning".cyan());
        }
        if model.supports_code {
            capabilities.push("💻 code".green());
        }

        // Build context and pricing info
        let mut info_parts = Vec::new();
        if let Some(ctx) = model.context_length {
            if ctx >= 1000000 {
                info_parts.push(format!("{}m ctx", ctx / 1000000));
            } else if ctx >= 1000 {
                info_parts.push(format!("{}k ctx", ctx / 1000));
            } else {
                info_parts.push(format!("{} ctx", ctx));
            }
        }
        if let Some(max_out) = model.max_output_tokens {
            if max_out >= 1000 {
                info_parts.push(format!("{}k out", max_out / 1000));
            } else {
                info_parts.push(format!("{} out", max_out));
            }
        }
        if let Some(input_price) = model.input_price_per_m {
            info_parts.push(format!("${:.2}/M in", input_price));
        }
        if let Some(output_price) = model.output_price_per_m {
            info_parts.push(format!("${:.2}/M out", output_price));
        }

        // Display model with metadata
        let model_display = if let Some(ref display_name) = model.display_name {
            if display_name != &model.id {
                format!("{} ({})", model.id, display_name)
            } else {
                model.id.clone()
            }
        } else {
            model.id.clone()
        };

        print!("  {} {}", "•".blue(), model_display.bold());

        if !capabilities.is_empty() {
            let capability_strings: Vec<String> =
                capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }

        if !info_parts.is_empty() {
            print!(" ({})", info_parts.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}

// MCP command handlers
pub async fn handle_mcp_command(command: crate::cli::McpCommands) -> Result<()> {
    use crate::mcp::{McpConfig, McpServerType as ConfigMcpServerType};
    use colored::Colorize;

    match command {
        crate::cli::McpCommands::Add {
            name,
            command_or_url,
            server_type,
            env,
        } => {
            let mut config = McpConfig::load()?;

            // Convert CLI enum to config enum
            let config_server_type = match server_type {
                crate::cli::McpServerType::Stdio => ConfigMcpServerType::Stdio,
                crate::cli::McpServerType::Sse => ConfigMcpServerType::Sse,
                crate::cli::McpServerType::Streamable => ConfigMcpServerType::Streamable,
            };

            // Convert env vec to HashMap
            let env_map: HashMap<String, String> = env.into_iter().collect();

            // For npx commands without -y, add it to ensure package download
            let final_command_or_url =
                if command_or_url.starts_with("npx ") && !command_or_url.contains(" -y ") {
                    command_or_url.replacen("npx ", "npx -y ", 1)
                } else {
                    command_or_url.clone()
                };

            config.add_server_with_env(
                name.clone(),
                final_command_or_url.clone(),
                config_server_type,
                env_map.clone(),
            )?;
            config.save()?;

            println!("{} MCP server '{}' added successfully", "✓".green(), name);
            println!("  Type: {:?}", server_type);
            println!("  Command/URL: {}", final_command_or_url);
            if !env_map.is_empty() {
                println!("  Environment variables:");
                for (key, _) in env_map {
                    println!("    - {}", key);
                }
            }
        }
        crate::cli::McpCommands::Delete { name } => {
            let mut config = McpConfig::load()?;

            if config.get_server(&name).is_none() {
                anyhow::bail!("MCP server '{}' not found", name);
            }

            config.delete_server(&name)?;
            config.save()?;

            println!("{} MCP server '{}' deleted successfully", "✓".green(), name);
        }
        crate::cli::McpCommands::List => {
            let config = McpConfig::load()?;
            let servers = config.list_servers();

            if servers.is_empty() {
                println!("No MCP servers configured.");
            } else {
                println!("\n{} Configured MCP servers:", "Servers:".bold().blue());
                for (name, server_config) in servers {
                    println!(
                        "  {} {} - {:?} ({})",
                        "•".blue(),
                        name.bold(),
                        server_config.server_type,
                        server_config.command_or_url
                    );
                }
            }
        }
        crate::cli::McpCommands::Stop { name } => {
            println!("{} Closing MCP server connection '{}'...", "🛑".red(), name);

            let daemon_client = crate::mcp_daemon::DaemonClient::new()?;
            match daemon_client.close_server(&name).await {
                Ok(_) => {
                    println!(
                        "{} MCP server '{}' connection closed successfully",
                        "✓".green(),
                        name
                    );
                }
                Err(e) => {
                    println!(
                        "{} Failed to close MCP server '{}': {}",
                        "⚠️".yellow(),
                        name,
                        e
                    );
                }
            }
        }
        crate::cli::McpCommands::Functions { name } => {
            let config = McpConfig::load()?;

            if config.get_server(&name).is_some() {
                println!(
                    "{} Listing functions for MCP server '{}'...",
                    "🔍".blue(),
                    name
                );

                // Use daemon client for persistent connections
                let daemon_client = crate::mcp_daemon::DaemonClient::new()?;

                crate::debug_log!("CLI: Starting MCP functions listing for server '{}'", name);

                // Ensure server is connected via daemon
                match daemon_client.ensure_server_connected(&name).await {
                    Ok(_) => {
                        crate::debug_log!("CLI: Server '{}' connected successfully", name);
                        match daemon_client.list_tools(&name).await {
                            Ok(all_tools) => {
                                crate::debug_log!(
                                    "CLI: Received tools response with {} servers",
                                    all_tools.len()
                                );
                                if let Some(tools) = all_tools.get(&name) {
                                    crate::debug_log!(
                                        "CLI: Server '{}' has {} tools",
                                        name,
                                        tools.len()
                                    );
                                    if tools.is_empty() {
                                        println!("No functions found for server '{}'", name);
                                    } else {
                                        println!(
                                            "\n{} Available functions:",
                                            "Functions:".bold().blue()
                                        );
                                        for tool in tools {
                                            println!(
                                                "  {} {} - {}",
                                                "•".blue(),
                                                tool.name.bold(),
                                                tool.description
                                                    .as_ref()
                                                    .map(|s| s.as_ref())
                                                    .unwrap_or("No description")
                                            );

                                            if let Some(properties) =
                                                tool.input_schema.get("properties")
                                            {
                                                if let Some(props_obj) = properties.as_object() {
                                                    if !props_obj.is_empty() {
                                                        println!(
                                                            "    Parameters: {}",
                                                            props_obj
                                                                .keys()
                                                                .map(|k| k.as_str())
                                                                .collect::<Vec<_>>()
                                                                .join(", ")
                                                                .dimmed()
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    crate::debug_log!(
                                        "CLI: No tools found for server '{}' in response",
                                        name
                                    );
                                    println!("No functions found for server '{}'", name);
                                }
                            }
                            Err(e) => {
                                crate::debug_log!("CLI: Failed to list tools: {}", e);
                                anyhow::bail!(
                                    "Failed to list functions from MCP server '{}': {}",
                                    name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        crate::debug_log!("CLI: Failed to connect to server '{}': {}", name, e);
                        anyhow::bail!("Failed to connect to MCP server '{}': {}", name, e);
                    }
                }
            } else {
                anyhow::bail!("MCP server '{}' not found", name);
            }
        }
                        crate::cli::McpCommands::Invoke {
            name,
            function,
            args,
        } => {
            let config = McpConfig::load()?;

            if config.get_server(&name).is_some() {
                println!(
                    "{} Invoking function '{}' on MCP server '{}'...",
                    "⚡".yellow(),
                    function,
                    name
                );
                if !args.is_empty() {
                    println!("Arguments: {}", args.join(", ").dimmed());
                }

                // Use daemon client for persistent connections
                let daemon_client = crate::mcp_daemon::DaemonClient::new()?;

                // Ensure server is connected via daemon
                match daemon_client.ensure_server_connected(&name).await {
                    Ok(_) => {
                        // Parse args as key=value pairs
                        let params = if args.is_empty() {
                            serde_json::json!({})
                        } else {
                            let mut params_obj = serde_json::Map::new();
                            for arg in args {
                                if let Some((key, value)) = arg.split_once('=') {
                                    params_obj.insert(key.to_string(), serde_json::json!(value));
                                } else {
                                    anyhow::bail!(
                                        "Invalid argument format: '{}'. Expected 'key=value'",
                                        arg
                                    );
                                }
                            }
                            serde_json::json!(params_obj)
                        };

                        match daemon_client.call_tool(&name, &function, params).await {
                            Ok(result) => {
                                println!("\n{} Result:", "Response:".bold().green());
                                println!("{}", serde_json::to_string_pretty(&result)?);
                            }
                            Err(e) => {
                                anyhow::bail!(
                                    "Failed to invoke function '{}' on MCP server '{}': {}",
                                    function,
                                    name,
                                    e
                                );
                            }
                        }

                        // Connection persists in daemon - browser stays open!
                        println!("\n{} Tool invocation completed. Server connection remains active in daemon.", "ℹ️".blue());
                        println!(
                            "{} Use 'lc mcp stop {}' if you want to close the server connection.",
                            "💡".yellow(),
                            name
                        );
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to connect to MCP server '{}': {}", name, e);
                    }
                }
            } else {
                anyhow::bail!("MCP server '{}' not found", name);
            }
        }
    }

    Ok(())
}

// Helper function to fetch MCP tools and convert them to OpenAI function format
pub async fn fetch_mcp_tools(
    tools_str: &str,
) -> Result<(Option<Vec<crate::provider::Tool>>, Vec<String>)> {
    use crate::mcp::McpConfig;

    // Parse comma-separated server names
    let server_names: Vec<&str> = tools_str.split(',').map(|s| s.trim()).collect();
    let mut all_tools = Vec::new();
    let mut valid_server_names = Vec::new();

    // Load MCP configuration
    let config = McpConfig::load()?;

                // Use daemon client for persistent connections
    let daemon_client = crate::mcp_daemon::DaemonClient::new()?;

    for server_name in server_names {
        if server_name.is_empty() {
            continue;
        }

        crate::debug_log!("Fetching tools from MCP server '{}'", server_name);

        // Check if server exists in configuration
        if config.get_server(server_name).is_some() {
            // Ensure server is connected via daemon
            match daemon_client.ensure_server_connected(server_name).await {
                Ok(_) => {
                    crate::debug_log!("Successfully connected to MCP server '{}'", server_name);
                    valid_server_names.push(server_name.to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to connect to MCP server '{}': {}",
                        server_name, e
                    );
                    continue;
                }
            }
        } else {
            eprintln!(
                "Warning: MCP server '{}' not found in configuration",
                server_name
            );
            continue;
        }
    }

    // Get all tools from connected servers using daemon client
    for server_name in &valid_server_names {
        match daemon_client.list_tools(server_name).await {
            Ok(server_tools) => {
                if let Some(tools) = server_tools.get(server_name) {
                    crate::debug_log!(
                        "Retrieved {} tools from server '{}'",
                        tools.len(),
                        server_name
                    );

                    for tool in tools {
                        // Convert MCP tool to OpenAI tool format
                        let openai_tool = crate::provider::Tool {
                            tool_type: "function".to_string(),
                            function: crate::provider::Function {
                                name: tool.name.to_string(),
                                description: tool
                                    .description
                                    .as_ref()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "No description".to_string()),
                                parameters: serde_json::to_value(&*tool.input_schema)
                                    .unwrap_or_else(|_| {
                                        serde_json::json!({
                                            "type": "object",
                                            "properties": {},
                                            "required": []
                                        })
                                    }),
                            },
                        };

                        all_tools.push(openai_tool);
                        crate::debug_log!(
                            "Added tool '{}' from server '{}'",
                            tool.name,
                            server_name
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to list tools from MCP server '{}': {}",
                    server_name, e
                );
            }
        }
    }

    // Connections persist in daemon - no cleanup needed

    if all_tools.is_empty() {
        crate::debug_log!("No tools found from any specified MCP servers");
        Ok((None, valid_server_names))
    } else {
        crate::debug_log!("Total {} tools fetched from MCP servers", all_tools.len());
        Ok((Some(all_tools), valid_server_names))
    }
}

// Display embedding models with metadata
fn display_embedding_models(models: &[crate::model_metadata::ModelMetadata]) -> Result<()> {
    use colored::Colorize;

    println!(
        "\n{} Available embedding models ({} total):",
        "Embedding Models:".bold().blue(),
        models.len()
    );

    let mut current_provider = String::new();
    for model in models {
        if model.provider != current_provider {
            current_provider = model.provider.clone();
            println!("\n{}", format!("{}:", current_provider).bold().green());
        }

        // Build context and pricing info
        let mut info_parts = Vec::new();
        if let Some(ctx) = model.context_length {
            if ctx >= 1000000 {
                info_parts.push(format!("{}m ctx", ctx / 1000000));
            } else if ctx >= 1000 {
                info_parts.push(format!("{}k ctx", ctx / 1000));
            } else {
                info_parts.push(format!("{} ctx", ctx));
            }
        }
        if let Some(input_price) = model.input_price_per_m {
            info_parts.push(format!("${:.2}/M", input_price));
        }

        // Display model with metadata
        let model_display = if let Some(ref display_name) = model.display_name {
            if display_name != &model.id {
                format!("{} ({})", model.id, display_name)
            } else {
                model.id.clone()
            }
        } else {
            model.id.clone()
        };

        print!("  {} {}", "•".blue(), model_display.bold());

        if !info_parts.is_empty() {
            print!(" ({})", info_parts.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}

// Embed command handler
pub async fn handle_embed_command(
    model: String,
    provider: Option<String>,
    database: Option<String>,
    files: Vec<String>,
    text: Option<String>,
    debug: bool,
) -> Result<()> {
    use colored::Colorize;

    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }

    // Validate input: either text or files must be provided
    if text.is_none() && files.is_empty() {
        anyhow::bail!("Either text or files must be provided for embedding");
    }

    let config = config::Config::load()?;

    // Resolve provider and model using the same logic as direct prompts
    let (provider_name, resolved_model) =
        resolve_model_and_provider(&config, provider, Some(model))?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    println!("{} Starting embedding process...", "🔄".blue());
    println!("{} Model: {}", "📊".blue(), resolved_model);
    println!("{} Provider: {}", "🏢".blue(), provider_name);

    let mut total_embeddings = 0;
    let mut total_tokens = 0;

    // Process files if provided
    if !files.is_empty() {
        println!("{} Processing files with glob patterns...", "📁".blue());

        // Expand file patterns and filter for text files
        let file_paths = crate::vector_db::FileProcessor::expand_file_patterns(&files)?;

        if file_paths.is_empty() {
            println!(
                "{} No text files found matching the patterns",
                "⚠️".yellow()
            );
        } else {
            println!(
                "{} Found {} text files to process",
                "✅".green(),
                file_paths.len()
            );

            for file_path in file_paths {
                println!("\n{} Processing file: {}", "📄".blue(), file_path.display());

                // Read and chunk the file
                match crate::vector_db::FileProcessor::process_file(&file_path) {
                    Ok(chunks) => {
                        println!("{} Split into {} chunks", "✂️".blue(), chunks.len());

                        // Process each chunk
                        for (chunk_index, chunk) in chunks.iter().enumerate() {
                            let embedding_request = crate::provider::EmbeddingRequest {
                                model: resolved_model.clone(),
                                input: chunk.clone(),
                                encoding_format: Some("float".to_string()),
                            };

                            match client.embeddings(&embedding_request).await {
                                Ok(response) => {
                                    if let Some(embedding_data) = response.data.first() {
                                        total_embeddings += 1;
                                        total_tokens += response.usage.total_tokens;

                                        // Store in vector database if specified
                                        if let Some(db_name) = &database {
                                            match crate::vector_db::VectorDatabase::new(db_name) {
                                                Ok(vector_db) => {
                                                    let file_path_str = file_path.to_string_lossy();
                                                    match vector_db.add_vector_with_metadata(
                                                        chunk,
                                                        &embedding_data.embedding,
                                                        &resolved_model,
                                                        &provider_name,
                                                        Some(&file_path_str),
                                                        Some(chunk_index as i32),
                                                        Some(chunks.len() as i32),
                                                    ) {
                                                        Ok(id) => {
                                                            println!("  {} Chunk {}/{} stored with ID: {}",
                                                                "💾".green(), chunk_index + 1, chunks.len(), id);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("  Warning: Failed to store chunk {}: {}", chunk_index + 1, e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("  Warning: Failed to create/open vector database '{}': {}", db_name, e);
                                                }
                                            }
                                        } else {
                                            // Just show progress without storing
                                            println!(
                                                "  {} Chunk {}/{} embedded ({} dimensions)",
                                                "✅".green(),
                                                chunk_index + 1,
                                                chunks.len(),
                                                embedding_data.embedding.len()
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "  Warning: Failed to embed chunk {}: {}",
                                        chunk_index + 1,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to process file '{}': {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    // Process text if provided
    if let Some(text_content) = text {
        println!("\n{} Processing text input...", "📝".blue());
        println!(
            "{} Text: \"{}\"",
            "📝".blue(),
            if text_content.len() > 50 {
                format!("{}...", &text_content[..50])
            } else {
                text_content.clone()
            }
        );

        let embedding_request = crate::provider::EmbeddingRequest {
            model: resolved_model.clone(),
            input: text_content.clone(),
            encoding_format: Some("float".to_string()),
        };

        match client.embeddings(&embedding_request).await {
            Ok(response) => {
                if let Some(embedding_data) = response.data.first() {
                    total_embeddings += 1;
                    total_tokens += response.usage.total_tokens;

                    println!(
                        "{} Vector dimensions: {}",
                        "📏".blue(),
                        embedding_data.embedding.len()
                    );

                    // Display vector preview
                    let embedding = &embedding_data.embedding;
                    if embedding.len() > 10 {
                        println!("\n{} Vector preview:", "🔍".blue());
                        print!("  [");
                        for (i, val) in embedding.iter().take(5).enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{:.6}", val);
                        }
                        print!(" ... ");
                        for (i, val) in embedding.iter().skip(embedding.len() - 5).enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{:.6}", val);
                        }
                        println!("]");
                    }

                    // Store in vector database if specified
                    if let Some(db_name) = &database {
                        match crate::vector_db::VectorDatabase::new(db_name) {
                            Ok(vector_db) => {
                                match vector_db.add_vector(
                                    &text_content,
                                    &embedding,
                                    &resolved_model,
                                    &provider_name,
                                ) {
                                    Ok(id) => {
                                        println!(
                                            "\n{} Stored in vector database '{}' with ID: {}",
                                            "💾".green(),
                                            db_name,
                                            id
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Failed to store in vector database: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to create/open vector database '{}': {}",
                                    db_name, e
                                );
                            }
                        }
                    }

                    // Output full vector as JSON for programmatic use
                    if files.is_empty() {
                        // Only show full vector for single text input
                        println!("\n{} Full vector (JSON):", "📋".dimmed());
                        println!("{}", serde_json::to_string(&embedding)?);
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to generate embeddings for text: {}", e);
            }
        }
    }

    // Summary
    println!("\n{} Embedding process completed!", "🎉".green());
    println!(
        "{} Total embeddings generated: {}",
        "📊".blue(),
        total_embeddings
    );
    println!("{} Total tokens used: {}", "💰".yellow(), total_tokens);

    if let Some(db_name) = &database {
        println!(
            "{} All embeddings stored in database: {}",
            "💾".green(),
            db_name
        );
    }

    Ok(())
}

// Similar command handler
pub async fn handle_similar_command(
    model: Option<String>,
    provider: Option<String>,
    database: String,
    limit: usize,
    query: String,
) -> Result<()> {
    use colored::Colorize;

    // Open the vector database
    let vector_db = crate::vector_db::VectorDatabase::new(&database)?;

    // Check if database has any vectors
    let count = vector_db.count()?;
    if count == 0 {
        anyhow::bail!(
            "Vector database '{}' is empty. Add some vectors first using 'lc embed -d {}'",
            database,
            database
        );
    }

    // Get model info from database if not provided
    let (resolved_model, resolved_provider) = match (&model, &provider) {
        (Some(m), Some(p)) => (m.clone(), p.clone()),
        _ => {
            if let Some((db_model, db_provider)) = vector_db.get_model_info()? {
                if model.is_some() || provider.is_some() {
                    println!(
                        "{} Using model from database: {}:{}",
                        "ℹ️".blue(),
                        db_provider,
                        db_model
                    );
                }
                (db_model, db_provider)
            } else {
                anyhow::bail!(
                    "No model specified and database '{}' has no stored model info",
                    database
                );
            }
        }
    };

    let config = config::Config::load()?;

    // Resolve provider and model
    let (provider_name, model_name) =
        resolve_model_and_provider(&config, Some(resolved_provider), Some(resolved_model))?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Generate embedding for query
    let embedding_request = crate::provider::EmbeddingRequest {
        model: model_name.clone(),
        input: query.clone(),
        encoding_format: Some("float".to_string()),
    };

    println!("{} Searching for similar content...", "🔍".blue());
    println!("{} Database: {}", "📊".blue(), database);
    println!(
        "{} Query: \"{}\"",
        "📝".blue(),
        if query.len() > 50 {
            format!("{}...", &query[..50])
        } else {
            query.clone()
        }
    );

    match client.embeddings(&embedding_request).await {
        Ok(response) => {
            if let Some(embedding_data) = response.data.first() {
                let query_vector = &embedding_data.embedding;

                // Find similar vectors
                let similar_results = vector_db.find_similar(query_vector, limit)?;

                if similar_results.is_empty() {
                    println!(
                        "\n{} No similar content found in database '{}'",
                        "❌".red(),
                        database
                    );
                } else {
                    println!(
                        "\n{} Found {} similar results:",
                        "✅".green(),
                        similar_results.len()
                    );

                    for (i, (entry, similarity)) in similar_results.iter().enumerate() {
                        let similarity_percent = (similarity * 100.0).round() as u32;
                        let similarity_color = if similarity_percent >= 80 {
                            format!("{}%", similarity_percent).green()
                        } else if similarity_percent >= 60 {
                            format!("{}%", similarity_percent).yellow()
                        } else {
                            format!("{}%", similarity_percent).red()
                        };

                        println!(
                            "\n{} {} (Similarity: {})",
                            format!("{}.", i + 1).bold(),
                            similarity_color,
                            format!("ID: {}", entry.id).dimmed()
                        );
                        println!("   {}", entry.text);
                        println!(
                            "   {}",
                            format!(
                                "Added: {}",
                                entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                            )
                            .dimmed()
                        );
                    }
                }
            } else {
                anyhow::bail!("No embedding data in response");
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to generate query embedding: {}", e);
        }
    }

    Ok(())
}

// Vectors command handler
pub async fn handle_vectors_command(command: crate::cli::VectorCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        crate::cli::VectorCommands::List => {
            let databases = crate::vector_db::VectorDatabase::list_databases()?;

            if databases.is_empty() {
                println!("No vector databases found.");
                println!(
                    "Create one by running: {}",
                    "lc embed -d <name> -m <model> \"your text\"".dimmed()
                );
            } else {
                println!("\n{} Vector databases:", "📊".bold().blue());

                for db_name in databases {
                    match crate::vector_db::VectorDatabase::new(&db_name) {
                        Ok(db) => {
                            let count = db.count().unwrap_or(0);
                            let model_info = db.get_model_info().unwrap_or(None);

                            print!("  {} {} ({} vectors)", "•".blue(), db_name.bold(), count);

                            if let Some((model, provider)) = model_info {
                                print!(" - {}:{}", provider.dimmed(), model.dimmed());
                            }

                            println!();
                        }
                        Err(_) => {
                            println!("  {} {} (error reading)", "•".red(), db_name.bold());
                        }
                    }
                }
            }
        }
        crate::cli::VectorCommands::Delete { name } => {
            // Check if database exists
            let databases = crate::vector_db::VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            crate::vector_db::VectorDatabase::delete_database(&name)?;
            println!(
                "{} Vector database '{}' deleted successfully",
                "✓".green(),
                name
            );
        }
        crate::cli::VectorCommands::Info { name } => {
            let databases = crate::vector_db::VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            let db = crate::vector_db::VectorDatabase::new(&name)?;
            let count = db.count()?;
            let model_info = db.get_model_info()?;

            println!("\n{} Vector database: {}", "📊".bold().blue(), name.bold());
            println!("Vectors: {}", count);

            if let Some((model, provider)) = model_info {
                println!("Model: {}:{}", provider, model);
            } else {
                println!("Model: {}", "Not set".dimmed());
            }

            if count > 0 {
                println!("\n{} Recent entries:", "📝".bold().blue());
                let vectors = db.get_all_vectors()?;
                for (i, entry) in vectors.iter().take(5).enumerate() {
                    let preview = if entry.text.len() > 60 {
                        format!("{}...", &entry.text[..60])
                    } else {
                        entry.text.clone()
                    };

                    let source_info = if let Some(ref file_path) = entry.file_path {
                        if let (Some(chunk_idx), Some(total_chunks)) =
                            (entry.chunk_index, entry.total_chunks)
                        {
                            format!(" [{}:{}/{}]", file_path, chunk_idx + 1, total_chunks)
                        } else {
                            format!(" [{}]", file_path)
                        }
                    } else {
                        String::new()
                    };

                    println!(
                        "  {}. {}{} ({})",
                        i + 1,
                        preview,
                        source_info.dimmed(),
                        entry
                            .created_at
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                            .dimmed()
                    );
                }

                if vectors.len() > 5 {
                    println!("  ... and {} more", vectors.len() - 5);
                }
            }
        }
    }

    Ok(())
}

// RAG helper function to retrieve relevant context
pub async fn retrieve_rag_context(
    db_name: &str,
    query: &str,
    _client: &crate::chat::LLMClient,
    _model: &str,
    _provider: &str,
) -> Result<String> {
    crate::debug_log!(
        "RAG: Starting context retrieval for database '{}' with query '{}'",
        db_name,
        query
    );

    // Open the vector database
    let vector_db = crate::vector_db::VectorDatabase::new(db_name)?;
    crate::debug_log!("RAG: Successfully opened vector database '{}'", db_name);

    // Check if database has any vectors
    let count = vector_db.count()?;
    crate::debug_log!("RAG: Database '{}' contains {} vectors", db_name, count);
    if count == 0 {
        crate::debug_log!("RAG: Database is empty, returning empty context");
        return Ok(String::new());
    }

    // Get model info from database
    let (db_model, db_provider) = if let Some((m, p)) = vector_db.get_model_info()? {
        crate::debug_log!("RAG: Using database model '{}' from provider '{}'", m, p);
        (m, p)
    } else {
        crate::debug_log!("RAG: No model info in database, returning empty context");
        return Ok(String::new());
    };

    // Create a client for the embedding provider (not the chat provider)
    let config = config::Config::load()?;
    let mut config_mut = config.clone();
    let embedding_client = chat::create_authenticated_client(&mut config_mut, &db_provider).await?;
    crate::debug_log!(
        "RAG: Created embedding client for provider '{}'",
        db_provider
    );

    // Use the database's embedding model for consistency
    let embedding_request = crate::provider::EmbeddingRequest {
        model: db_model.clone(),
        input: query.to_string(),
        encoding_format: Some("float".to_string()),
    };

    crate::debug_log!(
        "RAG: Generating embedding for query using model '{}'",
        db_model
    );

    // Generate embedding for query using the correct provider
    let response = embedding_client.embeddings(&embedding_request).await?;
    crate::debug_log!("RAG: Successfully generated embedding for query");

    if let Some(embedding_data) = response.data.first() {
        let query_vector = &embedding_data.embedding;
        crate::debug_log!("RAG: Query vector has {} dimensions", query_vector.len());

        // Find top 3 most similar vectors for context
        let similar_results = vector_db.find_similar(query_vector, 3)?;
        crate::debug_log!("RAG: Found {} similar results", similar_results.len());

        if similar_results.is_empty() {
            crate::debug_log!("RAG: No similar results found, returning empty context");
            return Ok(String::new());
        }

        // Format context
        let mut context = String::new();
        let mut included_count = 0;
        for (entry, similarity) in similar_results {
            crate::debug_log!(
                "RAG: Result similarity: {:.3} for text: '{}'",
                similarity,
                &entry.text[..50.min(entry.text.len())]
            );
            // Only include results with reasonable similarity (>0.3)
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }

        crate::debug_log!(
            "RAG: Included {} results in context (similarity > 0.3)",
            included_count
        );
        crate::debug_log!("RAG: Final context length: {} characters", context.len());

        Ok(context)
    } else {
        crate::debug_log!("RAG: No embedding data in response, returning empty context");
        Ok(String::new())
    }
}

// WebChatProxy command handlers
pub async fn handle_webchatproxy_command(command: WebChatProxyCommands) -> Result<()> {
    match command {
        WebChatProxyCommands::Providers { command } => match command {
            Some(WebChatProxyProviderCommands::List) => {
                handle_webchatproxy_providers_list().await?;
            }
            Some(WebChatProxyProviderCommands::Kagi { command }) => {
                handle_webchatproxy_kagi_command(command).await?;
            }
            None => {
                handle_webchatproxy_providers_list().await?;
            }
        },
        WebChatProxyCommands::Start {
            provider,
            port,
            host,
            key,
            generate_key,
            daemon,
        } => {
            handle_webchatproxy_start(provider, port, host, key, generate_key, daemon).await?;
        }
        WebChatProxyCommands::Stop { provider } => {
            handle_webchatproxy_stop(provider).await?;
        }
        WebChatProxyCommands::List => {
            handle_webchatproxy_list().await?;
        }
    }
    Ok(())
}

async fn handle_webchatproxy_providers_list() -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "Supported WebChatProxy Providers:".bold().blue());
    println!("  {} {} - Kagi Assistant API", "•".blue(), "kagi".bold());
    println!("\n{}", "Usage:".bold().blue());
    println!(
        "  {} Set auth: {}",
        "•".blue(),
        "lc w providers set kagi auth <token>".dimmed()
    );
    println!(
        "  {} Start proxy: {}",
        "•".blue(),
        "lc w start kagi".dimmed()
    );

    Ok(())
}

async fn fetch_kagi_models() -> Result<Vec<crate::webchatproxy::KagiModelProfile>> {
    crate::webchatproxy::fetch_kagi_models().await
}

async fn handle_webchatproxy_kagi_command(command: WebChatProxyKagiCommands) -> Result<()> {
    use colored::Colorize;
    use std::io::{self, Write};

    match command {
        WebChatProxyKagiCommands::Auth { token } => {
            let auth_token = if let Some(token) = token {
                token
            } else {
                print!("Enter authentication token for kagi: ");
                // Deliberately flush stdout to ensure prompt appears before password input
                io::stdout().flush()?;
                rpassword::read_password()?
            };

            // Store the auth token in webchatproxy config
            let mut config = crate::webchatproxy::WebChatProxyConfig::load()?;
            config.set_provider_auth("kagi", &auth_token)?;
            config.save()?;

            println!("{} Authentication set for provider 'kagi'", "✓".green());
        }
        WebChatProxyKagiCommands::Models => {
            // Fetch and display Kagi models
            match fetch_kagi_models().await {
                Ok(models) => {
                    println!("\n{} Available Kagi models:", "Models:".bold().blue());
                    for model in models {
                        let mut capabilities = Vec::new();
                        if model.internet_access {
                            capabilities.push("🌐 web".blue());
                        }
                        if model.personalizations {
                            capabilities.push("👤 personal".magenta());
                        }

                        let mut info_parts = Vec::new();
                        if let Some(ctx) = model.model_input_limit {
                            if ctx >= 1000000 {
                                info_parts.push(format!("{}m ctx", ctx / 1000000));
                            } else if ctx >= 1000 {
                                info_parts.push(format!("{}k ctx", ctx / 1000));
                            } else {
                                info_parts.push(format!("{} ctx", ctx));
                            }
                        }

                        print!(
                            "  {} {} ({})",
                            "•".blue(),
                            model.model_name.bold(),
                            model.model
                        );

                        if !capabilities.is_empty() {
                            let capability_strings: Vec<String> =
                                capabilities.iter().map(|c| c.to_string()).collect();
                            print!(" [{}]", capability_strings.join(" "));
                        }

                        if !info_parts.is_empty() {
                            print!(" ({})", info_parts.join(", ").dimmed());
                        }

                        if let Some(description) = &model.scorecard.description {
                            print!(" - {}", description.dimmed());
                        }

                        if model.scorecard.recommended {
                            print!(" {}", "⭐ recommended".yellow());
                        }

                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to fetch Kagi models: {}", "❌".red(), e);
                    eprintln!("Make sure you have set your Kagi authentication token with:");
                    eprintln!("  {}", "lc w p kagi auth".dimmed());
                }
            }
        }
    }

    Ok(())
}

async fn handle_webchatproxy_start(
    provider: String,
    port: u16,
    host: String,
    key: Option<String>,
    generate_key: bool,
    daemon: bool,
) -> Result<()> {
    use colored::Colorize;

    if provider != "kagi" {
        anyhow::bail!(
            "Unsupported provider '{}'. Currently only 'kagi' is supported.",
            provider
        );
    }

    // Generate API key if requested
    let final_key = if generate_key {
        let generated_key = crate::proxy::generate_api_key();
        println!(
            "{} Generated API key: {}",
            "🔑".green(),
            generated_key.bold()
        );
        Some(generated_key)
    } else {
        key
    };

    println!("\n{}", "WebChatProxy Server Configuration:".bold().blue());
    println!("  {} {}:{}", "Address:".bold(), host, port);
    println!("  {} {}", "Provider:".bold(), provider.green());

    if final_key.is_some() {
        println!("  {} {}", "Authentication:".bold(), "Enabled".green());
    } else {
        println!("  {} {}", "Authentication:".bold(), "Disabled".yellow());
    }

    println!("\n{}", "Available endpoints:".bold().blue());
    println!("  {} http://{}:{}/chat/completions", "•".blue(), host, port);
    println!(
        "  {} http://{}:{}/v1/chat/completions",
        "•".blue(),
        host,
        port
    );

    if daemon {
        println!("\n{} Starting in daemon mode...", "🔄".blue());
        println!(
            "{} Logs will be written to: ~/Library/Application Support/lc/{}.log",
            "📝".blue(),
            provider
        );

        // Start the webchatproxy server in daemon mode
        crate::webchatproxy::start_webchatproxy_daemon(host, port, provider.clone(), final_key)
            .await?;
    } else {
        println!("\n{} Press Ctrl+C to stop the server\n", "💡".yellow());

        // Start the webchatproxy server
        crate::webchatproxy::start_webchatproxy_server(host, port, provider, final_key).await?;
    }

    Ok(())
}

async fn handle_webchatproxy_stop(provider: String) -> Result<()> {
    use colored::Colorize;

    if provider != "kagi" {
        anyhow::bail!(
            "Unsupported provider '{}'. Currently only 'kagi' is supported.",
            provider
        );
    }

    println!(
        "{} Stopping webchatproxy server for '{}'...",
        "🛑".red(),
        provider
    );

    // Stop the webchatproxy daemon
    match crate::webchatproxy::stop_webchatproxy_daemon(&provider).await {
        Ok(_) => {
            println!(
                "{} WebChatProxy server for '{}' stopped successfully",
                "✓".green(),
                provider
            );
        }
        Err(e) => {
            println!(
                "{} Failed to stop WebChatProxy server for '{}': {}",
                "⚠️".yellow(),
                provider,
                e
            );
        }
    }

    Ok(())
}

async fn handle_webchatproxy_list() -> Result<()> {
    use colored::Colorize;

    println!("\n{} Running WebChatProxy servers:", "📊".bold().blue());

    // List running webchatproxy daemons
    match crate::webchatproxy::list_webchatproxy_daemons().await {
        Ok(servers) => {
            if servers.is_empty() {
                println!("No WebChatProxy servers currently running.");
            } else {
                for (provider, info) in servers {
                    println!(
                        "  {} {} - {}:{} (PID: {})",
                        "•".blue(),
                        provider.bold(),
                        info.host,
                        info.port,
                        info.pid
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{} Failed to list WebChatProxy servers: {}",
                "⚠️".yellow(),
                e
            );
        }
    }

    Ok(())
}

// Sync command handlers
pub async fn handle_sync_command(command: SyncCommands) -> Result<()> {
    match command {
        SyncCommands::Providers => crate::sync::handle_sync_providers().await,
        SyncCommands::Configure { provider, command } => {
            crate::sync::handle_sync_configure(&provider, command).await
        }
        SyncCommands::To {
            provider,
            encrypted,
            debug,
            yes,
        } => {
            // Set debug mode if requested
            if debug {
                set_debug_mode(true);
            }
            crate::sync::handle_sync_to(&provider, encrypted, yes).await
        }
        SyncCommands::From {
            provider,
            encrypted,
            debug,
            yes,
        } => {
            // Set debug mode if requested
            if debug {
                set_debug_mode(true);
            }
            crate::sync::handle_sync_from(&provider, encrypted, yes).await
        }
    }
}

// Search command handlers
pub async fn handle_search_command(command: SearchCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        SearchCommands::Provider { command } => handle_search_provider_command(command).await,
        SearchCommands::Query {
            provider,
            query,
            format,
            count,
        } => {
            let engine = crate::search::SearchEngine::new()?;

            println!(
                "{} Searching with {} for: '{}'",
                "🔍".blue(),
                provider.bold(),
                query
            );

            match engine.search(&provider, &query, Some(count)).await {
                Ok(results) => match format.as_str() {
                    "json" => {
                        println!("{}", engine.format_results_json(&results)?);
                    }
                    "md" | "markdown" => {
                        println!("{}", engine.format_results_markdown(&results));
                    }
                    _ => {
                        anyhow::bail!("Invalid format '{}'. Use 'json' or 'md'", format);
                    }
                },
                Err(e) => {
                    anyhow::bail!("Search failed: {}", e);
                }
            }

            Ok(())
        }
    }
}

async fn handle_search_provider_command(command: SearchProviderCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        SearchProviderCommands::Add { name, url } => {
            let mut config = crate::search::SearchConfig::load()?;

            // Auto-detect provider type from URL
            match config.add_provider_auto(name.clone(), url.clone()) {
                Ok(_) => {
                    config.save()?;

                    // Get the detected provider type for display
                    let provider_config = config.get_provider(&name)?;
                    let provider_type = &provider_config.provider_type;

                    println!(
                        "{} Search provider '{}' added successfully",
                        "✓".green(),
                        name
                    );
                    println!(
                        "  Type: {} (auto-detected)",
                        format!("{:?}", provider_type).to_lowercase()
                    );
                    println!("  URL: {}", url);

                    // Provider-specific instructions using the new API key header method
                    let api_key_header = provider_type.api_key_header();
                    if !api_key_header.is_empty() {
                        println!("\n{} Don't forget to set the API key:", "💡".yellow());
                        println!(
                            "  lc search provider set {} {} <your-api-key>",
                            name, api_key_header
                        );
                    } else {
                        println!("\n{} No API key required for this provider!", "✅".green());
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to add search provider: {}", e);
                }
            }
        }
        SearchProviderCommands::Delete { name } => {
            let mut config = crate::search::SearchConfig::load()?;
            config.delete_provider(&name)?;
            config.save()?;

            println!(
                "{} Search provider '{}' deleted successfully",
                "✓".green(),
                name
            );
        }
        SearchProviderCommands::Set {
            provider,
            header_name,
            header_value,
        } => {
            let mut config = crate::search::SearchConfig::load()?;
            config.set_header(&provider, header_name.clone(), header_value)?;
            config.save()?;

            println!(
                "{} Header '{}' set for search provider '{}'",
                "✓".green(),
                header_name,
                provider
            );
        }
        SearchProviderCommands::List => {
            let config = crate::search::SearchConfig::load()?;
            let providers = config.list_providers();

            if providers.is_empty() {
                println!("No search providers configured.");
                println!(
                    "Add one with: {}",
                    "lc search provider add <name> <url>".dimmed()
                );
            } else {
                println!("\n{}", "Search Providers:".bold().blue());

                for (name, provider_config) in providers {
                    let has_auth = provider_config.headers.contains_key("X-Subscription-Token")
                        || provider_config.headers.contains_key("Authorization")
                        || provider_config.headers.contains_key("x-api-key")
                        || provider_config.headers.contains_key("X-API-KEY");
                    let auth_status = if has_auth { "✓".green() } else { "✗".red() };

                    println!(
                        "  {} {} - {} (Auth: {})",
                        "•".blue(),
                        name.bold(),
                        provider_config.url,
                        auth_status
                    );

                    if !provider_config.headers.is_empty() {
                        println!("    Headers: {}", provider_config.headers.len());
                    }
                }

                if let Some(default) = config.get_default_provider() {
                    println!("\n{} {}", "Default provider:".bold(), default.green());
                }
            }
        }
    }

    Ok(())
}

// Helper function to integrate search results as context
async fn integrate_search_context(
    search_spec: &str,
    query: &str,
    enhanced_prompt: &mut String,
) -> Result<bool> {
    use colored::Colorize;

    // Parse search spec: can be "provider" or "provider:query"
    let (provider, search_query) = if search_spec.contains(':') {
        let parts: Vec<&str> = search_spec.splitn(2, ':').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        // Use the original prompt as the search query
        (search_spec.to_string(), query.to_string())
    };

    // Check if provider is configured
    let search_config = crate::search::SearchConfig::load()?;
    if !search_config.has_provider(&provider) {
        // Try to use default provider if available
        if let Some(default_provider) = search_config.get_default_provider() {
            if provider == "default" || provider.is_empty() {
                println!(
                    "{} Using default search provider: {}",
                    "🔍".blue(),
                    default_provider
                );
                return integrate_search_with_provider(
                    default_provider,
                    &search_query,
                    enhanced_prompt,
                )
                .await;
            }
        }
        anyhow::bail!(
            "Search provider '{}' not found. Configure it with 'lc search provider add'",
            provider
        );
    }

    integrate_search_with_provider(&provider, &search_query, enhanced_prompt).await
}

async fn integrate_search_with_provider(
    provider: &str,
    search_query: &str,
    enhanced_prompt: &mut String,
) -> Result<bool> {
    use colored::Colorize;

    let engine = crate::search::SearchEngine::new()?;

    println!("{} Searching for: '{}'", "🔍".blue(), search_query);

    match engine.search(provider, search_query, Some(5)).await {
        Ok(results) => {
            if results.results.is_empty() {
                println!("{} No search results found", "⚠️".yellow());
                return Ok(false);
            }

            println!(
                "{} Found {} search results",
                "✅".green(),
                results.results.len()
            );

            // Extract context from search results
            let search_context = engine.extract_context_for_llm(&results, 5);

            // Prepend search context to the enhanced prompt
            *enhanced_prompt = format!("{}\n\nUser query: {}", search_context, enhanced_prompt);

            Ok(true)
        }
        Err(e) => {
            anyhow::bail!("Search failed: {}", e);
        }
    }
}

// Image command handler
pub async fn handle_image_command(
    prompt: String,
    model: Option<String>,
    provider: Option<String>,
    size: String,
    count: u32,
    output: Option<String>,
    debug: bool,
) -> Result<()> {
    use colored::Colorize;
    use std::fs;
    use std::io::{self, Write};
    use std::path::Path;

    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }

    let config = config::Config::load()?;

    // Resolve provider and model using the same logic as other commands
    let (provider_name, model_name) = resolve_model_and_provider(&config, provider, model)?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    println!(
        "{} Generating {} image(s) with prompt: \"{}\"",
        "🎨".blue(),
        count,
        prompt
    );
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏢".blue(), provider_name);
    println!("{} Size: {}", "📐".blue(), size);

    // Create image generation request
    let image_request = crate::provider::ImageGenerationRequest {
        prompt: prompt.clone(),
        model: Some(model_name.clone()),
        n: Some(count),
        size: Some(size.clone()),
        quality: Some("standard".to_string()),
        style: None,
        response_format: Some("url".to_string()),
    };

    // Generate images
    print!("{} ", "Generating...".dimmed());
    io::stdout().flush()?;

    match client.generate_images(&image_request).await {
        Ok(response) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Generating..."
            println!(
                "{} Successfully generated {} image(s)!",
                "✅".green(),
                response.data.len()
            );

            // Create output directory if specified
            let output_dir = if let Some(dir) = output {
                let path = Path::new(&dir);
                if !path.exists() {
                    fs::create_dir_all(path)?;
                    println!("{} Created output directory: {}", "📁".blue(), dir);
                }
                Some(dir)
            } else {
                None
            };

            // Process each generated image
            for (i, image_data) in response.data.iter().enumerate() {
                let image_num = i + 1;

                if let Some(url) = &image_data.url {
                    println!(
                        "\n{} Image {}/{}",
                        "🖼️".blue(),
                        image_num,
                        response.data.len()
                    );
                    println!("   URL: {}", url);

                    if let Some(revised_prompt) = &image_data.revised_prompt {
                        if revised_prompt != &prompt {
                            println!("   Revised prompt: {}", revised_prompt.dimmed());
                        }
                    }

                    // Download image if output directory is specified
                    if let Some(ref dir) = output_dir {
                        let filename = format!(
                            "image_{}_{}.png",
                            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                            image_num
                        );
                        let filepath = Path::new(dir).join(&filename);

                        match download_image(url, &filepath).await {
                            Ok(_) => {
                                println!("   {} Saved to: {}", "💾".green(), filepath.display());
                            }
                            Err(e) => {
                                eprintln!("   {} Failed to download image: {}", "❌".red(), e);
                            }
                        }
                    }
                } else if let Some(b64_data) = &image_data.b64_json {
                    println!(
                        "\n{} Image {}/{} (Base64)",
                        "🖼️".blue(),
                        image_num,
                        response.data.len()
                    );

                    // For base64 data, always save to a file (either specified output dir or current dir)
                    let save_dir = output_dir.as_deref().unwrap_or(".");
                    let filename = format!(
                        "image_{}_{}.png",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        image_num
                    );
                    let filepath = Path::new(save_dir).join(&filename);

                    match save_base64_image(b64_data, &filepath) {
                        Ok(_) => {
                            println!("   {} Saved to: {}", "💾".green(), filepath.display());
                        }
                        Err(e) => {
                            eprintln!("   {} Failed to save image: {}", "❌".red(), e);
                        }
                    }

                    if let Some(revised_prompt) = &image_data.revised_prompt {
                        if revised_prompt != &prompt {
                            println!("   Revised prompt: {}", revised_prompt.dimmed());
                        }
                    }
                }
            }

            if output_dir.is_none() {
                // Check if we had any URL-based images that weren't downloaded
                let has_url_images = response.data.iter().any(|img| img.url.is_some());
                if has_url_images {
                    println!(
                        "\n{} Use --output <directory> to automatically download URL-based images",
                        "💡".yellow()
                    );
                }
            }
        }
        Err(e) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Generating..."
            anyhow::bail!("Failed to generate images: {}", e);
        }
    }

    Ok(())
}

// Helper function to download image from URL
async fn download_image(url: &str, filepath: &std::path::Path) -> Result<()> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    std::fs::write(filepath, bytes)?;

    Ok(())
}

// Helper function to save base64 image data
fn save_base64_image(b64_data: &str, filepath: &std::path::Path) -> Result<()> {
    use base64::{engine::general_purpose, Engine as _};

    let image_bytes = general_purpose::STANDARD.decode(b64_data)?;
    std::fs::write(filepath, image_bytes)?;

    Ok(())
}

// Audio transcription command handler
pub async fn handle_transcribe_command(
    audio_files: Vec<String>,
    model: Option<String>,
    provider: Option<String>,
    language: Option<String>,
    prompt: Option<String>,
    format: String,
    temperature: Option<f32>,
    output: Option<String>,
    debug: bool,
) -> Result<()> {
    use crate::audio_utils;
    use colored::Colorize;
    use std::io::{self, Write};

    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }

    if audio_files.is_empty() {
        anyhow::bail!("No audio files provided for transcription");
    }

    let config = config::Config::load()?;

    // Default to whisper-1 model if not specified
    let model_str = model.unwrap_or_else(|| "whisper-1".to_string());
    
    // Resolve provider and model
    let (provider_name, model_name) = if let Some(p) = provider {
        (p, model_str)
    } else {
        // Try to find a provider that has the whisper model
        let provider_name = config
            .providers
            .iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("whisper")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "openai".to_string());
        (provider_name, model_str)
    };

    // Get provider config with authentication
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    println!(
        "{} Transcribing {} audio file(s)",
        "🎤".blue(),
        audio_files.len()
    );
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏢".blue(), provider_name);
    if let Some(ref lang) = language {
        println!("{} Language: {}", "🌐".blue(), lang);
    }
    println!("{} Format: {}", "📄".blue(), format);

    let mut all_transcriptions = Vec::new();

    for (i, audio_file) in audio_files.iter().enumerate() {
        println!(
            "\n{} Processing file {}/{}: {}",
            "📁".blue(),
            i + 1,
            audio_files.len(),
            audio_file
        );

        print!("{} ", "Transcribing...".dimmed());
        io::stdout().flush()?;

        // Process audio file (handles both local files and URLs)
        let audio_data = if audio_file.starts_with("http://") || audio_file.starts_with("https://") {
            audio_utils::process_audio_url(audio_file)?
        } else {
            audio_utils::process_audio_file(std::path::Path::new(audio_file))?
        };

        // Create transcription request
        let transcription_request = crate::provider::AudioTranscriptionRequest {
            file: audio_data,
            model: model_name.clone(),
            language: language.clone(),
            prompt: prompt.clone(),
            response_format: Some(format.clone()),
            temperature,
        };

        // Transcribe audio
        match client.transcribe_audio(&transcription_request).await {
            Ok(response) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Transcribing..."
                println!("{} Transcription complete!", "✅".green());
                
                // Display or save transcription
                let transcription_text = response.text;
                
                if let Some(ref output_file) = output {
                    // Append to output file if multiple files
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output_file)?;
                    
                    if audio_files.len() > 1 {
                        writeln!(file, "\n=== {} ===", audio_file)?;
                    }
                    writeln!(file, "{}", transcription_text)?;
                    
                    all_transcriptions.push(transcription_text);
                } else {
                    // Print to stdout
                    if audio_files.len() > 1 {
                        println!("\n{} Transcription for {}:", "📝".blue(), audio_file);
                    } else {
                        println!("\n{} Transcription:", "📝".blue());
                    }
                    println!("{}", transcription_text);
                    
                    all_transcriptions.push(transcription_text);
                }
            }
            Err(e) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Transcribing..."
                eprintln!("{} Failed to transcribe {}: {}", "❌".red(), audio_file, e);
            }
        }
    }

    if let Some(output_file) = output {
        println!(
            "\n{} All transcriptions saved to: {}",
            "💾".green(),
            output_file
        );
    }

    Ok(())
}

// Text-to-speech command handler
pub async fn handle_tts_command(
    text: String,
    model: Option<String>,
    provider: Option<String>,
    voice: String,
    format: String,
    speed: Option<f32>,
    output: String,
    debug: bool,
) -> Result<()> {
    use colored::Colorize;
    use std::io::{self, Write};

    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }

    let config = config::Config::load()?;

    // Default to tts-1 model if not specified
    let model_str = model.unwrap_or_else(|| "tts-1".to_string());
    
    // Resolve provider and model
    let (provider_name, model_name) = if let Some(p) = provider {
        (p, model_str)
    } else {
        // Try to find a provider that has TTS models
        let provider_name = config
            .providers
            .iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("tts")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "openai".to_string());
        (provider_name, model_str)
    };

    // Get provider config with authentication
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    if provider_config.api_key.is_none() {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Truncate text for display if it's too long
    let display_text = if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text.clone()
    };

    println!("{} Generating speech", "🔊".blue());
    println!("{} Text: \"{}\"", "📝".blue(), display_text);
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏢".blue(), provider_name);
    println!("{} Voice: {}", "🎭".blue(), voice);
    println!("{} Format: {}", "🎵".blue(), format);
    if let Some(s) = speed {
        println!("{} Speed: {}x", "⚡".blue(), s);
    }

    print!("{} ", "Generating speech...".dimmed());
    io::stdout().flush()?;

    // Create TTS request
    let tts_request = crate::provider::AudioSpeechRequest {
        model: model_name,
        input: text,
        voice,
        response_format: Some(format.clone()),
        speed,
    };

    // Generate speech
    match client.generate_speech(&tts_request).await {
        Ok(audio_bytes) => {
            print!("\r{}\r", " ".repeat(25)); // Clear "Generating speech..."
            
            // Save audio to file
            std::fs::write(&output, audio_bytes)?;
            
            println!(
                "{} Speech generated successfully!",
                "✅".green()
            );
            println!("{} Saved to: {}", "💾".green(), output);
            
            // Show file size
            let metadata = std::fs::metadata(&output)?;
            let size_kb = metadata.len() as f64 / 1024.0;
            println!("{} File size: {:.2} KB", "📊".blue(), size_kb);
        }
        Err(e) => {
            print!("\r{}\r", " ".repeat(25)); // Clear "Generating speech..."
            anyhow::bail!("Failed to generate speech: {}", e);
        }
    }

    Ok(())
}

// Dump metadata command handler
pub async fn handle_dump_metadata_command(provider: Option<String>, list: bool) -> Result<()> {
    use crate::dump_metadata::MetadataDumper;

    if list {
        // List available cached metadata files
        MetadataDumper::list_cached_metadata().await?;
    } else if let Some(provider_name) = provider {
        // Dump metadata for specific provider
        MetadataDumper::dump_provider_by_name(&provider_name).await?;
    } else {
        // Dump metadata for all providers
        MetadataDumper::dump_all_metadata().await?;
    }

    Ok(())
}

// Usage command handler
pub async fn handle_usage_command(
    command: Option<UsageCommands>,
    days: Option<u32>,
    tokens_only: bool,
    requests_only: bool,
    limit: usize,
) -> Result<()> {
    use crate::usage_stats::{UsageAnalyzer, BarChart, display_usage_overview};
    use colored::Colorize;

    let analyzer = UsageAnalyzer::new()?;
    let stats = analyzer.get_usage_stats(days)?;

    if stats.total_requests == 0 {
        println!("{} No usage data found", "ℹ️".blue());
        if days.is_some() {
            println!("Try expanding the time range or check if you have any logged interactions.");
        }
        return Ok(());
    }

    match command {
        Some(UsageCommands::Daily { count }) => {
            let value_type = if tokens_only {
                "tokens"
            } else if requests_only {
                "requests"
            } else {
                "tokens"
            };
            
            BarChart::render_time_series(
                "📅 Daily Usage",
                &stats.daily_usage,
                value_type,
                50,
                count.min(limit),
            );
        }
        Some(UsageCommands::Weekly { count }) => {
            let value_type = if tokens_only {
                "tokens"
            } else if requests_only {
                "requests"
            } else {
                "tokens"
            };
            
            BarChart::render_time_series(
                "📊 Weekly Usage",
                &stats.weekly_usage,
                value_type,
                50,
                count.min(limit),
            );
        }
        Some(UsageCommands::Monthly { count }) => {
            let value_type = if tokens_only {
                "tokens"
            } else if requests_only {
                "requests"
            } else {
                "tokens"
            };
            
            BarChart::render_time_series(
                "📈 Monthly Usage",
                &stats.monthly_usage,
                value_type,
                50,
                count.min(limit),
            );
        }
        Some(UsageCommands::Yearly { count }) => {
            let value_type = if tokens_only {
                "tokens"
            } else if requests_only {
                "requests"
            } else {
                "tokens"
            };
            
            BarChart::render_time_series(
                "📊 Yearly Usage",
                &stats.yearly_usage,
                value_type,
                50,
                count.min(limit),
            );
        }
        Some(UsageCommands::Models { count }) => {
            let value_type = if tokens_only {
                "tokens"
            } else if requests_only {
                "requests"
            } else {
                "tokens"
            };
            
            BarChart::render_horizontal(
                "🤖 Top Models by Usage",
                &stats.model_usage,
                value_type,
                50,
                count.min(limit),
            );
        }
        None => {
            // Default: show overview and top charts
            display_usage_overview(&stats);

            if !tokens_only && !requests_only {
                // Show both tokens and requests by default
                BarChart::render_horizontal(
                    "🤖 Top Models by Token Usage",
                    &stats.model_usage,
                    "tokens",
                    50,
                    limit.min(5),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Usage (Tokens)",
                    &stats.daily_usage,
                    "tokens",
                    50,
                    limit.min(14),
                );
            } else if tokens_only {
                BarChart::render_horizontal(
                    "🤖 Top Models by Token Usage",
                    &stats.model_usage,
                    "tokens",
                    50,
                    limit.min(10),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Token Usage",
                    &stats.daily_usage,
                    "tokens",
                    50,
                    limit.min(14),
                );
            } else if requests_only {
                BarChart::render_horizontal(
                    "🤖 Top Models by Request Count",
                    &stats.model_usage,
                    "requests",
                    50,
                    limit.min(10),
                );

                BarChart::render_time_series(
                    "📅 Recent Daily Request Count",
                    &stats.daily_usage,
                    "requests",
                    50,
                    limit.min(14),
                );
            }
        }
    }

    Ok(())
}

// Completion generation handler
pub async fn handle_completions_command(shell: CompletionShell) -> Result<()> {
    crate::completion::generate_completions(shell).await
}

// Custom completion functions for dynamic values
#[allow(dead_code)]
pub fn complete_providers() -> Vec<String> {
    crate::completion::get_available_providers()
}

#[allow(dead_code)]
pub fn complete_models() -> Vec<String> {
    crate::completion::get_available_models()
}

// Include test module
#[cfg(test)]
mod tests;
