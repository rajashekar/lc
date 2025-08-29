// Model-related modules
pub mod metadata;
pub mod cache;
pub mod unified_cache;
pub mod dump_metadata;

// Re-export with old names for compatibility
pub use metadata as model_metadata;
pub use cache as models_cache;