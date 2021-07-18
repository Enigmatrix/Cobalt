use crate::error::*;

pub use tracing::*;
use tracing_subscriber::EnvFilter;

pub fn setup_log() -> Result<()> {
    let filter = EnvFilter::new("runner,data,util,h2=info");

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            // .with_max_level(Level::from(crate::config::Config::instance().log.level))
            .pretty()
            .with_env_filter(filter)
            // .with_thread_ids(true)
            // .compact()
            .finish(),
    )
    .wrap_err("setup global tracing subscriber")
}