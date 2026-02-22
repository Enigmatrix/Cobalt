use std::collections::HashMap;
use std::sync::Arc;

use util::error::Result;

use super::actions::BrowserAction;
use super::website_info::BaseWebsiteUrl;
use crate::objects::Window;

/// The browser state for a window (URL, incognito status).
#[derive(Debug, Clone)]
pub struct BrowserWindowInfo {
    /// Current URL of the active tab.
    pub url: String,
    /// Whether the window is in incognito / private mode.
    pub is_incognito: bool,
}

/// Extra info resolved during detection.
#[derive(Debug, Clone, Default)]
pub struct ResolvedWindowInfo {
    /// Process executable path, if fetched during detection.
    pub exe_path: Option<String>,
}

/// Full result of a [`Browser::identify`] call.
#[derive(Debug, Clone)]
pub struct IdentifyResult {
    /// Current browser window state.
    pub info: BrowserWindowInfo,
    /// Additional info resolved during detection.
    pub resolved_info: ResolvedWindowInfo,
}

/// Trait for interacting with browsers.
///
/// Consumers hold [`ArcBrowser`] (`Arc<dyn Browser>`) for thread-safe sharing.
///
/// # `identify()` dual behaviour
///
/// - **First call** for a window: full detection + probe (expensive).
///   `resolved_info` may contain the exe path, etc.
/// - **Subsequent calls**: returns cached info (cheap), kept up-to-date by an
///   internal watcher. `resolved_info` is [`Default`] (empty).
pub trait Browser: Send + Sync + 'static {
    /// Identify whether a window is a browser and extract its current state.
    ///
    /// Returns `None` if the window is not a browser or not a valid content
    /// window (e.g. a Ctrl-F dialog).
    fn identify(&self, window: &Window) -> Result<Option<IdentifyResult>>;

    /// Declaratively set the URL-to-action mapping.
    ///
    /// The backend stores this and applies matching actions when tab changes
    /// are detected internally. Replaces the previous mapping entirely.
    fn set_actions(&self, actions: HashMap<BaseWebsiteUrl, BrowserAction>) -> Result<()>;

    /// Periodic tick driven by the event loop.
    ///
    /// Used for watcher subscription sync, stale-entry cleanup and other
    /// backend-specific housekeeping.
    fn tick(&self) -> Result<()>;
}

/// Shared browser backend reference.
pub type ArcBrowser = Arc<dyn Browser>;
