use chaos_testing::parsers::{
    http::HttpParser, postgres::PostgresParser, redis::RedisParser, sql::SqlParser,
};

fn main() {
    println!("=== Chaos Testing Parser Demo ===\n");

    demo_sql_parser();
    demo_redis_parser();
    demo_postgres_parser();
    demo_http_parser();
}

fn demo_sql_parser() {
    println!("SQL Parser Demo:");
    println!("----------------");

    let queries = vec![
        "SELECT * FROM users WHERE id = 1",
        "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')",
        "UPDATE users SET active = true WHERE id = 1",
        "DELETE FROM users WHERE id = 1",
        "CREATE TABLE products (id INT, name TEXT)",
    ];

    for query in queries {
        if let Some(parsed) = SqlParser::parse(query) {
            let query_type = SqlParser::classify_query(query);
            let tables = SqlParser::extract_table_names(query);
            println!("  Query: {}", &query[..50.min(query.len())]);
            println!("  Type: {:?}", query_type);
            println!("  Tables: {:?}", tables);
            println!("  Params: {:?}\n", parsed.params);
        }
    }
}

fn demo_redis_parser() {
    println!("\nRedis Parser Demo:");
    println!("------------------");

    let commands = vec![
        ("GET", "key1"),
        ("SET", "key1 value1"),
        ("HGET", "hash field"),
        ("DEL", "key1"),
        ("INCR", "counter"),
    ];

    for (cmd, args) in commands {
        let cmd_type = RedisParser::classify_command(cmd);
        let is_read_only = RedisParser::is_read_only(cmd);
        println!("  Command: {} {}", cmd, args);
        println!("  Type: {:?}", cmd_type);
        println!("  Read-only: {}\n", is_read_only);
    }

    let resp_data = b"*2\r\n$3\r\nGET\r\n$4\r\nkey1\r\n";
    if let Some(parsed) = RedisParser::parse(resp_data) {
        println!("  Parsed RESP: {} {:?}", parsed.command, parsed.args);
    }
}

fn demo_postgres_parser() {
    println!("\nPostgreSQL Parser Demo:");
    println!("-----------------------");

    let simple_query = b"Q\x00\x00\x00\x1aSELECT * FROM users\0";
    if let Some(query) = PostgresParser::parse_simple_query(simple_query) {
        println!("  Simple Query: {}", query.query);
        println!("  Database: {:?}\n", query.database);
    }

    let message_types = vec![b'Q', b'P', b'B', b'E', b'X'];
    for msg_type in message_types {
        if let Some(pg_type) = PostgresParser::message_type(&[msg_type]) {
            println!("  Message '{}': {:?}", msg_type as char, pg_type);
        }
    }
}

fn demo_http_parser() {
    use hyper::{HeaderMap, Method, Uri};

    println!("\n\nHTTP Parser Demo:");
    println!("-----------------");

    let method = Method::GET;
    let uri: Uri = "/api/users/123?filter=active&sort=name".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());

    let request = HttpParser::parse_request(&method, &uri, &headers, None);
    println!("  Method: {}", request.method);
    println!("  URI: {}", request.uri);
    println!("  Query Params: {:?}", request.query_params);
    println!("  Is JSON: {}", HttpParser::is_json_content(&headers));

    let pattern = HttpParser::extract_endpoint_pattern(&uri);
    println!("  Endpoint Pattern: {}", pattern);

    let response = HttpParser::parse_response(200, &headers, Some(b"OK".to_vec()));
    println!("\n  Response Status: {}", response.status_code);
    println!("  Response Headers: {:?}", response.headers);
}
