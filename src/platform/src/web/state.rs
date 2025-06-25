use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use util::future::sync::RwLock;

use crate::objects::{ProcessId, Window};

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
pub struct StateInner {
    /// Cache of whether a window is a browser or not.
    /// Not present means that we don't know if it's a browser or not.
    pub browser_windows: HashMap<Window, bool>,
    /// Processes that are known to be browsers.
    pub browser_processes: HashSet<ProcessId>,
}

/// Shared state of browsers and websites seen in the desktop
pub type State = Arc<RwLock<StateInner>>;

/// Default state of [State]
pub fn default_state() -> State {
    Arc::new(RwLock::new(StateInner::default()))
}
