mod raw;

use raw::engine_server::{Engine, EngineServer};
pub use raw::{AppRepr, Empty, UsageSwitch};
use tonic::*;
use util::futures::sync::{broadcast, mpsc};
use util::*;

#[derive(Debug)]
pub struct EngineMessenger {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
}

#[derive(Debug)]
pub struct EngineWorker {
    usage_switches_tx: broadcast::Sender<UsageSwitch>,
    usage_switches_rx: broadcast::Receiver<UsageSwitch>,
}

impl EngineMessenger {
    pub fn push_usage_switch(&self, us: UsageSwitch) -> Result<()> {
        self.usage_switches_tx
            .send(us)
            //.with_context(|| "Send Usage Switch")?;
            .unwrap(); // TODO better result!
        Ok(())
    }
}

impl EngineWorker {
    pub fn new() -> (EngineMessenger, EngineWorker) {
        let (usage_switches_tx, usage_switches_rx) = broadcast::channel(1);
        let usage_switches_tx2 = usage_switches_tx.clone();
        (
            EngineMessenger { usage_switches_tx },
            EngineWorker {
                usage_switches_tx: usage_switches_tx2,
                usage_switches_rx,
            },
        )
    }

    pub async fn serve(self) -> Result<()> {
        let addr = "[::1]:50051".parse()?;

        transport::Server::builder()
            .add_service(EngineServer::new(self))
            .serve(addr)
            .await
            .with_context(|| "Serving EngineWorker")?;

        Ok(())
    }
}

#[tonic::async_trait]
impl Engine for EngineWorker {
    type OngoingUsageChangesStream = mpsc::Receiver<Result<UsageSwitch, Status>>;
    type AppUpdatesStream = mpsc::Receiver<Result<AppRepr, Status>>;

    async fn ongoing_usage_changes(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::OngoingUsageChangesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);
        let mut recver = self.usage_switches_tx.subscribe();

        futures::spawn(async move {
            loop {
                let us = recver
                    .recv()
                    .await
                    .map_err(|e| Status::internal(e.to_string()));

                if let Err(_) = tx.send(us).await {
                    log::warn!("ending task.."); // TODO better agnostics
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
        todo!("app_updates")
    }
}
