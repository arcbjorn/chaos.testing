use crate::models::CapturedRequest;
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                protocol TEXT NOT NULL,
                method TEXT NOT NULL,
                uri TEXT NOT NULL,
                headers TEXT NOT NULL,
                body BLOB,
                response_status INTEGER,
                response_headers TEXT,
                response_body BLOB,
                duration_ms INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON requests(timestamp)",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_uri ON requests(uri)", [])?;

        Ok(())
    }

    pub fn store_request(&self, request: &CapturedRequest) -> Result<()> {
        let headers_json = serde_json::to_string(&request.request.headers)?;
        let response_headers = request
            .response
            .as_ref()
            .map(|r| serde_json::to_string(&r.headers))
            .transpose()?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO requests (
                id, timestamp, protocol, method, uri, headers, body,
                response_status, response_headers, response_body, duration_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                request.id,
                request.timestamp.to_rfc3339(),
                format!("{:?}", request.protocol),
                request.request.method,
                request.request.uri,
                headers_json,
                request.request.body.as_deref(),
                request.response.as_ref().map(|r| r.status_code),
                response_headers,
                request.response.as_ref().and_then(|r| r.body.as_deref()),
                request.duration_ms,
            ],
        )?;

        Ok(())
    }

    pub fn get_all_requests(&self) -> Result<Vec<CapturedRequest>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, protocol, method, uri, headers, body,
                    response_status, response_headers, response_body, duration_ms
             FROM requests
             ORDER BY timestamp",
        )?;

        let requests = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<Vec<u8>>>(6)?,
                row.get::<_, Option<u16>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<Vec<u8>>>(9)?,
                row.get::<_, Option<u64>>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for (
            id,
            timestamp,
            protocol,
            method,
            uri,
            headers_json,
            body,
            response_status,
            response_headers_json,
            response_body,
            duration_ms,
        ) in requests.flatten()
        {
            let request = self.deserialize_request(
                id,
                timestamp,
                protocol,
                method,
                uri,
                headers_json,
                body,
                response_status,
                response_headers_json,
                response_body,
                duration_ms,
            )?;
            result.push(request);
        }

        Ok(result)
    }

    fn deserialize_request(
        &self,
        id: String,
        timestamp: String,
        protocol: String,
        method: String,
        uri: String,
        headers_json: String,
        body: Option<Vec<u8>>,
        response_status: Option<u16>,
        response_headers_json: Option<String>,
        response_body: Option<Vec<u8>>,
        duration_ms: Option<u64>,
    ) -> Result<CapturedRequest> {
        use crate::models::{Protocol, RequestData, ResponseData};

        let headers = serde_json::from_str(&headers_json)?;
        let protocol = match protocol.as_str() {
            "Http" => Protocol::Http,
            "Https" => Protocol::Https,
            "Sql" => Protocol::Sql,
            "Redis" => Protocol::Redis,
            "Kafka" => Protocol::Kafka,
            "Grpc" => Protocol::Grpc,
            _ => Protocol::Http,
        };

        let response = if let Some(status) = response_status {
            Some(ResponseData {
                status_code: status,
                headers: response_headers_json
                    .map(|h| serde_json::from_str(&h))
                    .transpose()?
                    .unwrap_or_default(),
                body: response_body,
            })
        } else {
            None
        };

        Ok(CapturedRequest {
            id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)?.into(),
            protocol,
            request: RequestData {
                method,
                uri,
                headers,
                body,
                query_params: Default::default(),
            },
            response,
            duration_ms,
        })
    }

    pub fn count_requests(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM requests", [], |row| row.get(0))?;
        Ok(count)
    }
}
