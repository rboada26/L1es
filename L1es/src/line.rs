// src/line.rs

pub struct CacheLine {
    pub valid: bool,           // Is this line valid?
    pub tag: u64,             // Tag bits for address matching
    pub data: Vec<u8>,        // Actual cached data (or just simulate size)
    pub last_access: u64,     // For LRU replacement policy
    pub access_count: u64,    // For statistics/debugging
}

impl CacheLine {
    /// Create a new empty cache line
    pub fn new(line_size: usize) -> Self {
        CacheLine {
            valid: false,
            tag: 0,
            data: vec![0; line_size], // Initialize with zeros
            last_access: 0,
            access_count: 0,
        }
    }
    
    /// Check if this line contains the given tag (cache hit)
    pub fn is_hit(&self, tag: u64) -> bool {
        self.valid && self.tag == tag
    }
    
    /// Update this line with new data (on cache miss)
    pub fn update(&mut self, tag: u64, timestamp: u64) {
        self.valid = true;
        self.tag = tag;
        self.last_access = timestamp;
        self.access_count += 1;
        // Note: In a real cache, we'd copy actual data here
        // For simulation, we just update metadata
    }
    
    /// Mark this line as invalid (for flush operations)
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.tag = 0;
        self.last_access = 0;
        // Keep access_count for statistics
    }
}
