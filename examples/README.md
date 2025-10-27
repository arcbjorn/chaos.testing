# Chaos Testing Demo

Complete end-to-end demonstration of the Chaos Testing framework.

## Setup

### 1. Install Dependencies

```bash
# Install Chaos Testing
cd ..
cargo install --path .

# Install demo API dependencies
cd examples
pip install -r requirements.txt
```

### 2. Start the Demo API

```bash
# Terminal 1: Start the demo backend on port 9000
python demo-api.py
```

You should see:
```
Starting Demo API on http://localhost:9000
INFO:     Uvicorn running on http://0.0.0.0:9000
```

### 3. Start the Interceptor

```bash
# Terminal 2: Start chaos-testing proxy on port 8080, forwarding to port 9000
chaos-testing observe --port 8080 --target http://localhost:9000 --output demo.db
```

You should see:
```
HTTP interceptor listening on 127.0.0.1:8080
Storing captures in: demo.db
Forwarding requests to: http://localhost:9000
```

### 4. Send Test Traffic

```bash
# Terminal 3: Send requests through the proxy
./test-traffic.sh
```

Or manually:
```bash
curl http://localhost:8080/api/users
curl http://localhost:8080/api/products/123
curl -X POST http://localhost:8080/api/orders -H "Content-Type: application/json" -d '{"item":"widget"}'
```

### 5. Generate Tests

```bash
# Generate Python tests
chaos-testing generate --input demo.db --language python --output generated-tests

# Generate Go tests
chaos-testing generate --input demo.db --language go --output generated-tests

# Generate Rust tests
chaos-testing generate --input demo.db --language rust --output generated-tests
```

### 6. Run Generated Tests

```bash
# Python
cd generated-tests
pip install pytest requests
pytest test_generated.py -v

# Go
cd generated-tests
go mod init test
go get github.com/stretchr/testify
go test -v

# Rust
cd generated-tests
cargo init --name test
cargo add reqwest tokio --features tokio/macros,tokio/rt-multi-thread,reqwest/json
cargo test
```

## What Gets Captured

The interceptor captures:
- HTTP method and URL
- Request headers
- Response status codes
- Response headers and body
- Request duration (ms)
- Timestamp

All stored in SQLite (`demo.db`) for analysis.

## Expected Output

After running test traffic, you should see:
```
Captured 6 requests
Generated tests in generated-tests/
  - Python: test_generated.py (6 tests)
  - Go: test_generated.go (6 tests)
  - Rust: test_generated.rs (6 tests)
```

## Testing the Generated Tests

The generated tests should pass when run against the demo API:

```bash
# Keep demo API running on port 9000
# The generated tests will hit http://localhost:8080 (change to :9000 in test files)
```

Edit the generated test files to point to `http://localhost:9000` instead of `http://localhost:8080`.

## Architecture

```
Client (curl/test-traffic.sh)
    ↓
Chaos Testing Proxy (localhost:8080)
    ↓ [captures & stores to demo.db]
    ↓
Demo API (localhost:9000)
    ↓
Response back through proxy
```

## Troubleshooting

**Port already in use:**
```bash
lsof -ti:8080 | xargs kill -9
lsof -ti:9000 | xargs kill -9
```

**jq not found:**
```bash
# Ubuntu/Debian
sudo apt install jq

# macOS
brew install jq
```

**Demo API won't start:**
```bash
pip install --upgrade fastapi uvicorn pydantic
```
