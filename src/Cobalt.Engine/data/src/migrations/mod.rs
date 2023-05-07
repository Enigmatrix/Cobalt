use crate::migrator::Migration;

mod m1;

pub fn default_migrations() -> Vec<Box<dyn Migration>> {
    vec![Box::new(m1::Migration1)]
}
