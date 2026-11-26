// src/line.rs

pub struct CacheLine {
    pub valid: bool,           // Is this line valid?
    pub dirty: bool,           // Modified since it was filled from memory?
    pub tag: u64,              // Tag bits for address matching
    pub data: Vec<u8>,         // Cached copy of the memory line
    pub last_access: u64,      // For LRU replacement policy
    pub access_count: u64,     // For statistics/debugging
}

impl CacheLine {
    /// Create a new empty cache line
    pub fn new(line_size: usize) -> Self {
        CacheLine {
            valid: false,
            dirty: false,
            tag: 0,
            data: vec![0; line_size],
            last_access: 0,
            access_count: 0,
        }
    }

    /// Check if this line contains the given tag (cache hit)
    pub fn is_hit(&self, tag: u64) -> bool {
        self.valid && self.tag == tag
    }

    /// Fill this line with data fetched from memory (on a cache miss)
    pub fn fill(&mut self, tag: u64, data: Vec<u8>, timestamp: u64) {
        debug_assert_eq!(data.len(), self.data.len(), "fill data has wrong length");
        self.valid = true;
        self.dirty = false;
        self.tag = tag;
        self.data = data;
        self.last_access = timestamp;
        self.access_count += 1;
    }

    /// Record a hit on this line (for LRU and statistics)
    pub fn touch(&mut self, timestamp: u64) {
        self.last_access = timestamp;
        self.access_count += 1;
    }

    /// Mark this line as invalid (for flush operations).
    /// Callers must write back dirty data first.
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.dirty = false;
        self.tag = 0;
        self.last_access = 0;
        // Keep access_count for statistics
    }
}
