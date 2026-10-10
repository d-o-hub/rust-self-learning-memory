//! Embedding tools execute implementations split into modules.

mod configure;
mod generate;
mod query;
mod status;

pub use configure::configured_embedding_storage;
