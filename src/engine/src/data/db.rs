use crate::data::entities::*;
use crate::errors::*;

#[derive(Debug)]
pub struct Database;

impl Database {
    pub fn new() -> Self {
        Database
    }

    pub fn insert_app(&mut self, app: &mut App) -> Result<()> {
        Ok(())
    }

    pub fn insert_session(&mut self, sess: &mut Session) -> Result<()> {
        Ok(())
    }

    pub fn insert_usage(&mut self, usage: &mut Usage) -> Result<()> {
        Ok(())
    }

    pub fn app_id_by_app_identification(
        &mut self,
        identification: &AppIdentification,
    ) -> Option<u64> {
        todo!()
    }
}
