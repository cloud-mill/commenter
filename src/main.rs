mod common;
mod handlers;
mod models;
mod persistent;
mod service;

use service::server::init_server;

#[tokio::main]
async fn main() {
    init_server().await;
}
