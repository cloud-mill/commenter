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

pub async fn init_mongo_connection(config: &MongoDbConfig) -> Result<Client> {
    let connection_string = format!(
        "mongodb://{}:{}@{}:{}/{}?authSource={}",
        &config.mongo_username,
        &config.mongo_password,
        &config.mongo_host,
        &config.mongo_port,
        &config.mongo_db_name,
        &config.mongo_auth_source,
    );

    let mut options = ClientOptions::parse(&connection_string).await?;

    options.max_pool_size = Some(config.mongo_max_connections);

    let client = Client::with_options(options)?;

    // test client connection
    let db = client.database(&config.mongo_db_name);
    db.run_command(doc! { "ping": 1 }, None).await?;

    Ok(client)
}
