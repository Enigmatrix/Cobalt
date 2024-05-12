use std::thread;

use resolver::{AppInfoResolver, AppInfoResolverRequest};
use util::{
    channels::Receiver,
    config::Config,
    error::Result,
    future::{
        executor::{LocalPool, LocalSpawner},
        task::LocalSpawnExt,
    },
};

mod engine;
mod resolver;
mod sentry;

fn main() -> Result<()> {
    let config = util::config::get_config()?;
    util::setup(&config)?;

    let (tx, rx) = util::channels::unbounded();

    let resolver = {
        let config = config.clone();
        thread::spawn(move || resolver_thread(&config, rx))
    };

    println!("Hello, world!");
    resolver.join().expect("resolver: no panic")?;
    Ok(())
}

fn resolver_thread(config: &Config, rx: Receiver<AppInfoResolverRequest>) -> Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    async fn resolve_loop(
        config: &Config,
        rx: Receiver<AppInfoResolverRequest>,
        spawner: LocalSpawner,
    ) -> Result<()> {
        loop {
            let req = rx.recv_async().await?;
            let config = config.clone();
            spawner.spawn_local(async move {
                AppInfoResolver::update_app(&config, req)
                    .await
                    .expect("update app with info");
            })?;
        }
    }
    pool.run_until(resolve_loop(config, rx, spawner))
        .expect("resolver loop");
    Ok(())
}
