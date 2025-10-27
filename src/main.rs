use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber;

mod generators;
mod interceptor;
mod models;
mod parsers;
mod storage;

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
            use generators::TestGenerator;
            use std::fs;

            info!("Generating tests from {}", input);
            info!("Target: {} ({})", language, framework.as_deref().unwrap_or("auto"));

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
            println!("Chaos testing not yet implemented");
        }

        Commands::Analyze { input } => {
            info!("Analyzing captured traffic from {}", input);
            println!("Traffic analysis not yet implemented");
        }
    }

    Ok(())
}
