// src/measurement.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    pub timestamp: DateTime<Utc>,
    pub config_name: String,
    pub test_name: String,
    pub cache_stats: CacheStatistics,
    pub attack_results: Option<AttackResults>,
    pub timing_data: Vec<AccessTiming>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_accesses: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub average_access_time: f64,
    pub cache_size_kb: usize,
    pub associativity: usize,
    pub line_size: usize,
    pub num_sets: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackResults {
    pub attack_type: String,
    pub success_rate: f64,
    pub detection_accuracy: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub leaked_bytes: usize,
    pub total_bytes: usize,
    pub attack_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTiming {
    pub address: u64,
    pub access_time: u64,
    pub cache_hit: bool,
    pub timestamp: u64,
}

pub struct MeasurementCollector {
    pub results: Vec<SimulationResult>,
    pub current_timings: Vec<AccessTiming>,
    pub start_time: std::time::Instant,
}

impl MeasurementCollector {
    pub fn new() -> Self {
        MeasurementCollector {
            results: Vec::new(),
            current_timings: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    pub fn record_access(&mut self, address: u64, access_time: u64, cache_hit: bool, timestamp: u64) {
        self.current_timings.push(AccessTiming {
            address,
            access_time,
            cache_hit,
            timestamp,
        });
    }
    
    pub fn complete_simulation(
        &mut self, 
        config_name: String,
        test_name: String,
        cache_stats: CacheStatistics,
        attack_results: Option<AttackResults>,
        metadata: HashMap<String, String>
    ) {
        let result = SimulationResult {
            timestamp: Utc::now(),
            config_name,
            test_name,
            cache_stats,
            attack_results,
            timing_data: self.current_timings.clone(),
            metadata,
        };
        
        self.results.push(result);
        self.current_timings.clear();
    }
    
    /// Save results as JSON
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.results)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Save results as CSV
    pub fn save_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        
        // Write header
        wtr.write_record(&[
            "timestamp", "config_name", "test_name", "hit_rate", "miss_rate",
            "avg_access_time", "cache_size_kb", "associativity", "attack_success",
            "leaked_bytes", "total_accesses"
        ])?;
        
        // Write data
        for result in &self.results {
            wtr.write_record(&[
                result.timestamp.to_rfc3339(),
                result.config_name.clone(),
                result.test_name.clone(),
                result.cache_stats.hit_rate.to_string(),
                result.cache_stats.miss_rate.to_string(),
                result.cache_stats.average_access_time.to_string(),
                result.cache_stats.cache_size_kb.to_string(),
                result.cache_stats.associativity.to_string(),
                result.attack_results.as_ref()
                    .map(|a| a.success_rate.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                result.attack_results.as_ref()
                    .map(|a| a.leaked_bytes.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                result.cache_stats.total_accesses.to_string(),
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    /// Save detailed timing data as CSV
    pub fn save_timing_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        
        // Write header
        wtr.write_record(&["simulation_id", "address", "access_time", "cache_hit", "timestamp"])?;
        
        // Write timing data for each simulation
        for (sim_id, result) in self.results.iter().enumerate() {
            for timing in &result.timing_data {
                wtr.write_record(&[
                    sim_id.to_string(),
                    format!("0x{:x}", timing.address),
                    timing.access_time.to_string(),
                    timing.cache_hit.to_string(),
                    timing.timestamp.to_string(),
                ])?;
            }
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    /// Generate summary report
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("=== Cache Simulation Summary ===\n\n");
        
        summary.push_str(&format!("Total simulations: {}\n", self.results.len()));
        summary.push_str(&format!("Total execution time: {:.2}s\n\n", 
            self.start_time.elapsed().as_secs_f64()));
        
        // Group results by configuration
        let mut config_groups: HashMap<String, Vec<&SimulationResult>> = HashMap::new();
        for result in &self.results {
            config_groups.entry(result.config_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        for (config, results) in config_groups {
            summary.push_str(&format!("--- {} ---\n", config));
            
            let avg_hit_rate: f64 = results.iter()
                .map(|r| r.cache_stats.hit_rate)
                .sum::<f64>() / results.len() as f64;
                
            summary.push_str(&format!("  Average hit rate: {:.1}%\n", avg_hit_rate * 100.0));
            
            let successful_attacks = results.iter()
                .filter_map(|r| r.attack_results.as_ref())
                .filter(|a| a.success_rate > 0.5)
                .count();
                
            summary.push_str(&format!("  Successful attacks: {}/{}\n", 
                successful_attacks, 
                results.iter().filter(|r| r.attack_results.is_some()).count()));
            
            summary.push_str("\n");
        }
        
        summary
    }
}
