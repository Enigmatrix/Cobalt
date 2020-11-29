use crate::error::*;

pub use tracing::*;

pub fn setup_log() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE) // TODO set based on config
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )
    .with_context(|| "Setup global tracing subscriber")
}
