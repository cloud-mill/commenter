use axum::{
    routing::{get, post},
    Extension, Router,
};
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tracing::info;

use crate::persistent::MongoDbConfig;
use crate::{
    common::handlers::health_check,
    handlers::{
        create_branch_comment, create_root_comment, delete_comment, get_all_comments,
        get_branch_comments_next, get_branch_comments_rest, get_root_comments, react_to_comment,
        undo_react_to_comment, update_comment_text,
    },
    persistent::{init_mongo_connection, PersistentLayer},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub mongodb_connection_string: String,
    pub mongodb_max_pool_size: Option<u32>,
}

pub async fn init_server() {
    tracing_subscriber::fmt::init();

    // Configure server
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "localhost".to_string()),
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "7000".to_string())
            .parse()
            .expect("SERVER_PORT must be a number"),
        mongodb_connection_string: env::var("MONGODB_CONNECTION_STRING")
            .expect("MONGODB_CONNECTION_STRING must be set"),
        mongodb_max_pool_size: env::var("MONGODB_MAX_POOL_SIZE")
            .ok()
            .map(|s| s.parse().expect("MONGODB_MAX_POOL_SIZE must be a number")),
    };

    let mongo_client = init_mongo_connection(
        &config.mongodb_connection_string,
        config.mongodb_max_pool_size,
    )
    .await
    .unwrap();

    // extract the database name from the connection string
    let options = ClientOptions::parse(&config.mongodb_connection_string)
        .await
        .unwrap();
    let mongo_db_name = options.default_database.clone().unwrap_or_default();

    let mongo_config = MongoDbConfig { mongo_db_name };

    let persistent_layer = PersistentLayer {
        mongo_client,
        mongo_config,
    };

    let api_routes = Router::new()
        .route("/root-comment/new", post(create_root_comment))
        .route("/root-comments", get(get_root_comments))
        .route("/branch-comment/new", post(create_branch_comment))
        .route("/branch-comments/next", get(get_branch_comments_next))
        .route("/branch-comments/rest", get(get_branch_comments_rest))
        .route("/comments/all", get(get_all_comments))
        .route("/comment/update", post(update_comment_text))
        .route("/comment/delete", post(delete_comment))
        .route("/reaction/new", post(react_to_comment))
        .route("/reaction/undo", post(undo_react_to_comment))
        .layer(Extension(Arc::new(persistent_layer)));

    let health_probe_routes: Router = Router::new().route("/healthz", get(health_check));
    let app = Router::new().merge(api_routes).merge(health_probe_routes);

    let addr_str = format!("{}:{}", &config.server_host, &config.server_port);
    let addr = SocketAddr::from_str(&addr_str).expect("invalid server address in config");
    info!("Config server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
