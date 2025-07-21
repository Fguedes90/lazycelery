//! Application module providing state management and business logic for LazyCelery.
//!
//! This module is organized into separate concerns:
//! - `state`: Core application state, navigation, and UI state management
//! - `actions`: Business logic for broker operations and user actions

mod actions;
mod state;

// Re-export the main types for convenience
pub use state::{AppState, Tab};

// Create a type alias for backward compatibility
pub type App = AppState;
