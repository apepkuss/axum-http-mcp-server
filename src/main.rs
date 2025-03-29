use axum::{
    Json, Router,
    body::Body,
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
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
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
        "POST example: curl -X POST http://127.0.0.1:10086/api/counter -H \"Content-Type: application/json\" -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"call_tool\",\"params\":{{\"name\":\"increment\"}}}}'"
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
    // The request must follow JSON-RPC structure with format:
    // {
    //   "jsonrpc": "2.0",
    //   "id": 1,
    //   "method": "call_tool",
    //   "params": { "name": "increment" }
    // }
    body: Body,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // First try to extract the raw body as JSON
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {}", e),
            ));
        }
    };

    // Parse the JSON directly instead of trying to use complex types
    let json_value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(val) => val,
        Err(e) => {
            // Log the error and the received JSON for debugging
            let json_str = String::from_utf8_lossy(&bytes);
            tracing::error!("Failed to parse JSON: {}\nReceived JSON: {}", e, json_str);
            return Err((StatusCode::BAD_REQUEST, "Invalid JSON format".to_string()));
        }
    };

    // Validate it's a JSON-RPC request
    if json_value["jsonrpc"] != "2.0" || json_value["method"] != "call_tool" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid JSON-RPC request. Expected jsonrpc=2.0 and method=call_tool".to_string(),
        ));
    }

    // Extract the tool name
    let tool_name = match json_value["params"]["name"].as_str() {
        Some(name) => name,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing or invalid tool name in params".to_string(),
            ));
        }
    };

    // Process based on tool name
    let result = match tool_name {
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
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid tool name: {}", tool_name),
            ));
        }
    };

    // Return JSON-RPC response with result
    let response = json!({
        "jsonrpc": "2.0",
        "id": json_value["id"],
        "result": result
    });

    Ok(Json(response))
}
