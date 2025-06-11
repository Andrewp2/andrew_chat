//! This crate contains all shared UI components for the workspace.

mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

mod echo;
pub use echo::Echo;

// Re-export ModelConfig from the API crate
pub use api::model_config::ModelConfig;
