// src/config_demo.rs

use crate::cache::Cache;
use crate::cache_config::{CacheConfig, ReplacementPolicy};

/// Demonstrate different cache configurations
pub fn demo_cache_configurations() {
    println!("\n=== Cache Configuration Comparison ===");
    
    // Create different cache configurations
    let configs = vec![
        CacheConfig::direct_mapped(16 * 1024, 64),   // Direct-mapped
        CacheConfig::set_associative(16 * 1024, 64, 2, ReplacementPolicy::LRU), // 2-way
        CacheConfig::set_associative(16 * 1024, 64, 4, ReplacementPolicy::LRU), // 4-way
        CacheConfig::fully_associative(4 * 1024, 64, ReplacementPolicy::LRU),   // Fully associative
    ];
    
    // Test pattern: addresses that will show different behavior
    let test_addresses = vec![
        0x1000, 0x2000, 0x3000, 0x4000, 0x5000,  // Sequential
        0x1000, 0x2000,  // Repeat some (test hit rate)
        0x11000, 0x21000, 0x31000,  // Different patterns
    ];
    
    for config in configs {
        println!("--- Testing: {} ---", config.name);
        
        let mut cache = Cache::from_config(config);
        let mut hits = 0;
        
        for &addr in &test_addresses {
            let (hit, _time) = cache.access(addr);
            if hit { hits += 1; }
        }
        
        let hit_rate = hits as f64 / test_addresses.len() as f64 * 100.0;
        println!("Hit rate: {:.1}% ({}/{} hits)", hit_rate, hits, test_addresses.len());
        
        let stats = cache.stats();
        println!("Cache stats: {} sets, {}-way associative\n", 
                 cache.num_sets, cache.associativity);
    }
}
