use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use rmcp::model::{ClientJsonRpcMessage, ClientRequest, JsonRpcMessage};
use serde_json::json;
use std::sync::Arc;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

const BIND_ADDRESS: &str = "127.0.0.1:10086";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Configure logging with source code information
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true) // Show filename
                .with_line_number(true) // Show line number
                .with_target(true), // Show target module
        )
        .init();

    // Create HTTP server that doesn't depend on SSE
    let counter_service = CounterService::new();
    let app = Router::new()
        .route("/api/counter", post(http_counter_handler))
        .route("/api/counter", get(http_counter_get))
        .with_state(counter_service);

    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    tracing::info!("Starting HTTP API server on {}", BIND_ADDRESS);
    tracing::info!(
        "POST example: curl -X POST http://127.0.0.1:10086/api/counter -H \"Content-Type: application/json\" -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"counter\",\"arguments\":{{\"operation\":\"increment\"}}}}}}'"
    );
    tracing::info!("GET example: curl http://127.0.0.1:10086/api/counter");

    let http_server = axum::serve(tcp_listener, app);

    if let Err(e) = http_server.await {
        tracing::error!(error = %e, "HTTP server shutdown with error");
    }

    Ok(())
}

// A standalone service that doesn't depend on SSE
#[derive(Clone)]
struct CounterService {
    counter: Arc<tokio::sync::Mutex<i32>>,
}

impl CounterService {
    fn new() -> Self {
        Self {
            counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    async fn increment(&self) -> i32 {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        *counter
    }

    async fn decrement(&self) -> i32 {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        *counter
    }

    async fn get_value(&self) -> i32 {
        let counter = self.counter.lock().await;
        *counter
    }
}

// Simple GET endpoint to retrieve counter value
async fn http_counter_get(State(service): State<CounterService>) -> Json<serde_json::Value> {
    let value = service.get_value().await;
    Json(json!({ "value": value }))
}

// HTTP handler that doesn't require SSE session
async fn http_counter_handler(
    State(service): State<CounterService>,
    Json(message): Json<ClientJsonRpcMessage>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Create a span for tracking context with source file and line information
    let span = tracing::info_span!("json_rpc_handler", file = file!(), line = line!());
    let _guard = span.enter();

    // Process the message using rmcp types
    match &message {
        JsonRpcMessage::Request(req) => {
            match &req.request {
                ClientRequest::CallToolRequest(tool_req) => {
                    match tool_req.params.name.as_ref() {
                        "counter" => {
                            // Get the operation from the arguments
                            let op_name = tool_req
                                .params
                                .arguments
                                .as_ref()
                                .unwrap()
                                .get("operation")
                                .unwrap();

                            let op_name_str = op_name.as_str().unwrap();
                            tracing::debug!(
                                file = file!(),
                                line = line!(),
                                "op_name: {:?}",
                                &op_name_str
                            );

                            let result = match op_name_str {
                                "increment" => {
                                    let new_value = service.increment().await;
                                    json!({ "value": new_value })
                                }
                                "decrement" => {
                                    let new_value = service.decrement().await;
                                    json!({ "value": new_value })
                                }
                                "get_value" => {
                                    let value = service.get_value().await;
                                    json!({ "value": value })
                                }
                                _ => {
                                    tracing::error!(
                                        file = file!(),
                                        line = line!(),
                                        "Invalid operation name: {}",
                                        op_name_str
                                    );
                                    return Err((
                                        StatusCode::BAD_REQUEST,
                                        format!("Invalid operation name: {}", op_name_str),
                                    ));
                                }
                            };

                            // Construct a proper JsonRpcResponse
                            let response = json!({
                                "jsonrpc": "2.0",
                                "id": req.id,
                                "result": result
                            });

                            return Ok(Json(response));
                        }
                        _ => {
                            tracing::error!(
                                file = file!(),
                                line = line!(),
                                "Invalid tool name: {}",
                                tool_req.params.name.as_ref()
                            );
                            return Err((
                                StatusCode::BAD_REQUEST,
                                format!("Invalid tool name: {}", tool_req.params.name.as_ref()),
                            ));
                        }
                    }
                }
                _ => {
                    tracing::error!(
                        file = file!(),
                        line = line!(),
                        "Expected CallToolRequest, got: {:?}",
                        req.request
                    );
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Invalid request type. Expected call_tool request.".to_string(),
                    ));
                }
            }
        }
        _ => {
            tracing::error!(
                file = file!(),
                line = line!(),
                "Expected Request, got: {:?}",
                message
            );
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid JSON-RPC message type. Expected a request.".to_string(),
            ));
        }
    }
}
