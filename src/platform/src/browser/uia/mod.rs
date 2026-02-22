pub(crate) mod detect;
pub(crate) mod state;
pub(crate) mod watcher;

// Backward-compat re-exports (temporary, removed in step 6)
pub use detect::*;
pub use state::*;
pub use watcher::*;
