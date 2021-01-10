mod raw;

pub mod dto {
    pub use super::raw::{UpdatedEntity, Empty, UsageSwitch};
}

use config::Config;
use dto::*;
use raw::relay_server::{Relay, RelayServer};
use tonic::*;
use util::futures::sync::{broadcast, mpsc};
use util::*;
use crate::data::model;
use prost::Enumeration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum EntityType {
    App = 0,
    Tag = 1,
    Alert = 2
}

#[derive(Debug)]
pub struct RelayServiceTx {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
    entity_updates_tx: broadcast::Sender<UpdatedEntity>,
}

#[derive(Debug)]
pub struct RelayService {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
    usage_switches_rx: broadcast::Receiver<UsageSwitch>,
    entity_updates_tx: broadcast::Sender<UpdatedEntity>,
    entity_updates_rx: broadcast::Receiver<UpdatedEntity>,
}

impl RelayServiceTx {
    pub fn push_usage_switch(&self, us: UsageSwitch) -> Result<()> {
        self.usage_switches_tx
            .send(us)
            //.with_context(|| "Send Usage Switch")?;
            .unwrap(); // TODO better result!
        Ok(())
    }

    pub fn push_app_update(&self, id: model::Id) -> Result<()> {
        self.entity_updates_tx
            .send(UpdatedEntity { etype: EntityType::App as i32, id })
            //.with_context(|| "Send App Update")?;
            .unwrap(); // TODO better result!
        Ok(())
    }
}

impl RelayService {
    pub fn new() -> (RelayServiceTx, RelayService) {
        let (usage_switches_tx, usage_switches_rx) = broadcast::channel(1);
        let usage_switches_tx2 = usage_switches_tx.clone();

        let (entity_updates_tx, entity_updates_rx) = broadcast::channel(1);
        let entity_updates_tx2 = entity_updates_tx.clone();

        (
            RelayServiceTx { usage_switches_tx, entity_updates_tx },
            RelayService {
                usage_switches_tx: usage_switches_tx2,
                usage_switches_rx,
                entity_updates_tx: entity_updates_tx2,
                entity_updates_rx,
            },
        )
    }

    pub async fn serve(self) -> Result<()> {
        let addr = Config::instance().service_addr.parse()?;

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
    type EntityUpdatesStream = mpsc::Receiver<Result<UpdatedEntity, Status>>;

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

    async fn entity_updates(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::EntityUpdatesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);
        let mut recver = self.entity_updates_tx.subscribe();

        futures::spawn(async move {
            loop {
                let app_repr = recver
                    .recv()
                    .await
                    .map_err(|e| Status::internal(e.to_string()));

                if let Err(_) = tx.send(app_repr).await {
                    log::warn!("ending EntityUpdates stream.."); // TODO better agnostics
                    break;
                }
            }
        });

        Ok(Response::new(rx))
    }

    async fn inform_entity_update(&self, req: Request<UpdatedEntity>) -> Result<Response<Empty>, Status> {
        self.entity_updates_tx.send(req.get_ref().clone())
            .map(|_| Response::new(Empty {}))
            .map_err(|_| Status::internal("Failed to send EntityUpdate to tx"))
    }
}
