use crate::agents::{self, AgentsResponse};
use crate::tasks::{self, TasksResponse};
use agent_state::Config;
use agent_state::SharedAgentState;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum StateServerError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
struct ErrorMessage(String);
type ErrorResponse = (StatusCode, Json<ErrorMessage>);

pub struct StateServer {
    config: Config,
    agent_state: Arc<SharedAgentState>,
}

impl StateServer {
    pub fn new(config: Config, agent_state: Arc<SharedAgentState>) -> Self {
        Self {
            agent_state,
            config,
        }
    }

    pub async fn start(&self) -> Result<(), StateServerError> {
        let app = Router::new()
            .route("/up", get(|| async { "OK" }))
            .route("/agents", get(agents_action))
            .route("/tasks", get(tasks_action))
            .route("/config", get(config_action))
            .with_state(self.agent_state.clone());

        let addr = format!("0.0.0.0:{}", self.config.http_port);
        let listener = tokio::net::TcpListener::bind(addr.clone()).await?;

        info!("Starting HTTP server on {} ...", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn agents_action(
    State(state): State<Arc<SharedAgentState>>,
) -> Result<Json<AgentsResponse>, ErrorResponse> {
    if let Ok(agents) = agents::handler(state.clone()).await {
        Ok(Json(agents))
    } else {
        Err(bad_request("Unable to fetch agents."))
    }
}

async fn tasks_action(
    State(state): State<Arc<SharedAgentState>>,
) -> Result<Json<TasksResponse>, ErrorResponse> {
    if let Ok(tasks) = tasks::handler(state.clone()).await {
        Ok(Json(tasks))
    } else {
        Err(bad_request("Unable to fetch tasks."))
    }
}

async fn config_action() -> Result<Json<Config>, ErrorResponse> {
    Ok(Json(Config::from_env()))
}

fn bad_request(msg: &str) -> ErrorResponse {
    (StatusCode::BAD_REQUEST, Json(ErrorMessage(msg.to_string())))
}
