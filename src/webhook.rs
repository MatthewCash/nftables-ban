use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use rusqlite::Connection;
use sd_notify::NotifyState;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex};

use crate::{
    config::Config,
    database::{self},
    nftables::{add_addrs_to_set, remove_addrs_from_set},
};

#[derive(Clone, Debug, Deserialize)]
pub struct WebhookConfig {
    addr: SocketAddr,
    auth_token: String,
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    addr: Ipv4Addr,
}

#[derive(Clone, Debug)]
struct WebhookState {
    db_conn: Arc<Mutex<Connection>>,
    config: Config,
}

#[axum::debug_handler]
async fn handle_add(
    headers: HeaderMap,
    State(state): State<WebhookState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<(), StatusCode> {
    if headers
        .get("authorization")
        .is_none_or(|a| *a != *state.config.webhook.auth_token)
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    async {
        database::add_addr(
            &*state.db_conn.lock().await,
            payload.addr,
            &state.config.database,
        )?;

        add_addrs_to_set(&state.config.nftables, &[payload.addr])?;

        log::debug!("added addr {} to blacklist", payload.addr);
        Result::<()>::Ok(())
    }
    .await
    .map_err(|err| {
        log::error!("{:?}", err.context("failed to handle add request"));
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[axum::debug_handler]
async fn handle_remove(
    headers: HeaderMap,
    State(state): State<WebhookState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<(), StatusCode> {
    if headers
        .get("authorization")
        .is_none_or(|a| *a != *state.config.webhook.auth_token)
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    async {
        database::remove_addr(
            &*state.db_conn.lock().await,
            payload.addr,
            &state.config.database,
        )?;

        remove_addrs_from_set(&state.config.nftables, &[payload.addr])?;

        log::debug!("removed addr {} from blacklist", payload.addr);
        Result::<()>::Ok(())
    }
    .await
    .map_err(|err| {
        log::error!("{:?}", err.context("failed to handle remove request"));
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Serialize)]
struct CheckResponse {
    present: bool,
}

#[axum::debug_handler]
async fn handle_check(
    headers: HeaderMap,
    State(state): State<WebhookState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<Json<CheckResponse>, StatusCode> {
    if headers
        .get("authorization")
        .is_none_or(|a| *a != *state.config.webhook.auth_token)
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let present = database::check_addr(
        &*state.db_conn.lock().await,
        payload.addr,
        &state.config.database,
    )
    .map_err(|err| {
        log::error!("{:?}", err.context("failed to handle check request"));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CheckResponse { present }))
}

pub async fn start_listening(db_conn: Connection, config: Config) -> Result<()> {
    let listener = TcpListener::bind(config.webhook.addr).await?;

    let router = Router::new()
        .route("/add", post(handle_add))
        .route("/remove", post(handle_remove))
        .route("/check", post(handle_check))
        .with_state(WebhookState {
            db_conn: Arc::new(Mutex::new(db_conn)),
            config,
        });

    let _ = sd_notify::notify(false, &[NotifyState::Ready]);
    log::info!("Listening for webhooks...");

    axum::serve(listener, router).await?;

    Ok(())
}
