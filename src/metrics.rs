use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub parse_count: usize,
    pub parse_ns_p50: u64,
    pub parse_ns_p95: u64,
    pub parse_ns_p99: u64,
    pub retries: usize,
    pub timeouts: usize,
    pub total_duration_ms: u64,
    pub rpc_calls: usize,
    pub rpc_duration_ms: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            parse_count: 0,
            parse_ns_p50: 0,
            parse_ns_p95: 0,
            parse_ns_p99: 0,
            retries: 0,
            timeouts: 0,
            total_duration_ms: 0,
            rpc_calls: 0,
            rpc_duration_ms: 0,
        }
    }
}

pub struct MetricsCollector {
    start_time: Instant,
    parse_times: Vec<u64>,
    retry_count: usize,
    timeout_count: usize,
    rpc_call_count: usize,
    rpc_duration: std::time::Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            parse_times: Vec::new(),
            retry_count: 0,
            timeout_count: 0,
            rpc_call_count: 0,
            rpc_duration: std::time::Duration::ZERO,
        }
    }
    
    pub fn record_parse_time(&mut self, duration: std::time::Duration) {
        self.parse_times.push(duration.as_nanos() as u64);
    }
    
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }
    
    pub fn record_timeout(&mut self) {
        self.timeout_count += 1;
    }
    
    pub fn record_rpc_call(&mut self, duration: std::time::Duration) {
        self.rpc_call_count += 1;
        self.rpc_duration += duration;
    }
    
    pub fn finalize(self) -> Metrics {
        let total_duration = self.start_time.elapsed();
        let mut parse_times = self.parse_times;
        parse_times.sort();
        
        let parse_ns_p50 = if parse_times.is_empty() { 0 } else {
            parse_times[parse_times.len() / 2]
        };
        
        let parse_ns_p95 = if parse_times.len() < 20 { 
            parse_times.last().copied().unwrap_or(0) 
        } else {
            parse_times[(parse_times.len() * 95) / 100]
        };
        
        let parse_ns_p99 = if parse_times.len() < 100 { 
            parse_times.last().copied().unwrap_or(0) 
        } else {
            parse_times[(parse_times.len() * 99) / 100]
        };
        
        Metrics {
            parse_count: parse_times.len(),
            parse_ns_p50,
            parse_ns_p95,
            parse_ns_p99,
            retries: self.retry_count,
            timeouts: self.timeout_count,
            total_duration_ms: total_duration.as_millis() as u64,
            rpc_calls: self.rpc_call_count,
            rpc_duration_ms: self.rpc_duration.as_millis() as u64,
        }
    }
}
