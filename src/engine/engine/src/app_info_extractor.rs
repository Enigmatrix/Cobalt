use std::cell::RefCell;
use std::rc::Rc;

use data::db::Database;
use data::models;
use platform::objects::AppInfo;
use utils::channels::Receiver;
use utils::errors::*;
use utils::futures::{block_on, LocalExecutor};
use utils::tracing::info;

type Callback = Box<dyn Fn(models::App) -> Result<()>>;

pub struct AppInfoExtractor<'a> {
    db: Rc<RefCell<Database<'a>>>,
    extract_rx: Receiver<(models::Ref<models::App>, models::AppIdentity)>,
    cb: Rc<RefCell<Callback>>,
    exec: LocalExecutor<'a>,
}

impl<'a> AppInfoExtractor<'a> {
    pub fn new(
        db: Database<'a>,
        extract_rx: Receiver<(models::Ref<models::App>, models::AppIdentity)>,
        cb: Box<dyn Fn(models::App) -> Result<()>>,
    ) -> Self {
        Self {
            db: Rc::new(RefCell::new(db)),
            extract_rx,
            cb: Rc::new(RefCell::new(cb)),
            exec: LocalExecutor::new(),
        }
    }

    async fn extract(
        app: models::Ref<models::App>,
        identity: models::AppIdentity,
        cb: Rc<RefCell<Callback>>,
        db: Rc<RefCell<Database<'a>>>,
    ) -> Result<()> {
        let app_info = match identity.clone() {
            models::AppIdentity::Win32 { path } => AppInfo::from_win32(&path)
                .await
                .context("get win32 app info")?,
            models::AppIdentity::UWP { aumid } => AppInfo::from_uwp(&aumid)
                .await
                .context("get uwp app info")?,
        };

        let app = models::App {
            id: app,
            name: app_info.name,
            description: app_info.description,
            company: app_info.company,
            color: None, // TODO color derivation
            identity,
        };

        let blob = {
            let mut db = db.borrow_mut();
            let sz = app_info.logo.Size()?;
            db.create_app_icon(&app.id, sz as usize)
                .context("create app icon")?
        };

        info!("created blob of size {}", blob.size());

        // TODO write logo
        // app_info.logo.ReadAsync(buffer, count, options)

        {
            let mut db = db.borrow_mut();
            db.initialize_app(&app).context("initialize app")?;
        }

        {
            let cb = cb.borrow();
            cb(app).context("callback with extracted app info")?;
        }

        Ok(())
    }

    pub fn run(self) {
        block_on(self.exec.run(async {
            loop {
                let (app, identity) = self
                    .extract_rx
                    .recv_async()
                    .await
                    .context("recv extract tx")
                    .unwrap();
                let cb = self.cb.clone();
                let db = self.db.clone();
                self.exec
                    .spawn(async move {
                        Self::extract(app, identity, cb, db)
                            .await
                            .context("extract app info")
                            .unwrap();
                    })
                    .detach();
            }
        }));
    }
}
