# Chaos Testing

**Language-Agnostic Backend Testing Framework**

Chaos Testing intercepts at the network and system level to test ANY backend - Python, Go, Java, Ruby, Rust, PHP, C#, Elixir, or anything else. No code parsing required.

## How It Works

Instead of parsing source code (language-specific), we intercept **protocols** (universal):

1. **Network Interception** - Captures HTTP/HTTPS, database queries, message queues
2. **Behavior Learning** - Analyzes traffic patterns to understand your application
3. **Test Generation** - Creates idiomatic tests in your language
4. **Chaos Replay** - Replays traffic with injected failures

## Quick Start

```bash
# Observe any running process
chaos-testing observe --pid=1234 --duration=60s

# Or observe by port
chaos-testing observe --port=8080

# Generate tests for your language
chaos-testing generate --language=python --framework=pytest
chaos-testing generate --language=go
chaos-testing generate --language=rust

# Run chaos tests
chaos-testing chaos --level=extreme
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

## Status

🚧 **In Active Development** 🚧

Current focus: HTTP interception → Protocol parsing → Test generation for Python/Go/Rust
