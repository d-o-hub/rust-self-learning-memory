//! Progressive Configuration Modes
//!
//! This module implements progressive configuration modes that address the
//! "choice overload" issue by providing clear progression from simple to advanced.
//!
//! # Progressive Disclosure Design
//!
//! The system provides three distinct modes with clear progression:
//!
//! 1. **Ultra-Simple Mode (30-second setup)**: One function call for basic redb usage
//! 2. **Simple Mode (3-5 function calls)**: Clear preset selection with guided overrides
//! 3. **Advanced Mode**: Interactive wizard for comprehensive setup
//!
//! # Usage Examples
//!
//! ## Ultra-Simple (30-second setup)
//!
//! ```no_run
//! use memory_cli::config::progressive::setup_quick_redb;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = setup_quick_redb().await?;
//!     println!("✅ Ready in under 30 seconds!");
//!     Ok(())
//! }
//! ```
//!
//! ## Simple Mode (Guided preset selection)
//!
//! ```no_run
//! use memory_cli::config::{SimpleSetup, ConfigPreset};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SimpleSetup::preset(ConfigPreset::Local)
//!         .with_custom(|c| {
//!             // Easy customization
//!             c.storage.max_episodes_cache = 2000;
//!
//!             // Auto-validation with helpful error messages
//!         })
//!         .build()
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Mode (Full configuration)
//!
//! ```no_run
//! use memory_cli::config::quick_setup;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = quick_setup().await?;
//!     Ok(())
//! }
//! ```

pub mod modes;
pub mod quick_setup;
pub mod simple_setup;

pub use modes::{ConfigurationMode, ModeRecommendation, UsagePattern, recommend_mode};
pub use quick_setup::setup_quick_redb;
pub use simple_setup::SimpleSetup;
