use crate::error::*;

pub use tracing::*;

pub fn setup_log() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::from(crate::config::Config::instance().log.level))
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )
    .with_context(|| "Setup global tracing subscriber")
}
