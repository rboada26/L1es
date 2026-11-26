// src/memory.rs

use std::collections::HashMap;

/// Default main-memory access latency in cycles (matches the old hard-coded miss cost).
pub const DEFAULT_MEMORY_LATENCY: u64 = 100;

/// Sparse, line-granular backing memory.
///
/// Only lines that have been written are stored; every other address reads as zero.
/// All addresses passed to `read_line` / `write_line` must be line-aligned.
pub struct Memory {
    lines: HashMap<u64, Vec<u8>>, // key = line-aligned base address
    line_size: usize,
    pub latency: u64,             // cycles per line transfer
    pub reads: u64,               // line reads (cache fills)
    pub writes: u64,              // line writes (write-backs)
}

impl Memory {
    pub fn new(line_size: usize) -> Self {
        Self::with_latency(line_size, DEFAULT_MEMORY_LATENCY)
    }

    pub fn with_latency(line_size: usize, latency: u64) -> Self {
        assert!(line_size.is_power_of_two(), "line size must be a power of two");
        Memory {
            lines: HashMap::new(),
            line_size,
            latency,
            reads: 0,
            writes: 0,
        }
    }

    /// Line-aligned base address containing `addr`.
    pub fn line_base(&self, addr: u64) -> u64 {
        addr & !(self.line_size as u64 - 1)
    }

    /// Read a full line (counted as a memory access).
    pub fn read_line(&mut self, base: u64) -> Vec<u8> {
        debug_assert_eq!(base, self.line_base(base), "unaligned line read");
        self.reads += 1;
        self.lines
            .get(&base)
            .cloned()
            .unwrap_or_else(|| vec![0; self.line_size])
    }

    /// Write a full line (counted as a memory access).
    pub fn write_line(&mut self, base: u64, data: &[u8]) {
        debug_assert_eq!(base, self.line_base(base), "unaligned line write");
        debug_assert_eq!(data.len(), self.line_size, "line data has wrong length");
        self.writes += 1;
        self.lines.insert(base, data.to_vec());
    }

    /// Write bytes directly into memory, bypassing the cache and access counters.
    /// Intended for setting up initial state (e.g. a secret for Spectre).
    pub fn load(&mut self, addr: u64, bytes: &[u8]) {
        for (i, &b) in bytes.iter().enumerate() {
            let a = addr + i as u64;
            let base = self.line_base(a);
            let line_size = self.line_size;
            let line = self.lines.entry(base).or_insert_with(|| vec![0; line_size]);
            line[(a - base) as usize] = b;
        }
    }

    /// Read one byte without timing or counters (debugging/tests).
    /// Note: this sees memory only, not newer dirty data still in the cache.
    pub fn peek(&self, addr: u64) -> u8 {
        let base = self.line_base(addr);
        self.lines
            .get(&base)
            .map_or(0, |line| line[(addr - base) as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwritten_memory_reads_zero() {
        let mut m = Memory::new(64);
        assert_eq!(m.read_line(0x1000), vec![0; 64]);
        assert_eq!(m.peek(0xdead_beef), 0);
        assert_eq!(m.reads, 1);
    }

    #[test]
    fn load_spans_lines_and_peek_reads_back() {
        let mut m = Memory::new(64);
        m.load(60, b"ABCDEFGH"); // crosses the 64-byte boundary
        assert_eq!(m.peek(60), b'A');
        assert_eq!(m.peek(63), b'D');
        assert_eq!(m.peek(64), b'E');
        assert_eq!(m.peek(67), b'H');
        assert_eq!(m.writes, 0, "load must not count as a timed access");
    }

    #[test]
    fn write_line_then_read_line() {
        let mut m = Memory::new(16);
        let data: Vec<u8> = (0..16).collect();
        m.write_line(0x40, &data);
        assert_eq!(m.read_line(0x40), data);
        assert_eq!((m.reads, m.writes), (1, 1));
    }
}
