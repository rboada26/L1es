// src/set.rs

use crate::line::CacheLine;
use crate::cache_config::ReplacementPolicy;

pub struct CacheSet {
    pub lines: Vec<CacheLine>,
    pub associativity: usize,        // Number of lines in this set (e.g., 8-way)
    pub access_count: u64,           // Total accesses to this set
    pub hit_count: u64,              // Cache hits in this set
    pub miss_count: u64,             // Cache misses in this set
    pub replacement_policy: ReplacementPolicy, // How to choose victim for replacement
    pub fifo_index: usize,           // For FIFO replacement
}

impl CacheSet {
    /// Create a new cache set with given associativity and line size
    pub fn new(associativity: usize, line_size: usize, policy: ReplacementPolicy) -> Self {
        let mut lines = Vec::with_capacity(associativity);
        for _ in 0..associativity {
            lines.push(CacheLine::new(line_size));
        }
        
        CacheSet {
            lines,
            associativity,
            access_count: 0,
            hit_count: 0,
            miss_count: 0,
            replacement_policy: policy,
            fifo_index: 0,
        }
    }
    
    /// Access this set with the given tag - returns (hit, evicted_tag)
    /// hit = true if cache hit, false if miss
    /// evicted_tag = Some(tag) if an existing line was evicted, None otherwise
    pub fn access(&mut self, tag: u64, timestamp: u64) -> (bool, Option<u64>) {
        self.access_count += 1;
        
        // Check for cache hit
        for line in &mut self.lines {
            if line.is_hit(tag) {
                line.update(tag, timestamp);
                self.hit_count += 1;
                return (true, None);
            }
        }
        
        // Cache miss - need to find a line to replace
        self.miss_count += 1;
        let evicted_tag = self.replace_line(tag, timestamp);
        (false, evicted_tag)
    }
    
    /// Replace a line using the configured replacement policy
    fn replace_line(&mut self, tag: u64, timestamp: u64) -> Option<u64> {
        // First, try to find an invalid line
        for line in &mut self.lines {
            if !line.valid {
                line.update(tag, timestamp);
                return None;
            }
        }
        
        // All lines valid - use replacement policy
        let victim_idx = match self.replacement_policy {
            ReplacementPolicy::LRU => self.find_lru_victim(),
            ReplacementPolicy::FIFO => self.find_fifo_victim(),
            ReplacementPolicy::Random => self.find_random_victim(),
        };
        
        let evicted_tag = self.lines[victim_idx].tag;
        self.lines[victim_idx].update(tag, timestamp);
        Some(evicted_tag)
    }
    
    /// Find LRU (Least Recently Used) victim
    fn find_lru_victim(&self) -> usize {
        let mut lru_idx = 0;
        let mut oldest_time = self.lines[0].last_access;
        
        for (i, line) in self.lines.iter().enumerate() {
            if line.last_access < oldest_time {
                oldest_time = line.last_access;
                lru_idx = i;
            }
        }
        
        lru_idx
    }
    
    /// Find FIFO (First In, First Out) victim
    fn find_fifo_victim(&mut self) -> usize {
        let victim = self.fifo_index;
        self.fifo_index = (self.fifo_index + 1) % self.associativity;
        victim
    }
    
    /// Find random victim
    fn find_random_victim(&self) -> usize {
        // Simple pseudo-random based on access count
        (self.access_count as usize) % self.associativity
    }
    
    /// Flush a specific tag from this set (for Flush+Reload attacks)
    pub fn flush(&mut self, tag: u64) -> bool {
        for line in &mut self.lines {
            if line.is_hit(tag) {
                line.invalidate();
                return true;
            }
        }
        false
    }
    
    /// Get hit rate for this set
    pub fn hit_rate(&self) -> f64 {
        if self.access_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / self.access_count as f64
        }
    }
    
    /// Check if this set is full (all lines valid)
    pub fn is_full(&self) -> bool {
        self.lines.iter().all(|line| line.valid)
    }
    
    /// Get the number of valid lines in this set
    pub fn valid_lines(&self) -> usize {
        self.lines.iter().filter(|line| line.valid).count()
    }
}
