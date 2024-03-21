use anyhow::Result;
mod mongo;
use mongodb::{options::ClientOptions, Client};

#[derive(Debug)]
pub struct PersistentLayer {
    pub mongo_client: Client,

    pub mongo_db_name: String,
}

pub async fn init_mongo_connection(connection_string: &str) -> Result<Client> {
    let options = ClientOptions::parse(&connection_string).await?;

    let client = Client::with_options(options)?;

    Ok(client)
}
