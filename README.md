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

  ```bash
  # 增加计数器
  curl -X POST http://127.0.0.1:10086/api/counter \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"increment"}}'

  # 减少计数器
  curl -X POST http://127.0.0.1:10086/api/counter \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"decrement"}}'

  # 获取计数器值
  curl -X POST http://127.0.0.1:10086/api/counter \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"get_value"}}'
  ```
