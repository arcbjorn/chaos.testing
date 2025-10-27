use crate::storage::Storage;
use anyhow::Result;
use std::collections::HashMap;

pub struct Analyzer {
    storage: Storage,
}

impl Analyzer {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn analyze(&self) -> Result<AnalysisReport> {
        let requests = self.storage.get_all_requests()?;

        if requests.is_empty() {
            return Ok(AnalysisReport::default());
        }

        let total_requests = requests.len();
        let mut endpoint_stats: HashMap<String, EndpointStats> = HashMap::new();
        let mut status_codes: HashMap<u16, usize> = HashMap::new();
        let mut total_duration = 0u64;
        let mut methods: HashMap<String, usize> = HashMap::new();

        for req in &requests {
            let endpoint = format!("{} {}", req.request.method, req.request.uri);

            let stats = endpoint_stats.entry(endpoint.clone()).or_insert(EndpointStats {
                endpoint,
                count: 0,
                avg_duration_ms: 0.0,
                min_duration_ms: u64::MAX,
                max_duration_ms: 0,
                success_rate: 0.0,
                success_count: 0,
            });

            stats.count += 1;

            if let Some(duration) = req.duration_ms {
                total_duration += duration;
                stats.min_duration_ms = stats.min_duration_ms.min(duration);
                stats.max_duration_ms = stats.max_duration_ms.max(duration);
            }

            if let Some(response) = &req.response {
                *status_codes.entry(response.status_code).or_insert(0) += 1;
                if response.status_code < 400 {
                    stats.success_count += 1;
                }
            }

            *methods.entry(req.request.method.clone()).or_insert(0) += 1;
        }

        for stats in endpoint_stats.values_mut() {
            stats.success_rate = (stats.success_count as f64 / stats.count as f64) * 100.0;

            let total_endpoint_duration: u64 = requests
                .iter()
                .filter(|r| format!("{} {}", r.request.method, r.request.uri) == stats.endpoint)
                .filter_map(|r| r.duration_ms)
                .sum();

            stats.avg_duration_ms = total_endpoint_duration as f64 / stats.count as f64;
        }

        let avg_response_time = if total_requests > 0 {
            total_duration as f64 / total_requests as f64
        } else {
            0.0
        };

        let success_count = status_codes
            .iter()
            .filter(|(code, _)| **code < 400)
            .map(|(_, count)| count)
            .sum();

        let error_count = status_codes
            .iter()
            .filter(|(code, _)| **code >= 400)
            .map(|(_, count)| count)
            .sum();

        let mut endpoints: Vec<EndpointStats> = endpoint_stats.into_values().collect();
        endpoints.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(AnalysisReport {
            total_requests,
            unique_endpoints: endpoints.len(),
            avg_response_time_ms: avg_response_time,
            success_count,
            error_count,
            status_codes,
            methods,
            endpoints,
        })
    }
}

#[derive(Debug, Default)]
pub struct AnalysisReport {
    pub total_requests: usize,
    pub unique_endpoints: usize,
    pub avg_response_time_ms: f64,
    pub success_count: usize,
    pub error_count: usize,
    pub status_codes: HashMap<u16, usize>,
    pub methods: HashMap<String, usize>,
    pub endpoints: Vec<EndpointStats>,
}

#[derive(Debug)]
pub struct EndpointStats {
    pub endpoint: String,
    pub count: usize,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub success_rate: f64,
    pub success_count: usize,
}

impl AnalysisReport {
    pub fn print(&self) {
        println!("\n=== Traffic Analysis Report ===\n");

        println!("Overview:");
        println!("  Total Requests: {}", self.total_requests);
        println!("  Unique Endpoints: {}", self.unique_endpoints);
        println!("  Avg Response Time: {:.2}ms", self.avg_response_time_ms);
        println!("  Success: {} ({:.1}%)", self.success_count,
            (self.success_count as f64 / self.total_requests as f64) * 100.0);
        println!("  Errors: {} ({:.1}%)", self.error_count,
            (self.error_count as f64 / self.total_requests as f64) * 100.0);

        println!("\nHTTP Methods:");
        let mut methods: Vec<_> = self.methods.iter().collect();
        methods.sort_by(|a, b| b.1.cmp(a.1));
        for (method, count) in methods {
            println!("  {}: {}", method, count);
        }

        println!("\nStatus Codes:");
        let mut codes: Vec<_> = self.status_codes.iter().collect();
        codes.sort_by_key(|a| a.0);
        for (code, count) in codes {
            println!("  {}: {}", code, count);
        }

        println!("\nTop Endpoints:");
        for (i, stats) in self.endpoints.iter().take(10).enumerate() {
            println!("\n{}. {} (called {} times)", i + 1, stats.endpoint, stats.count);
            println!("   Avg: {:.2}ms | Min: {}ms | Max: {}ms",
                stats.avg_duration_ms, stats.min_duration_ms, stats.max_duration_ms);
            println!("   Success Rate: {:.1}%", stats.success_rate);
        }

        println!("\n");
    }
}
