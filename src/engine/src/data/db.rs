use crate::data::entities::*;
use crate::errors::*;

#[derive(Debug)]
pub struct Database;

impl Database {
    pub fn new() -> Self {
        Database
    }

    pub fn insert_app(&mut self, _app: &mut App) -> Result<()> {
        Ok(())
    }

    pub fn insert_session(&mut self, _sess: &mut Session) -> Result<()> {
        Ok(())
    }

    pub fn insert_usage(&mut self, _usage: &mut Usage) -> Result<()> {
        Ok(())
    }

    pub fn app_id_by_app_identification(
        // TODO use this instead of just re-getting the app everytime!
        &mut self,
        _identification: &AppIdentification,
    ) -> Option<u64> {
        todo!()
    }
}
