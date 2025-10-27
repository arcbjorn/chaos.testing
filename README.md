# Chaos Testing

**Language-Agnostic Backend Testing Framework**

Chaos Testing intercepts at the network and system level to test ANY backend - Python, Go, Java, Ruby, Rust, PHP, C#, Elixir, or anything else. No code parsing required.

## How It Works

Instead of parsing source code (language-specific), we intercept **protocols** (universal):

1. **Network Interception** - Captures HTTP/HTTPS, database queries, message queues
2. **Behavior Learning** - Analyzes traffic patterns to understand your application
3. **Test Generation** - Creates idiomatic tests in your language
4. **Chaos Replay** - Replays traffic with injected failures

## Installation

```bash
cargo install --path .
```

## Quick Start

### 1. Capture Traffic

Start the interceptor as a proxy between your client and backend:

```bash
# Terminal 1: Start your backend on port 9000
# (any language - Python, Go, Node.js, etc.)

# Terminal 2: Start interceptor proxy
chaos-testing observe --port 8080 --target http://localhost:9000 --output my-app.db
```

Send requests through the proxy:

```bash
# Terminal 3: Send traffic through proxy (port 8080)
curl http://localhost:8080/api/users
curl http://localhost:8080/api/products/123
curl -X POST http://localhost:8080/api/orders -d '{"item":"widget"}'
```

The proxy forwards to your backend and captures everything!

### 2. Generate Tests

Generate tests in your preferred language:

```bash
# Python with pytest
chaos-testing generate --input my-app.db --language python --framework pytest

# Go
chaos-testing generate --input my-app.db --language go

# Rust
chaos-testing generate --input my-app.db --language rust
```

This creates test files in the `tests/` directory:
- Python: `tests/test_generated.py`
- Go: `tests/test_generated.go`
- Rust: `tests/test_generated.rs`

### 3. Run Generated Tests

```bash
# Python
cd tests && pytest test_generated.py

# Go
cd tests && go test

# Rust
cd tests && cargo test
```

## Supported Languages

Works with **ANY** language because it intercepts at the network level:
- Python (FastAPI, Django, Flask)
- Go (Gin, Echo, net/http)
- Rust (Axum, Actix, Rocket)
- Java (Spring Boot, Quarkus)
- Ruby (Rails, Sinatra)
- PHP (Laravel, Symfony)
- C# (.NET, ASP.NET)
- Elixir (Phoenix)
- Node.js (Express, Fastify)
- And literally anything else!

## Features

- ✅ Zero configuration - point and observe
- ✅ Language agnostic - works with compiled & interpreted languages
- ✅ Discovers actual behavior - not what code says, what it DOES
- ✅ Protocol-aware - HTTP, SQL, Redis, Kafka, gRPC
- ✅ Chaos engineering - inject failures at network level
- ✅ Works with closed source - no code access needed

## Architecture

```
┌─────────────────┐
│  Your Backend   │  (Any Language)
│   (Port 8080)   │
└────────┬────────┘
         │
         ├─── HTTP Requests
         ├─── Database Queries
         ├─── Queue Messages
         │
    ┌────▼─────────────┐
    │  Chaos Testing   │
    │   Interceptor    │
    └────┬─────────────┘
         │
         ├─► Protocol Parsers
         ├─► Behavior Learner
         ├─► Test Generator
         └─► Chaos Engine
```

## Project Structure

```
chaos-testing/
├── src/
│   ├── interceptor.rs    # HTTP proxy server
│   ├── parsers/          # Protocol parsers (HTTP, SQL)
│   ├── storage.rs        # SQLite persistence
│   ├── models.rs         # Data structures
│   ├── generators/       # Test code generators
│   │   ├── python.rs
│   │   ├── go.rs
│   │   └── rust_gen.rs
│   └── main.rs           # CLI entry point
```

## Features Implemented

- ✅ HTTP traffic interception
- ✅ SQLite storage for captured requests
- ✅ HTTP/SQL protocol parsing
- ✅ Test generation for Python/Go/Rust
- ✅ Clean conventional commits
- ⏳ Chaos injection (coming soon)
- ⏳ Behavior pattern analysis (coming soon)

## Demo

See the full working demo in [`examples/`](examples/):

```bash
cd examples
python demo-api.py  # Terminal 1
chaos-testing observe --port 8080 --target http://localhost:9000  # Terminal 2
./test-traffic.sh   # Terminal 3
chaos-testing generate --language python  # Generate tests
```

Full instructions: [examples/README.md](examples/README.md)

## Development

```bash
# Build
cargo build --release

# Run
cargo run -- observe --port 8080 --target http://localhost:9000

# Test
cargo test
```
