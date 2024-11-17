#[allow(unused_imports)]
use anyhow::{anyhow, bail, Error, Result};
#[allow(unused_imports)]
use tracing::{debug, error, info, span, trace, warn, Level};

use axum::{
    routing::{get, post},
    Extension, Router,
};
use common_macros::*;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Weak;
use tower_http::{services::ServeDir, trace::TraceLayer};

use maud::{html, Markup, DOCTYPE};

mod endpoint;
mod page;

mod subscriber;
use subscriber::*;

mod message;
use message::*;

mod topic;
use topic::*;

declare_settings! {
    keep_alive_secs: u64,
}

pub async fn server(
    port: u16,
    op_mode: op_mode::OpMode,
    pkg_name: &'static str,
    pkg_version: &'static str,
) -> Result<()> {
    let url_prefix = match op_mode {
        op_mode::OpMode::Prod => format!("/{pkg_name}"),
        op_mode::OpMode::Local => "".to_owned(),
        op_mode => format!("/{op_mode}/{pkg_name}"),
    };

    let shared_state = Arc::new(tokio::sync::RwLock::new(AppState {
        pkg_name,
        pkg_version,
        url_prefix,
        op_mode,
        topics: HashMap::new(),
    }));
    let app = endpoint::router()
        .layer(Extension(shared_state.clone()))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("will start web server at PORT={port}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .map_err(|err| anyhow!(err))
}

pub type SharedState = Arc<tokio::sync::RwLock<AppState>>;

pub struct AppState {
    pkg_name: &'static str,
    pkg_version: &'static str,
    url_prefix: String,
    op_mode: op_mode::OpMode,
    topics: HashMap<String, Arc<std::sync::RwLock<Topic>>>,
}
