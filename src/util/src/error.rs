use std::panic;

use color_eyre::config;
pub use color_eyre::eyre::*;

use crate::tracing::error;

/// Setup error handling
pub fn setup() -> color_eyre::Result<()> {
    // the equivalent of color_eyre::install()

    #[cfg(debug_assertions)]
    let (panic_hook, eyre_hook) = config::HookBuilder::default().into_hooks();
    #[cfg(not(debug_assertions))]
    let (panic_hook, eyre_hook) = config::HookBuilder::default()
        .theme(config::Theme::new()) // blank theme to disable color
        .into_hooks();
    eyre_hook.install()?;
    // panic_hook.install();

    // don't install the panic hook, we'll do it manually:
    // add a custom panic hook that logs the panic and exits
    panic::set_hook(Box::new(move |panic_info| {
        let report = panic_hook.panic_report(panic_info).to_string();
        error!("unhandled panic: {report}");
        eprintln!("{report}");
        std::process::exit(1);
    }));
    Ok(())
}
