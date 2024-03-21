use axum::{
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, str::FromStr, sync::Arc};
use tracing::info;

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

    pub mongo_db_name: String,
}

const CONFIG_PATH: &str = "/etc/commenter/config.json";

pub async fn init_server() {
    tracing_subscriber::fmt::init();

    // Configure server
    let config_str = fs::read_to_string(CONFIG_PATH)
        .expect("error reading config as str at /etc/commenter/config.json");
    let config: Config = serde_json::from_str(&config_str).expect("error deserializing config");

    let mongo_client = init_mongo_connection(&config.mongodb_connection_string)
        .await
        .unwrap();
    let persistent_layer = PersistentLayer {
        mongo_client,
        mongo_db_name: config.mongo_db_name,
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
