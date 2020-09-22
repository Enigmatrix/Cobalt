use crate::data::entities::*;

pub struct Database;

impl Database {
    pub fn new() -> Self {
        todo!()
    }

    pub fn insert_app(&mut self, app: &mut App) {
        todo!();
    }

    pub fn insert_session(&mut self, sess: &mut Session) {
        todo!()
    }

    pub fn insert_usage(&mut self, usage: &mut Usage) {
        todo!()
    }

    pub fn app_id_by_app_identification(
        &mut self,
        identification: &AppIdentification,
    ) -> Option<u64> {
        todo!()
    }
}
