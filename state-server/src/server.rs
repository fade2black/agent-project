use crate::agents;
use crate::agents::AgentsResponse;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;
use udp_discovery::UdpDiscovery;

#[derive(Clone)]
pub(crate) struct AgentState {
    pub discovery: Arc<UdpDiscovery>,
}

#[derive(Debug, Error)]
pub enum StateServerError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
struct ErrorMessage(String);
type ErrorResponse = (StatusCode, Json<ErrorMessage>);

pub struct HttpServer {
    port: u16,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        HttpServer { port }
    }

    pub async fn run(self, discovery: Arc<UdpDiscovery>) -> Result<(), StateServerError> {
        let state = AgentState { discovery };

        let app = Router::new()
            .route("/up", get(|| async { "OK" }))
            .route("/agents", get(agents_action))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(addr.clone()).await?;

        info!("Starting HTTP server on {} ...", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn agents_action(
    State(state): State<AgentState>,
) -> Result<Json<AgentsResponse>, ErrorResponse> {
    if let Ok(agents) = agents::handler(&state).await {
        Ok(Json(agents))
    } else {
        Err(bad_request("Unable to fetch agents."))
    }
}

fn bad_request(msg: &str) -> ErrorResponse {
    (StatusCode::BAD_REQUEST, Json(ErrorMessage(msg.to_string())))
}
