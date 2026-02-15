use crate::agents::{self, AgentsResponse};
use crate::bundle::{self, BundleResponse};
use crate::control_state::{self, ControlStateResponse};
use crate::tasks::{self, TasksResponse};
use crate::winners::{self, WinnersResponse};
use agent_state::{Config, SharedAgentState, Telemetry};
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
            .route("/bundle", get(bundle_action))
            .route("/winners", get(winners_action))
            .route("/tasks", get(tasks_action))
            .route("/config", get(config_action))
            .route("/state", get(state_action))
            .route("/telemetry", get(telemetry_action))
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

async fn bundle_action(
    State(state): State<Arc<SharedAgentState>>,
) -> Result<Json<BundleResponse>, ErrorResponse> {
    if let Ok(bundle) = bundle::handler(state.clone()).await {
        Ok(Json(bundle))
    } else {
        Err(bad_request("Unable to fetch bundle."))
    }
}

async fn winners_action(
    State(state): State<Arc<SharedAgentState>>,
) -> Result<Json<WinnersResponse>, ErrorResponse> {
    if let Ok(winners) = winners::handler(state.clone()).await {
        Ok(Json(winners))
    } else {
        Err(bad_request("Unable to fetch winners."))
    }
}

async fn config_action() -> Result<Json<Config>, ErrorResponse> {
    Ok(Json(Config::from_env()))
}

async fn state_action(
    State(state): State<Arc<SharedAgentState>>,
) -> Result<Json<ControlStateResponse>, ErrorResponse> {
    if let Ok(control_state) = control_state::handler(state.clone()).await {
        Ok(Json(control_state))
    } else {
        Err(bad_request("Unable to fetch control state."))
    }
}

async fn telemetry_action() -> Result<Json<Telemetry>, ErrorResponse> {
    Ok(Json(Telemetry::new()))
}

fn bad_request(msg: &str) -> ErrorResponse {
    (StatusCode::BAD_REQUEST, Json(ErrorMessage(msg.to_string())))
}
