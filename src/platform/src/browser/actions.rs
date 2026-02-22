/// An action the backend applies when a browser tab matches a URL.
#[derive(Debug, Clone)]
pub enum BrowserAction {
    /// Close the matching tab.
    CloseTab,
    /// Dim the window to the given opacity (0.0 = fully dimmed, 1.0 = fully visible).
    Dim {
        /// Target opacity level.
        opacity: f64,
    },
}
