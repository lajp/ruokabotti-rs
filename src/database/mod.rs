use std::sync::Arc;

use serenity::prelude::TypeMapKey;
use sqlx::MySqlPool;
pub mod arvio;
pub mod lisaa_ruoka;
pub mod nouda_ruoka;
pub mod ruokakuva;

pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn new() -> Database {
        let pool = MySqlPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        Self { pool }
    }
}

impl TypeMapKey for Database {
    type Value = Arc<Database>;
}
