use std::thread;

use resolver::{AppInfoResolver, AppInfoResolverRequest};
use util::{
    channels::Receiver,
    error::Result,
    future::{
        executor::{LocalPool, LocalSpawner},
        task::SpawnExt,
    },
};

mod engine;
mod resolver;
mod sentry;

fn main() -> Result<()> {
    let config = util::config::get_config()?;
    util::setup(&config)?;

    let (tx, rx) = util::channels::unbounded();

    let resolver = thread::spawn(|| resolver_thread(rx));

    println!("Hello, world!");
    resolver.join().expect("resolver: no panic")?;
    Ok(())
}

fn resolver_thread(rx: Receiver<AppInfoResolverRequest>) -> Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    async fn resolve_loop(
        rx: Receiver<AppInfoResolverRequest>,
        spawner: LocalSpawner,
    ) -> Result<()> {
        loop {
            let req = rx.recv_async().await?;
            spawner.spawn(async {
                AppInfoResolver::update_app(req)
                    .await
                    .expect("update app with info");
            })?;
        }
    }
    pool.run_until(resolve_loop(rx, spawner))
        .expect("resolver loop");
    Ok(())
}
