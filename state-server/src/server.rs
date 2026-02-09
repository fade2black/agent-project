use crate::agents::{self, AgentsResponse};
use crate::tasks::{self, TasksResponse};
use agent_state::{AgentStore, Config, TaskStore};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
pub struct StateServerContext {
    pub agent_store: Arc<RwLock<AgentStore>>,
    pub task_store: Arc<RwLock<TaskStore>>,
}

#[derive(Debug, Error)]
pub enum StateServerError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
struct ErrorMessage(String);
type ErrorResponse = (StatusCode, Json<ErrorMessage>);

pub async fn start(state: StateServerContext) -> Result<(), StateServerError> {
    let config = Config::new();

    let app = Router::new()
        .route("/up", get(|| async { "OK" }))
        .route("/agents", get(agents_action))
        .route("/tasks", get(tasks_action))
        .route("/config", get(config_action))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;

    info!("Starting HTTP server on {} ...", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn agents_action(
    State(state): State<StateServerContext>,
) -> Result<Json<AgentsResponse>, ErrorResponse> {
    if let Ok(agents) = agents::handler(&state).await {
        Ok(Json(agents))
    } else {
        Err(bad_request("Unable to fetch agents."))
    }
}

async fn tasks_action(
    State(state): State<StateServerContext>,
) -> Result<Json<TasksResponse>, ErrorResponse> {
    if let Ok(tasks) = tasks::handler(&state).await {
        Ok(Json(tasks))
    } else {
        Err(bad_request("Unable to fetch tasks."))
    }
}

async fn config_action() -> Result<Json<Config>, ErrorResponse> {
    Ok(Json(Config::new()))
}

fn bad_request(msg: &str) -> ErrorResponse {
    (StatusCode::BAD_REQUEST, Json(ErrorMessage(msg.to_string())))
}
