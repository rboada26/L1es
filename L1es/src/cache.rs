// src/cache.rs

use crate::set::CacheSet;
use crate::cache_config::{CacheConfig, CacheType};
use crate::memory::Memory;

/// Cycles for a cache hit.
pub const HIT_LATENCY: u64 = 1;

pub struct Cache {
    pub sets: Vec<CacheSet>,
    pub config: CacheConfig,         // Cache configuration
    pub num_sets: usize,             // Number of cache sets
    pub associativity: usize,        // Ways per set
    pub line_size: usize,            // Bytes per line
    pub index_bits: usize,           // Bits for set index
    pub offset_bits: usize,          // Bits for byte offset within line
    pub timestamp: u64,              // Global timestamp for LRU
    pub memory: Memory,              // Backing main memory
    pub writebacks: u64,             // Dirty lines written back to memory
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
            memory: Memory::new(line_size),
            writebacks: 0,
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
    
    /// Access cache at given address - returns (hit, access_time_cycles).
    /// Legacy interface: behaves as a read whose value is discarded.
    pub fn access(&mut self, address: u64) -> (bool, u64) {
        let (_set, _way, hit, cycles) = self.get_line(address);
        (hit, cycles)
    }

    /// Read one byte - returns (value, access_time_cycles)
    pub fn read(&mut self, address: u64) -> (u8, u64) {
        let (set, way, _hit, cycles) = self.get_line(address);
        let (_, _, offset) = self.decompose_address(address);
        (self.sets[set].lines[way].data[offset], cycles)
    }

    /// Write one byte (write-back, write-allocate) - returns access_time_cycles
    pub fn write(&mut self, address: u64, value: u8) -> u64 {
        let (set, way, _hit, cycles) = self.get_line(address);
        let (_, _, offset) = self.decompose_address(address);
        let line = &mut self.sets[set].lines[way];
        line.data[offset] = value;
        line.dirty = true;
        cycles
    }

    /// Ensure the line containing `address` is cached.
    /// Returns (set_index, way, hit, cycles).
    fn get_line(&mut self, address: u64) -> (usize, usize, bool, u64) {
        self.timestamp += 1;
        let (tag, index, _offset) = self.decompose_address(address);

        if let Some(way) = self.sets[index].lookup(tag, self.timestamp) {
            return (index, way, true, HIT_LATENCY);
        }

        // Miss: evict (writing back if dirty), then fill from memory.
        let way = self.sets[index].choose_victim();
        let mut cycles = self.memory.latency;
        if self.write_back_if_dirty(index, way) {
            cycles += self.memory.latency;
        }

        let data = self.memory.read_line(self.memory.line_base(address));
        self.sets[index].lines[way].fill(tag, data, self.timestamp);
        (index, way, false, cycles)
    }

    /// Write a line back to memory if it is valid and dirty. Returns true if it did.
    fn write_back_if_dirty(&mut self, index: usize, way: usize) -> bool {
        let line = &self.sets[index].lines[way];
        if !(line.valid && line.dirty) {
            return false;
        }
        let base = self.line_address(line.tag, index);
        self.memory.write_line(base, &line.data);
        self.sets[index].lines[way].dirty = false;
        self.writebacks += 1;
        true
    }

    /// Rebuild a line's base address from its tag and set index.
    fn line_address(&self, tag: u64, index: usize) -> u64 {
        (tag << (self.offset_bits + self.index_bits)) | ((index as u64) << self.offset_bits)
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
    
    /// Flush specific address (for Flush+Reload attacks), like clflush:
    /// writes the line back if dirty, then invalidates it.
    /// Returns true if the line was cached.
    pub fn flush(&mut self, address: u64) -> bool {
        let (tag, index, _offset) = self.decompose_address(address);
        match self.sets[index].find(tag) {
            Some(way) => {
                self.write_back_if_dirty(index, way);
                self.sets[index].lines[way].invalidate();
                true
            }
            None => false,
        }
    }

    /// Write bytes into memory without timing (initial state, e.g. a secret).
    /// Any cached copies of the affected lines are updated too, so the
    /// cache never holds stale data.
    pub fn preload(&mut self, address: u64, bytes: &[u8]) {
        self.memory.load(address, bytes);
        for (i, &b) in bytes.iter().enumerate() {
            let a = address + i as u64;
            let (tag, index, offset) = self.decompose_address(a);
            if let Some(way) = self.sets[index].find(tag) {
                self.sets[index].lines[way].data[offset] = b;
            }
        }
    }

    /// Read one byte as the program would see it (cache first, then memory),
    /// without timing, statistics, or changing cache state.
    pub fn peek(&self, address: u64) -> u8 {
        let (tag, index, offset) = self.decompose_address(address);
        match self.sets[index].find(tag) {
            Some(way) => self.sets[index].lines[way].data[offset],
            None => self.memory.peek(address),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_config::ReplacementPolicy;
    use crate::memory::DEFAULT_MEMORY_LATENCY as MEM;

    // 4 sets x 2 ways x 16-byte lines. Addresses 0, 64, 128, ... all map to set 0.
    fn small() -> Cache {
        Cache::new(4, 2, 16)
    }
    const SET_STRIDE: u64 = 64;

    #[test]
    fn unwritten_memory_reads_zero_miss_then_hit() {
        let mut c = small();
        assert_eq!(c.read(0x1234), (0, MEM));
        assert_eq!(c.read(0x1234), (0, HIT_LATENCY));
        assert_eq!(c.memory.reads, 1);
    }

    #[test]
    fn write_is_cached_not_written_through() {
        let mut c = small();
        c.write(5, 42);
        assert_eq!(c.read(5), (42, HIT_LATENCY));
        assert_eq!(c.memory.peek(5), 0, "write-back: memory not updated yet");
        assert_eq!(c.peek(5), 42, "program-visible value comes from cache");
        assert_eq!(c.memory.writes, 0);
    }

    #[test]
    fn dirty_eviction_writes_back_and_value_survives() {
        let mut c = small();
        c.write(0, 42);
        c.access(SET_STRIDE);          // fills way 2 of set 0
        let (hit, cycles) = c.access(2 * SET_STRIDE); // evicts LRU = dirty line 0
        assert!(!hit);
        assert_eq!(cycles, 2 * MEM, "miss + write-back");
        assert_eq!(c.writebacks, 1);
        assert_eq!(c.memory.writes, 1);
        assert_eq!(c.memory.peek(0), 42);

        // Refetch evicts line 64, which is clean, so no second write-back.
        assert_eq!(c.read(0), (42, MEM));
        assert_eq!(c.writebacks, 1);
    }

    #[test]
    fn clean_eviction_costs_no_write_back() {
        let mut c = small();
        c.access(0);
        c.access(SET_STRIDE);
        assert_eq!(c.access(2 * SET_STRIDE), (false, MEM));
        assert_eq!(c.writebacks, 0);
    }

    #[test]
    fn write_allocate_preserves_rest_of_line() {
        let mut c = small();
        c.preload(0, &[1, 2, 3, 4]);
        assert_eq!(c.write(1, 99), MEM, "write miss fetches the line");
        assert_eq!(c.memory.reads, 1);
        c.flush(0);
        assert_eq!(
            [c.memory.peek(0), c.memory.peek(1), c.memory.peek(2), c.memory.peek(3)],
            [1, 99, 3, 4]
        );
    }

    #[test]
    fn flush_writes_back_dirty_line() {
        let mut c = small();
        c.write(7, 9);
        assert!(c.flush(7));
        assert_eq!(c.memory.peek(7), 9);
        assert_eq!(c.writebacks, 1);
        assert_eq!(c.read(7), (9, MEM), "flushed line misses but keeps its value");
    }

    #[test]
    fn flush_clean_or_absent_line() {
        let mut c = small();
        assert!(!c.flush(0), "not cached");
        c.access(0);
        assert!(c.flush(0));
        assert_eq!(c.writebacks, 0, "clean line needs no write-back");
        assert_eq!(c.access(0), (false, MEM));
    }

    #[test]
    fn preload_updates_cached_copy() {
        let mut c = small();
        c.read(0x20);
        c.preload(0x20, b"hi");
        assert_eq!(c.read(0x20), (b'h', HIT_LATENCY));
        assert_eq!(c.read(0x21), (b'i', HIT_LATENCY));
    }

    #[test]
    fn legacy_access_hit_miss_pattern_unchanged() {
        // 2-way LRU, all in set 0: 0 M, 64 M, 0 H, 128 M (evicts 64), 64 M (evicts 0)
        let mut c = small();
        let got: Vec<_> = [0, 64, 0, 128, 64].iter().map(|&a| c.access(a)).collect();
        assert_eq!(
            got,
            vec![(false, MEM), (false, MEM), (true, HIT_LATENCY), (false, MEM), (false, MEM)]
        );
        let s = c.stats();
        assert_eq!((s.total_hits, s.total_misses), (1, 4));
        assert_eq!(c.memory.writes, 0, "reads never cause write-backs");
    }

    #[test]
    fn line_address_round_trips() {
        let c = small();
        for addr in [0u64, 15, 16, 63, 64, 0x1234, 0xdead_beef] {
            let (tag, index, _) = c.decompose_address(addr);
            assert_eq!(c.line_address(tag, index), addr & !15);
        }
    }

    #[test]
    fn values_survive_eviction_in_every_cache_type() {
        let configs = [
            CacheConfig::direct_mapped(256, 16),
            CacheConfig::set_associative(256, 16, 4, ReplacementPolicy::FIFO),
            CacheConfig::fully_associative(256, 16, ReplacementPolicy::Random),
        ];
        for config in configs {
            let name = config.name.clone();
            let mut c = Cache::from_config(config);
            // Write far more lines than the cache holds, forcing evictions.
            for i in 0..64u64 {
                c.write(i * 16 + 3, i as u8);
            }
            assert!(c.writebacks > 0, "{name}: expected evictions");
            for i in 0..64u64 {
                assert_eq!(c.read(i * 16 + 3).0, i as u8, "{name}: line {i}");
            }
        }
    }
}
