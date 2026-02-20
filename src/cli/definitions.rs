//! CLI definitions and command structures
//! This file contains all the CLI struct and enum definitions

use clap::{Parser, Subcommand};

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
    #[arg(short = 'p', long = "provider", global = true)]
    pub provider: Option<String>,

    /// Model to use for the prompt
    #[arg(short = 'm', long = "model", global = true)]
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

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum McpServerType {
    /// Standard I/O based MCP server
    Stdio,
    /// Server-Sent Events MCP server
    Sse,
    /// Streamable HTTP MCP server
    Streamable,
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

// Command enums
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
    /// Set streaming output preference (alias: st)
    #[command(alias = "st")]
    Stream {
        /// Stream output (true/false)
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
    /// Get streaming output preference (alias: st)
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
    /// Delete streaming output preference (alias: st)
    #[command(alias = "st")]
    Stream,
}

#[derive(Subcommand)]
pub enum VectorCommands {
    /// List all vector databases (alias: l)
    #[command(alias = "l")]
    List,
    /// Create a new vector database (alias: c)
    #[command(alias = "c")]
    Create {
        /// Database name
        name: String,
    },
    /// Delete a vector database (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Database name
        name: String,
        /// Confirm deletion without prompt
        #[arg(long)]
        yes: bool,
    },
    /// Show database information (alias: i)
    #[command(alias = "i")]
    Info {
        /// Database name
        name: String,
    },
    /// Show database statistics (alias: s)
    #[command(alias = "s")]
    Stats {
        /// Database name
        name: String,
    },
    /// Clear all embeddings from database (alias: cl)
    #[command(alias = "cl")]
    Clear {
        /// Database name
        name: String,
        /// Confirm clear without prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum WebChatProxyCommands {
    /// Start web chat proxy server (alias: s)
    #[command(alias = "s")]
    Start {
        /// Port to listen on
        #[arg(short = 'p', long = "port", default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long = "host", default_value = "127.0.0.1")]
        host: String,
        /// Enable CORS for cross-origin requests
        #[arg(long = "cors")]
        cors: bool,
    },
}

#[derive(Subcommand)]
pub enum SearchCommands {
    /// Manage search providers (alias: p)
    #[command(alias = "p")]
    Provider {
        #[command(subcommand)]
        command: SearchProviderCommands,
    },
    /// Query a search provider directly (alias: q)
    #[command(alias = "q")]
    Query {
        /// Search provider name
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
    /// Add a search provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
        /// Provider URL (auto-detects type)
        url: String,
    },
    /// List all search providers (alias: l)
    #[command(alias = "l")]
    List,
    /// Delete a search provider (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Provider name
        name: String,
    },
    /// Set provider headers/configuration (alias: s)
    #[command(alias = "s")]
    Set {
        /// Provider name
        provider: String,
        /// Header name (e.g., X-API-KEY, Authorization)
        header_name: String,
        /// Header value
        header_value: String,
    },
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
    /// Start an MCP server (alias: s)
    #[command(alias = "s")]
    Start {
        /// Server name
        name: String,
        /// Server command (optional - uses stored configuration if not provided)
        command: Option<String>,
        /// Server arguments
        #[arg(short = 'a', long = "args")]
        args: Vec<String>,
    },
    /// Show MCP server status (alias: st)
    #[command(alias = "stat")]
    Status {
        /// Server name (optional, shows all if not specified)
        name: Option<String>,
    },
}
