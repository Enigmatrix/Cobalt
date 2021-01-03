mod raw;

pub mod dto {
    pub use super::raw::{AppId, Empty, UsageSwitch};
}

use dto::*;
use raw::relay_server::{Relay, RelayServer};
use tonic::*;
use util::futures::sync::{broadcast, mpsc};
use util::*;
use crate::data::model;

#[derive(Debug)]
pub struct RelayServiceTx {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
    app_updates_tx: broadcast::Sender<model::Id>,
}

#[derive(Debug)]
pub struct RelayService {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
    usage_switches_rx: broadcast::Receiver<UsageSwitch>,
    app_updates_tx: broadcast::Sender<model::Id>,
    app_updates_rx: broadcast::Receiver<model::Id>,
}

impl RelayServiceTx {
    pub fn push_usage_switch(&self, us: UsageSwitch) -> Result<()> {
        self.usage_switches_tx
            .send(us)
            //.with_context(|| "Send Usage Switch")?;
            .unwrap(); // TODO better result!
        Ok(())
    }

    pub fn push_app_update(&self, app_id: model::Id) -> Result<()> {
        self.app_updates_tx
            .send(app_id)
            //.with_context(|| "Send App Update")?;
            .unwrap(); // TODO better result!
        Ok(())
    }
}

impl RelayService {
    pub fn new() -> (RelayServiceTx, RelayService) {
        let (usage_switches_tx, usage_switches_rx) = broadcast::channel(1);
        let usage_switches_tx2 = usage_switches_tx.clone();

        let (app_updates_tx, app_updates_rx) = broadcast::channel(1);
        let app_updates_tx2 = app_updates_tx.clone();

        (
            RelayServiceTx { usage_switches_tx, app_updates_tx },
            RelayService {
                usage_switches_tx: usage_switches_tx2,
                usage_switches_rx,
                app_updates_tx: app_updates_tx2,
                app_updates_rx,
            },
        )
    }

    pub async fn serve(self) -> Result<()> {
        let addr = "[::1]:50051".parse()?;

        transport::Server::builder()
            .add_service(RelayServer::new(self))
            .serve(addr)
            .await
            .with_context(|| "Serving EngineWorker")?;

        Ok(())
    }
}

#[tonic::async_trait]
impl Relay for RelayService {
    type UsagesStream = mpsc::Receiver<Result<UsageSwitch, Status>>;
    type AppUpdatesStream = mpsc::Receiver<Result<AppId, Status>>;

    async fn usages(&self, _: Request<Empty>) -> Result<Response<Self::UsagesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);
        let mut recver = self.usage_switches_tx.subscribe();

        futures::spawn(async move {
            loop {
                let us = recver
                    .recv()
                    .await
                    .map_err(|e| Status::internal(e.to_string()));

                if let Err(_) = tx.send(us).await {
                    log::warn!("ending Usages stream.."); // TODO better agnostics
                    break;
                }
            }
        });

        Ok(Response::new(rx))
    }

    async fn app_updates(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::AppUpdatesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);
        let mut recver = self.app_updates_tx.subscribe();

        futures::spawn(async move {
            loop {
                let app_repr = recver
                    .recv()
                    .await
                    .map(|app_id| AppId { app_id })
                    .map_err(|e| Status::internal(e.to_string()));

                if let Err(_) = tx.send(app_repr).await {
                    log::warn!("ending AppUpdates stream.."); // TODO better agnostics
                    break;
                }
            }
        });

        Ok(Response::new(rx))
    }
}
