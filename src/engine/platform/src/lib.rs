pub mod errors;
pub mod events;
pub mod objects;

use utils::errors::*;

pub fn setup() -> Result<()> {
    objects::Timestamp::setup();
    Ok(())
}
