// src/cache.rs

use crate::set::CacheSet;
use crate::cache_config::{CacheConfig, CacheType};

pub struct Cache {
    pub sets: Vec<CacheSet>,
    pub config: CacheConfig,         // Cache configuration
    pub num_sets: usize,             // Number of cache sets
    pub associativity: usize,        // Ways per set
    pub line_size: usize,            // Bytes per line
    pub index_bits: usize,           // Bits for set index
    pub offset_bits: usize,          // Bits for byte offset within line
    pub timestamp: u64,              // Global timestamp for LRU
}

impl Cache {
    /// Create a new cache from configuration
    pub fn from_config(config: CacheConfig) -> Self {
        let num_sets = config.num_sets();
        let associativity = config.associativity();
        let line_size = config.line_size;
        
        let mut sets = Vec::with_capacity(num_sets);
        for _ in 0..num_sets {
            sets.push(CacheSet::new(associativity, line_size, config.replacement_policy));
        }
        
        let index_bits = if num_sets > 1 { (num_sets as f64).log2() as usize } else { 0 };
        let offset_bits = (line_size as f64).log2() as usize;
        
        Cache {
            sets,
            config,
            num_sets,
            associativity,
            line_size,
            index_bits,
            offset_bits,
            timestamp: 0,
        }
    }
    
    /// Create a new cache with given parameters (legacy interface)
    pub fn new(num_sets: usize, associativity: usize, line_size: usize) -> Self {
        let total_size = num_sets * associativity * line_size;
        let config = CacheConfig::set_associative(
            total_size, 
            line_size, 
            associativity, 
            crate::cache_config::ReplacementPolicy::LRU
        );
        Self::from_config(config)
    }
    
    /// Access cache at given address - returns (hit, access_time_cycles)
    pub fn access(&mut self, address: u64) -> (bool, u64) {
        self.timestamp += 1;
        
        let (tag, index, _offset) = self.decompose_address(address);
        let (hit, _evicted) = self.sets[index].access(tag, self.timestamp);
        
        // Simulate timing: cache hit = 1 cycle, miss = 100 cycles
        let access_time = if hit { 1 } else { 100 };
        
        (hit, access_time)
    }
    
    /// Decompose address into (tag, index, offset)
    fn decompose_address(&self, address: u64) -> (u64, usize, usize) {
        let offset = (address & ((1 << self.offset_bits) - 1)) as usize;
        
        let index = if self.index_bits > 0 {
            ((address >> self.offset_bits) & ((1 << self.index_bits) - 1)) as usize
        } else {
            0 // Fully associative cache
        };
        
        let tag = address >> (self.offset_bits + self.index_bits);
        
        (tag, index, offset)
    }
    
    /// Flush specific address (for Flush+Reload attacks)
    pub fn flush(&mut self, address: u64) -> bool {
        let (tag, index, _offset) = self.decompose_address(address);
        self.sets[index].flush(tag)
    }
    
    /// Get overall cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total_accesses = 0;
        let mut total_hits = 0;
        
        for set in &self.sets {
            total_accesses += set.access_count;
            total_hits += set.hit_count;
        }
        
        CacheStats {
            total_accesses,
            total_hits,
            total_misses: total_accesses - total_hits,
            hit_rate: if total_accesses == 0 { 0.0 } else { total_hits as f64 / total_accesses as f64 },
            config_name: self.config.name.clone(),
        }
    }
    
    /// Get detailed statistics per set
    pub fn detailed_stats(&self) -> Vec<SetStats> {
        self.sets.iter().enumerate().map(|(i, set)| {
            SetStats {
                set_index: i,
                accesses: set.access_count,
                hits: set.hit_count,
                misses: set.miss_count,
                hit_rate: set.hit_rate(),
                valid_lines: set.valid_lines(),
            }
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_accesses: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub config_name: String,
}

#[derive(Debug)]
pub struct SetStats {
    pub set_index: usize,
    pub accesses: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub valid_lines: usize,
}
