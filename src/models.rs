use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub protocol: Protocol,
    pub request: RequestData,
    pub response: Option<ResponseData>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Http,
    Https,
    Sql,
    Redis,
    Kafka,
    Grpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<String>,
    pub database: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCommand {
    pub command: String,
    pub args: Vec<String>,
    pub database: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub endpoint: String,
    pub method: String,
    pub request_count: u64,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dep_type: DependencyType,
    pub target: String,
    pub call_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Database,
    HttpService,
    Cache,
    Queue,
}
