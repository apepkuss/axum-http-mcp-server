# Axum-HTTP-MCP-Server

## Build

```bash
git clone https://github.com/apepkuss/axum-http-mcp-server.git

cd axum-http-mcp-server

cargo build --release
```

## Run

- Install WasmEdge Runtime

```bash
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1
```

- Start the server

  ```bash
  wasmedge --dir .:. ./target/wasm32-wasip1/release/axum-mcp-server.wasm
  ```

- Test the server

  - Call "counter" tool to increment the counter

    ```bash
    curl -X POST http://localhost:10086/api/counter \
      --header "Content-Type: application/json" \
      --data '{
          "jsonrpc": "2.0",
          "id": 5,
          "method": "tools/call",
          "params": {
              "name": "counter",
              "arguments": {
                  "operation": "increment"
              }
          }
      }'
    ```

    Response:

    ```json
    {
        "id": 5,
        "jsonrpc": "2.0",
        "result": {
            "value": 1
        }
    }
    ```

  - Call "counter" tool to decrement the counter

  ```bash
  curl -X POST http://127.0.0.1:10086/api/counter \
    --header "Content-Type: application/json" \
    --data '{
      "jsonrpc": "2.0",
      "id": 5,
      "method": "tools/call",
      "params": {
          "name": "counter",
          "arguments": {
              "operation": "decrement"
          }
      }
  }'
  ```

  Response:

  ```json
  {
    "id": 5,
    "jsonrpc": "2.0",
    "result": {
        "value": 0
    }
  }
  ```

  - Call "counter" tool to get the counter value

  ```bash
  curl -X POST http://127.0.0.1:10086/api/counter \
    --header "Content-Type: application/json" \
    --data '{
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "counter",
            "arguments": {
                "operation": "get_value",
            }
        }
    }'
  ```

  Response:

  ```json
  {
      "id": 5,
      "jsonrpc": "2.0",
      "result": {
          "value": 0
      }
  }

