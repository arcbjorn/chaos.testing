use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{Level, info};

mod analyzer;
mod chaos;
mod generators;
mod interceptor;
mod models;
mod parsers;
mod storage;
mod utils;

#[derive(Parser)]
#[command(name = "chaos-testing")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Language-Agnostic Backend Testing Framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Observe a running application and capture traffic
    Observe {
        #[arg(short, long, conflicts_with = "port")]
        pid: Option<u32>,

        #[arg(short = 'P', long, conflicts_with = "pid")]
        port: Option<u16>,

        #[arg(short, long, default_value = "60s")]
        duration: String,

        #[arg(short, long, default_value = "chaos-capture.db")]
        output: String,

        #[arg(short, long)]
        target: Option<String>,
    },

    /// Generate tests from captured traffic
    Generate {
        #[arg(short, long, default_value = "chaos-capture.db")]
        input: String,

        #[arg(short, long, default_value = "auto")]
        language: String,

        #[arg(short, long)]
        framework: Option<String>,

        #[arg(short, long, default_value = "tests")]
        output: String,
    },

    /// Run chaos testing scenarios
    Chaos {
        #[arg(short, long, default_value = "moderate")]
        level: String,

        #[arg(short, long, default_value = "chaos-capture.db")]
        input: String,

        #[arg(short, long)]
        url: String,
    },

    /// Analyze captured traffic without generating tests
    Analyze {
        #[arg(short, long, default_value = "chaos-capture.db")]
        input: String,
    },

    /// Parse and analyze a query or command
    Parse {
        #[arg(short, long)]
        query: String,

        #[arg(short, long, default_value = "sql")]
        protocol: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    info!("Chaos Testing - Language-Agnostic Backend Testing");

    match cli.command {
        Commands::Observe {
            pid,
            port,
            duration,
            output,
            target,
        } => {
            if let Some(pid) = pid {
                info!("Observing process {} for {}", pid, duration);
                info!("Output: {}", output);
                println!("Process observation not yet implemented");
            } else if let Some(port) = port {
                info!("Intercepting traffic on port {} for {}", port, duration);
                info!("Output: {}", output);

                let mut interceptor = interceptor::HttpInterceptor::new(port, output);
                if let Some(target_url) = target {
                    interceptor = interceptor.with_target(target_url);
                }
                interceptor.start().await?;
            } else {
                anyhow::bail!("Either --pid or --port must be specified");
            }
        }

        Commands::Generate {
            input,
            language,
            framework,
            output,
        } => {
            use std::fs;

            info!("Generating tests from {}", input);
            info!(
                "Target: {} ({})",
                language,
                framework.as_deref().unwrap_or("auto")
            );

            let storage = storage::Storage::new(&input)?;
            let requests = storage.get_all_requests()?;

            info!("Loaded {} captured requests", requests.len());

            if requests.is_empty() {
                println!("No requests found in capture file");
                return Ok(());
            }

            let generator = generators::get_generator(&language, framework.as_deref())?;
            let test_code = generator.generate(&requests)?;

            fs::create_dir_all(&output)?;
            let filename = format!("{}/test_generated.{}", output, generator.file_extension());
            fs::write(&filename, test_code)?;

            info!("Generated tests written to: {}", filename);
            println!("✓ Generated {} tests in {}", requests.len(), filename);
        }

        Commands::Chaos { level, input, url } => {
            info!("Running chaos testing at {} level", level);
            info!("Using capture: {}", input);
            info!("Target: {}", url);

            let storage = storage::Storage::new(&input)?;
            let chaos_level = chaos::ChaosLevel::from_str(&level);
            let engine = chaos::ChaosEngine::new(storage, chaos_level, url);

            let report = engine.run_chaos_tests().await?;
            report.print();
        }

        Commands::Analyze { input } => {
            info!("Analyzing captured traffic from {}", input);

            let storage = storage::Storage::new(&input)?;

            let total = storage.count_requests()?;
            info!("Total requests in database: {}", total);

            let endpoints = storage.get_unique_endpoints()?;
            info!("Found {} unique endpoints", endpoints.len());

            for endpoint in endpoints.iter().take(3) {
                let endpoint_requests = storage.get_requests_by_endpoint(endpoint)?;
                info!("  {}: {} requests", endpoint, endpoint_requests.len());
            }

            let analyzer = analyzer::Analyzer::new(storage);
            let report = analyzer.analyze()?;

            report.print();
        }

        Commands::Parse { query, protocol } => {
            use parsers::grpc::GrpcParser;
            use parsers::http::HttpParser;
            use parsers::kafka::KafkaParser;
            use parsers::postgres::PostgresParser;
            use parsers::redis::RedisParser;
            use parsers::sql::SqlParser;

            info!("Parsing query with {} protocol", protocol);

            match protocol.as_str() {
                "sql" => {
                    if let Some(parsed) = SqlParser::parse(&query) {
                        let query_type = SqlParser::classify_query(&query);
                        let tables = SqlParser::extract_table_names(&query);
                        println!("SQL Query Analysis:");
                        println!("  Type: {:?}", query_type);
                        println!("  Tables: {:?}", tables);
                        println!("  Params: {:?}", parsed.params);
                    } else {
                        println!("Failed to parse SQL query");
                    }
                }
                "redis" => {
                    let cmd = query.split_whitespace().next().unwrap_or("");
                    let cmd_type = RedisParser::classify_command(cmd);
                    let is_read_only = RedisParser::is_read_only(cmd);
                    println!("Redis Command Analysis:");
                    println!("  Command: {}", cmd);
                    println!("  Type: {:?}", cmd_type);
                    println!("  Read-only: {}", is_read_only);

                    let resp_format =
                        format!("*2\r\n$3\r\n{}\r\n$3\r\nkey\r\n", cmd.to_uppercase());
                    if let Some(parsed) = RedisParser::parse(resp_format.as_bytes()) {
                        println!("  Parsed RESP: {} {:?}", parsed.command, parsed.args);
                    }
                }
                "postgres" => {
                    println!("PostgreSQL Query Analysis:");

                    let mut pg_query = vec![b'Q'];
                    let query_with_null = format!("{}\0", query);
                    let len = (query_with_null.len() + 4) as u32;
                    pg_query.extend_from_slice(&len.to_be_bytes());
                    pg_query.extend_from_slice(query_with_null.as_bytes());

                    if let Some(msg_type) = PostgresParser::message_type(&pg_query) {
                        println!("  Message Type: {:?}", msg_type);
                    }

                    if let Some(parsed) = PostgresParser::parse_simple_query(&pg_query) {
                        println!("  Parsed Query: {}", parsed.query);
                        println!("  Database: {:?}", parsed.database);
                    }

                    let tables = PostgresParser::extract_table_names(&query);
                    if !tables.is_empty() {
                        println!("  Tables: {:?}", tables);
                    }

                    let prepared = format!("test_stmt\0{}\0", query);
                    let mut prep_msg = vec![b'P'];
                    let len = (prepared.len() + 4) as u32;
                    prep_msg.extend_from_slice(&len.to_be_bytes());
                    prep_msg.extend_from_slice(prepared.as_bytes());

                    if let Some((name, parsed)) =
                        PostgresParser::parse_prepared_statement(&prep_msg)
                    {
                        println!("  Prepared Statement: {}", name);
                        println!("  Query: {}", parsed.query);
                    }
                }
                "http" => {
                    use hyper::{HeaderMap, Method, Uri};
                    let uri: Uri = query.parse()?;
                    let method = Method::GET;
                    let headers = HeaderMap::new();
                    let pattern = HttpParser::extract_endpoint_pattern(&uri);
                    let request = HttpParser::parse_request(&method, &uri, &headers, None);
                    println!("HTTP Request Analysis:");
                    println!("  Endpoint Pattern: {}", pattern);
                    println!("  Query Params: {:?}", request.query_params);

                    let response = HttpParser::parse_response(200, &headers, None);
                    println!("  Response Status: {}", response.status_code);
                }
                "kafka" => {
                    println!("Kafka Topic Analysis:");
                    if let Some(topic) = KafkaParser::extract_topic(&query) {
                        println!("  Topic: {}", topic);
                        let msg_type = KafkaParser::classify_by_topic(&topic);
                        println!("  Type: {:?}", msg_type);
                    } else {
                        let msg_type = KafkaParser::classify_by_topic(&query);
                        println!("  Topic: {}", query);
                        println!("  Type: {:?}", msg_type);
                    }

                    let msg =
                        KafkaParser::parse_message(&query, 0, 0, None, Some(b"data".to_vec()));
                    println!("  Topic: {}", msg.topic);
                    println!("  Partition: {}", msg.partition);
                    println!("  Offset: {}", msg.offset);
                    println!("  Key: {:?}", msg.key);
                    println!(
                        "  Value size: {} bytes",
                        msg.value.as_ref().map(|v| v.len()).unwrap_or(0)
                    );
                }
                "grpc" => {
                    println!("gRPC Service Analysis:");
                    if let Some((service, method)) = GrpcParser::parse_service_path(&query) {
                        println!("  Service: {}", service);
                        println!("  Method: {}", method);

                        if let Some(pkg) = GrpcParser::extract_package(&service) {
                            println!("  Package: {}", pkg);
                        }

                        let method_type = GrpcParser::classify_method(&method);
                        println!("  Type: {:?}", method_type);
                        println!("  Streaming: {}", GrpcParser::is_streaming(&method));
                    } else {
                        println!("  Invalid gRPC path format");
                        println!("  Expected: /package.Service/Method");
                    }
                }
                _ => {
                    println!("Unknown protocol: {}", protocol);
                    println!("Supported: sql, redis, postgres, http, kafka, grpc");
                }
            }
        }
    }

    Ok(())
}
