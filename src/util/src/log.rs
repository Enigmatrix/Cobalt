use crate::error::*;

pub use tracing::*;

pub fn setup_log() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(super::config::Config::instance().log_level)
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )
    .with_context(|| "Setup global tracing subscriber")
}
