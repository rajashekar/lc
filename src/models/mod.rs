// Model-related modules
pub mod cache;
pub mod dump_metadata;
pub mod metadata;
pub mod unified_cache;

// Re-export with old names for compatibility
pub use cache as models_cache;
pub use metadata as model_metadata;
