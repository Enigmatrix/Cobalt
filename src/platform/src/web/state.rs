use std::sync::Arc;

use util::ds::{SmallHashMap, SmallHashSet};
use util::future::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::objects::{ProcessId, Window};

/// Information about a browser window
#[derive(Debug, Default, Clone)]
pub struct BrowserWindowInfo {
    /// Whether the browser window is in incognito mode
    pub is_incognito: bool,
}

/// Shared inner state of browsers and websites seen in the desktop
#[derive(Debug, Default)]
pub struct StateInner {
    /// Cache of whether a browser window states.
    /// If present and None, this is not a browser window.
    /// If present and Some, this is a browser window.
    /// Not present means that we don't know if it's a browser or not.
    pub browser_windows: SmallHashMap<Window, Option<BrowserWindowInfo>>,
    /// Processes that are known to be browsers.
    pub browser_processes: SmallHashSet<ProcessId>,
}

/// Shared state of browsers and websites seen in the desktop
pub type State = Arc<RwLock<StateInner>>;

/// Write locked state of [State]
pub type WriteLockedState<'a> = RwLockWriteGuard<'a, StateInner>;
/// Read locked state of [State]
pub type ReadLockedState<'a> = RwLockReadGuard<'a, StateInner>;

/// Default state of [State]
pub fn default_state() -> State {
    Arc::new(RwLock::new(StateInner::default()))
}
