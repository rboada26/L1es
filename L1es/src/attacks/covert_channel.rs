// src/attacks/covert_channel.rs
//
// Flush+Reload covert channel shared by the Spectre and Meltdown simulations.
// Transient code encodes a byte by touching probe[byte]; the attacker later
// times a read of every probe line and decodes the byte from the fast one.

use crate::cache::Cache;

/// One probe line per page, as in the original attacks.
pub const PROBE_STRIDE: u64 = 4096;

/// Reads faster than this (in cycles) count as cache hits.
pub const HIT_THRESHOLD: u64 = 50;

pub struct ProbeArray {
    pub base: u64,
}

impl ProbeArray {
    pub fn new(base: u64) -> Self {
        ProbeArray { base }
    }

    pub fn addr(&self, byte: u8) -> u64 {
        self.base + byte as u64 * PROBE_STRIDE
    }

    /// Evict every probe line so only the next encode leaves a trace.
    pub fn flush_all(&self, cache: &mut Cache) {
        for b in 0..=255u8 {
            cache.flush(self.addr(b));
        }
    }

    /// Transient side effect: bring probe[byte] into the cache.
    pub fn encode(&self, cache: &mut Cache, byte: u8) {
        cache.read(self.addr(byte));
    }

    /// Time a read of every probe line and return the bytes whose line was
    /// cached. Uses timing only, as a real attacker would.
    ///
    /// Each line is flushed again right after it is timed. With a 4 KiB stride
    /// every probe line maps to the same cache set, so without this the misses
    /// from the scan would evict the line carrying the signal before the scan
    /// reached it.
    pub fn reload(&self, cache: &mut Cache) -> Vec<u8> {
        let mut hits = Vec::new();
        for b in 0..=255u8 {
            let addr = self.addr(b);
            let (_value, cycles) = cache.read(addr);
            cache.flush(addr);
            if cycles < HIT_THRESHOLD {
                hits.push(b);
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_byte_value_round_trips() {
        let mut cache = Cache::new(64, 8, 64);
        let probe = ProbeArray::new(0x100000);
        for b in 0..=255u8 {
            probe.flush_all(&mut cache);
            probe.encode(&mut cache, b);
            assert_eq!(probe.reload(&mut cache), vec![b], "byte {b}");
        }
    }

    #[test]
    fn no_encode_means_no_hits() {
        let mut cache = Cache::new(64, 8, 64);
        let probe = ProbeArray::new(0x100000);
        probe.flush_all(&mut cache);
        assert!(probe.reload(&mut cache).is_empty());
    }
}
