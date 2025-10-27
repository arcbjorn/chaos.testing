use crate::models::CapturedRequest;
use crate::storage::Storage;
use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy)]
pub enum ChaosLevel {
    Mild,
    Moderate,
    Extreme,
}

impl ChaosLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mild" => Self::Mild,
            "extreme" => Self::Extreme,
            _ => Self::Moderate,
        }
    }

    pub fn failure_rate(&self) -> f64 {
        match self {
            Self::Mild => 0.05,
            Self::Moderate => 0.15,
            Self::Extreme => 0.30,
        }
    }

    pub fn max_delay_ms(&self) -> u64 {
        match self {
            Self::Mild => 100,
            Self::Moderate => 500,
            Self::Extreme => 2000,
        }
    }
}

pub struct ChaosEngine {
    storage: Storage,
    level: ChaosLevel,
    target_url: String,
}

impl ChaosEngine {
    pub fn new(storage: Storage, level: ChaosLevel, target_url: String) -> Self {
        Self {
            storage,
            level,
            target_url,
        }
    }

    pub async fn run_chaos_tests(&self) -> Result<ChaosReport> {
        let requests = self.storage.get_all_requests()?;

        if requests.is_empty() {
            anyhow::bail!("No requests found in capture file");
        }

        info!("Running chaos tests with {:?} level", self.level);
        info!("Replaying {} requests", requests.len());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let mut report = ChaosReport {
            total_tests: requests.len(),
            passed: 0,
            failed: 0,
            chaos_injected: 0,
            timeouts: 0,
            errors: Vec::new(),
        };

        for (i, request) in requests.iter().enumerate() {
            info!("Test {}/{}: {} {}", i + 1, requests.len(),
                request.request.method, request.request.uri);

            let should_inject = self.should_inject_chaos();

            if should_inject {
                report.chaos_injected += 1;
                match self.inject_chaos(&client, request, &mut report).await {
                    Ok(_) => report.passed += 1,
                    Err(e) => {
                        report.failed += 1;
                        report.errors.push(format!("{} {}: {}",
                            request.request.method, request.request.uri, e));
                    }
                }
            } else {
                match self.replay_normal(&client, request).await {
                    Ok(_) => report.passed += 1,
                    Err(e) => {
                        report.failed += 1;
                        report.errors.push(format!("{} {}: {}",
                            request.request.method, request.request.uri, e));
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(report)
    }

    fn should_inject_chaos(&self) -> bool {
        use rand::Rng as _;
        let mut rng = rand::rng();
        let random_val: f64 = rng.random();
        random_val < self.level.failure_rate()
    }

    async fn inject_chaos(
        &self,
        client: &reqwest::Client,
        request: &CapturedRequest,
        report: &mut ChaosReport,
    ) -> Result<()> {
        use rand::Rng as _;
        let mut rng = rand::rng();
        let chaos_type = rng.random_range(0..3);

        match chaos_type {
            0 => {
                let delay = rng.random_range(0..self.level.max_delay_ms());
                warn!("Injecting delay: {}ms", delay);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                self.replay_normal(client, request).await
            }
            1 => {
                warn!("Injecting timeout");
                report.timeouts += 1;
                let short_timeout = Duration::from_millis(1);
                let short_client = reqwest::Client::builder()
                    .timeout(short_timeout)
                    .build()?;
                self.replay_normal(&short_client, request).await
            }
            _ => {
                warn!("Simulating connection error");
                Err(anyhow::anyhow!("Chaos: simulated connection failure"))
            }
        }
    }

    async fn replay_normal(
        &self,
        client: &reqwest::Client,
        request: &CapturedRequest,
    ) -> Result<()> {
        let url = format!("{}{}", self.target_url, request.request.uri);

        let mut req_builder = match request.request.method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            _ => client.get(&url),
        };

        for (key, value) in &request.request.headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder.send().await?;
        let status = response.status();

        if let Some(expected_response) = &request.response
            && status.as_u16() != expected_response.status_code {
                anyhow::bail!(
                    "Status mismatch: expected {}, got {}",
                    expected_response.status_code,
                    status.as_u16()
                );
            }

        Ok(())
    }
}

pub struct ChaosReport {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub chaos_injected: usize,
    pub timeouts: usize,
    pub errors: Vec<String>,
}

impl ChaosReport {
    pub fn print(&self) {
        println!("\n=== Chaos Testing Report ===\n");
        println!("Total Tests: {}", self.total_tests);
        println!("Passed: {} ({:.1}%)", self.passed,
            (self.passed as f64 / self.total_tests as f64) * 100.0);
        println!("Failed: {} ({:.1}%)", self.failed,
            (self.failed as f64 / self.total_tests as f64) * 100.0);
        println!("Chaos Injected: {}", self.chaos_injected);
        println!("Timeouts: {}", self.timeouts);

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for (i, error) in self.errors.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            if self.errors.len() > 10 {
                println!("  ... and {} more", self.errors.len() - 10);
            }
        }

        println!("\n");
    }
}
