use super::model;
use anyhow::*;
use std::collections::HashMap;

static mut APPID: u64 = 1;

// TODO PLACEHOLDER, IN-MEMORY REPRESENTATION

pub struct Database {
    apps: HashMap<model::AppIdentity, model::App>,
}

impl Database {
    pub fn new() -> Result<Database> {
        Ok(Database {
            apps: HashMap::new(),
        })
    }

    pub fn insert_app(&mut self, mut app: model::App) -> Result<model::App> {
        unsafe {
            APPID += 1;
            app.id = APPID;
        }
        let app1 = app.clone();
        let app2 = app.clone();
        self.apps.insert(app.identity, app2);
        Ok(app1)
    }

    pub fn insert_session(&mut self, session: model::Session) -> Result<model::Session> {
        Ok(session)
    }

    pub fn app_by_identity(&mut self, identity: &model::AppIdentity) -> Result<Option<model::App>> {
        Ok(self.apps.get(identity).cloned())
    }
}
