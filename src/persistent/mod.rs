use anyhow::Result;
use axum_macros::FromRef;
mod mongo;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(FromRef, Serialize, Deserialize, Clone, Debug)]
pub struct MongoDbConfig {
    #[from_ref(skip)]
    pub mongo_username: String,

    #[from_ref(skip)]
    pub mongo_password: String,

    #[from_ref(skip)]
    pub mongo_host: String,

    pub mongo_port: u16,

    #[from_ref(skip)]
    pub mongo_db_name: String,

    #[from_ref(skip)]
    pub mongo_auth_source: String,

    pub mongo_max_connections: u32,
}

#[derive(Debug)]
pub struct PersistentLayer {
    pub mongo_client: Client,

    pub mongo_config: MongoDbConfig,
}

pub async fn init_mongo_connection(
    connection_string: &str,
    max_pool_size: Option<u32>,
) -> Result<Client> {
    let mut options = ClientOptions::parse(connection_string).await?;

    if let Some(max_pool_size) = max_pool_size {
        options.max_pool_size = Some(max_pool_size);
    }

    let client = Client::with_options(options)?;

    let db_name = options
        .default_database
        .clone()
        .expect("database name must be specified in the connection string");

    let db = client.database(&db_name);
    db.run_command(doc! { "ping": 1 }, None).await?;

    Ok(client)
}
