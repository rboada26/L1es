// src/main.rs

mod line;
mod set;
mod cache;
mod cache_config;
mod config_demo;
mod simulator;
mod cli;
mod measurement;

use clap::Parser;
use std::fs;

pub mod attacks {
    pub mod prime_probe;
    pub mod flush_reload;
    pub mod spectre_sim;
    pub mod meltdown_sim;
}

fn main() {
    let cli = cli::Cli::parse();
    
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&cli.output) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }
    
    // Initialize measurement collector
    let mut collector = measurement::MeasurementCollector::new();
    
    match &cli.command {
        cli::Commands::Basic { size, line_size, ways } => {
            println!("Running basic cache test: {}KB, {}-way, {}-byte lines", 
                     size, ways, line_size);
            run_basic_test(size * 1024, *line_size, *ways, &mut collector);
        },
        
        cli::Commands::Compare { all_policies, all_ways } => {
            println!("Comparing cache configurations...");
            run_comparison(*all_policies, *all_ways, &mut collector);
        },
        
        cli::Commands::Attack { attack_type } => {
            println!("Running attack simulations...");
            run_attacks(attack_type, &mut collector);
        },
        
        cli::Commands::Benchmark { iterations, detailed } => {
            println!("Running benchmark suite with {} iterations...", iterations);
            run_benchmark(*iterations, *detailed, &mut collector);
        },
        
        cli::Commands::Report { configs, scenarios } => {
            println!("Generating attack effectiveness report...");
            generate_report(configs.clone(), scenarios.clone(), &mut collector);
        },
    }
    
    // Save results
    save_results(&cli, &collector);
    
    // Print summary
    if cli.verbose {
        println!("\n{}", collector.generate_summary());
    }
    
    println!("Results saved to: {}", cli.output.display());
}

fn run_basic_test(
    size: usize, 
    line_size: usize, 
    ways: usize, 
    collector: &mut measurement::MeasurementCollector
) {
    use crate::cache_config::{CacheConfig, ReplacementPolicy};
    use crate::cache::Cache;
    use std::collections::HashMap;
    
    let config = CacheConfig::set_associative(size, line_size, ways, ReplacementPolicy::LRU);
    let mut cache = Cache::from_config(config.clone());
    
    println!("Testing {} cache", config.name);
    
    // Run test pattern
    let addresses = vec![0x1000, 0x2000, 0x3000, 0x1000, 0x4000, 0x2000];
    
    for &addr in &addresses {
        let (hit, time) = cache.access(addr);
        collector.record_access(addr, time, hit, cache.timestamp);
        
        println!("Access 0x{:x}: {} ({} cycles)", 
                 addr, if hit { "HIT" } else { "MISS" }, time);
    }
    
    let stats = cache.stats();
    println!("Final hit rate: {:.1}%", stats.hit_rate * 100.0);
    
    // Convert to measurement format
    let cache_stats = measurement::CacheStatistics {
        total_accesses: stats.total_accesses,
        total_hits: stats.total_hits,
        total_misses: stats.total_misses,
        hit_rate: stats.hit_rate,
        miss_rate: 1.0 - stats.hit_rate,
        average_access_time: if stats.total_accesses > 0 {
            collector.current_timings.iter()
                .map(|t| t.access_time as f64)
                .sum::<f64>() / collector.current_timings.len() as f64
        } else { 0.0 },
        cache_size_kb: size / 1024,
        associativity: ways,
        line_size,
        num_sets: cache.num_sets,
    };
    
    collector.complete_simulation(
        config.name,
        "basic_test".to_string(),
        cache_stats,
        None,
        HashMap::new(),
    );
}

fn run_comparison(
    _all_policies: bool, 
    _all_ways: bool, 
    collector: &mut measurement::MeasurementCollector
) {
    use crate::cache_config::{CacheConfig, ReplacementPolicy};
    use crate::cache::Cache;
    use std::collections::HashMap;
    
    println!("=== Cache Configuration Comparison ===");
    
    // Create different cache configurations to test
    let configs = vec![
        CacheConfig::direct_mapped(16 * 1024, 64),
        CacheConfig::set_associative(16 * 1024, 64, 2, ReplacementPolicy::LRU),
        CacheConfig::set_associative(16 * 1024, 64, 4, ReplacementPolicy::LRU),
        CacheConfig::set_associative(16 * 1024, 64, 8, ReplacementPolicy::LRU),
        CacheConfig::fully_associative(4 * 1024, 64, ReplacementPolicy::LRU),
    ];
    
    // Test pattern that shows different cache behavior
    let test_addresses = vec![
        0x1000, 0x2000, 0x3000, 0x4000, 0x5000,  // Sequential
        0x1000, 0x2000,  // Repeat some (test hit rate)
        0x11000, 0x21000, 0x31000,  // Different patterns
    ];
    
    for config in configs {
        println!("--- Testing: {} ---", config.name);
        
        let mut cache = Cache::from_config(config.clone());
        
        // Run test pattern and collect timing data
        for &addr in &test_addresses {
            let (hit, time) = cache.access(addr);
            collector.record_access(addr, time, hit, cache.timestamp);
        }
        
        let stats = cache.stats();
        println!("Hit rate: {:.1}% ({}/{} hits)", 
                 stats.hit_rate * 100.0, stats.total_hits, stats.total_accesses);
        println!("Cache stats: {} sets, {}-way associative\n", 
                 cache.num_sets, cache.associativity);
        
        // Convert to measurement format
        let cache_stats = measurement::CacheStatistics {
            total_accesses: stats.total_accesses,
            total_hits: stats.total_hits,
            total_misses: stats.total_misses,
            hit_rate: stats.hit_rate,
            miss_rate: 1.0 - stats.hit_rate,
            average_access_time: if stats.total_accesses > 0 {
                collector.current_timings.iter()
                    .map(|t| t.access_time as f64)
                    .sum::<f64>() / collector.current_timings.len() as f64
            } else { 0.0 },
            cache_size_kb: config.total_size / 1024,
            associativity: config.associativity(),
            line_size: config.line_size,
            num_sets: config.num_sets(),
        };
        
        // Record this configuration's results
        collector.complete_simulation(
            config.name,
            "configuration_comparison".to_string(),
            cache_stats,
            None,
            HashMap::new(),
        );
    }
}

fn run_attacks(
    attack_type: &cli::AttackCommands, 
    _collector: &mut measurement::MeasurementCollector
) {
    match attack_type {
        cli::AttackCommands::PrimeProbe { target_set } => {
            println!("Running Prime+Probe attack on cache set {}", target_set);
            attacks::prime_probe::demo_prime_probe();
        },
        cli::AttackCommands::FlushReload { addresses } => {
            println!("Running Flush+Reload attack monitoring {} addresses", addresses);
            attacks::flush_reload::demo_flush_reload();
        },
        cli::AttackCommands::Spectre { secret } => {
            println!("Running Spectre attack with secret: '{}'", secret);
            attacks::spectre_sim::demo_spectre_attack();
        },
        cli::AttackCommands::Meltdown { kernel_data } => {
            println!("Running Meltdown attack extracting: '{}'", kernel_data);
            attacks::meltdown_sim::demo_meltdown_attack();
        },
        cli::AttackCommands::All => {
            println!("Running all attack demonstrations...");
            attacks::prime_probe::demo_prime_probe();
            attacks::flush_reload::demo_flush_reload();
            attacks::spectre_sim::demo_spectre_attack();
            attacks::meltdown_sim::demo_meltdown_attack();
        },
    }
}

fn run_benchmark(
    _iterations: usize, 
    _detailed: bool, 
    _collector: &mut measurement::MeasurementCollector
) {
    println!("Benchmark functionality coming soon!");
}

fn generate_report(
    _configs: Vec<String>, 
    _scenarios: Vec<String>, 
    _collector: &mut measurement::MeasurementCollector
) {
    println!("Report generation functionality coming soon!");
}

fn save_results(cli: &cli::Cli, collector: &measurement::MeasurementCollector) {
    if cli.json {
        let json_path = cli.output.join("results.json");
        if let Err(e) = collector.save_json(&json_path) {
            eprintln!("Failed to save JSON results: {}", e);
        } else {
            println!("JSON results saved to: {}", json_path.display());
        }
    }
    
    if cli.csv {
        let csv_path = cli.output.join("results.csv");
        if let Err(e) = collector.save_csv(&csv_path) {
            eprintln!("Failed to save CSV results: {}", e);
        } else {
            println!("CSV results saved to: {}", csv_path.display());
        }
        
        let timing_path = cli.output.join("timing.csv");
        if let Err(e) = collector.save_timing_csv(&timing_path) {
            eprintln!("Failed to save timing data: {}", e);
        } else {
            println!("Timing data saved to: {}", timing_path.display());
        }
    }
}
