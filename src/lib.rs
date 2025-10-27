//! # Chaos Testing
//!
//! Language-agnostic backend testing framework using network-level interception.
//!
//! ## Overview
//!
//! Chaos Testing intercepts network traffic at the protocol level rather than parsing
//! source code, making it work with any programming language.
//!
//! ## Features
//!
//! - **Protocol Parsers**: HTTP, SQL, Redis, PostgreSQL, and more
//! - **Traffic Capture**: Intercept and store requests/responses
//! - **Test Generation**: Generate tests in Python, Go, Rust
//! - **Chaos Engineering**: Inject failures (delays, timeouts, errors)
//! - **Traffic Analysis**: Analyze patterns and dependencies
//!
//! ## Example
//!
//! ```rust,no_run
//! use chaos_testing::parsers::sql::SqlParser;
//!
//! let query = "SELECT * FROM users WHERE id = 1";
//! if let Some(parsed) = SqlParser::parse(query) {
//!     println!("Parsed query: {:?}", parsed);
//! }
//! ```

/// Data models for captured requests, responses, and analysis
pub mod models;

/// Protocol parsers for HTTP, SQL, Redis, PostgreSQL, Kafka, gRPC
pub mod parsers {
    /// gRPC request parser
    pub mod grpc;
    /// HTTP request/response parser
    pub mod http;
    /// Kafka message parser
    pub mod kafka;
    /// PostgreSQL wire protocol parser
    pub mod postgres;
    /// Redis RESP protocol parser
    pub mod redis;
    /// SQL query parser and classifier
    pub mod sql;
}
