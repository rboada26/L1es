// src/memory.rs

use vstd::prelude::*;

use std::collections::HashMap;

/// Default main-memory access latency in cycles (matches the old hard-coded miss cost).


verus! {
    #[cfg(verus_keep_ghost)]
    use vstd::arithmetic::power2::{pow2, lemma_pow2_pos, lemma_pow2_strictly_increases, lemma2_to64};
    #[cfg(verus_keep_ghost)]
    use vstd::bits::{low_bits_mask, lemma_u64_low_bits_mask_is_mod};
    #[cfg(verus_keep_ghost)]
    use vstd::arithmetic::div_mod::{lemma_mod_mod};
    #[cfg(verus_keep_ghost)]
    use vstd::std_specs::hash::EntrySpecFns;
    #[cfg(verus_keep_ghost)]
    use std::alloc::Allocator;

    pub const DEFAULT_MEMORY_LATENCY: u64 = 100;

    /// Sparse, line-granular backing memory.
    ///
    /// Only lines that have been written are stored; every other address reads as zero.
    /// All addresses passed to `read_line` / `write_line` must be line-aligned.
    pub struct Memory {
        pub lines: HashMap<u64, Vec<u8>>, // key = line-aligned base address
        pub line_size: usize,
        pub latency: u64,             // cycles per line transfer
        pub reads: u64,               // line reads (cache fills)
        pub writes: u64,              // line writes (write-backs)
    }




    pub open spec fn is_pow2(x: usize) -> bool {
        exists|k: nat| #[trigger] pow2(k) == x as int
    }

    pub assume_specification[ usize::is_power_of_two ](x: usize) -> (b: bool)
    ensures
        b <==> is_pow2(x);

    /// `Entry::or_insert_with`: like vstd's `or_insert`, but the default comes from calling `default`
    /// (only when the entry is vacant).
    pub assume_specification<'a, K, V, A: Allocator, F: FnOnce() -> V>[ std::collections::hash_map::Entry::<'a, K, V, A>::or_insert_with::<F> ](
        entry: std::collections::hash_map::Entry<'a, K, V, A>,
        default: F,
    ) -> (value: &'a mut V)
        requires
            entry.value() is None ==> default.requires(()),
        ensures
            entry.value() matches Some(v) ==> *value == v,
            entry.value() is None ==> default.ensures((), *value),
            entry.final_value() == Some(*final(value)),
    ;

    impl Memory {
        pub open spec fn line_view(&self, base: u64) -> Seq<u8> {
            if self.lines@.contains_key(base) {
                self.lines@[base]@
            } else {
                Seq::new(self.line_size as nat, |_i: int| 0u8)
            }
        }
    }

    impl View for Memory {
        // map over all line-aligned bases (infinite in principle, so IMap rather than Map)
        type V = IMap<u64, Seq<u8>>;
        open spec fn view(&self) -> IMap<u64, Seq<u8>> {
            IMap::new(
                |b: u64| b % (self.line_size as u64) == 0,
                |b: u64| self.line_view(b),
            )
        }
    }

    impl Memory {

        pub open spec fn valid(&self) -> bool {
            &&& is_pow2(self.line_size)
            &&& vstd::std_specs::hash::hash_map_inv(&self.lines) // needed for HashMap::get's spec
            &&& forall|a: u64| {
                #[trigger] self.lines@.contains_key(a) ==> self.lines@[a].len() == self.line_size
            }
        }

        
        pub fn with_latency(line_size: usize, latency: u64) -> (result: Self)
            requires is_pow2(line_size), 
            ensures result.valid() 
        {
            assert!(line_size.is_power_of_two(), "line size must be a power of two");
            Memory {
                lines: HashMap::new(),
                line_size,
                latency,
                reads: 0,
                writes: 0,
            }
        }

        pub fn new(line_size: usize) -> (result: Self)
            requires is_pow2(line_size),
            ensures result.valid()
        {
            Self::with_latency(line_size, DEFAULT_MEMORY_LATENCY)
        }



        /// Line-aligned base address containing `addr`.
        pub fn line_base(&self, addr: u64) -> (res: u64)
            requires self.valid(),
            ensures {
                &&& res % (self.line_size as u64) == 0
                &&& res == addr - addr % (self.line_size as u64)
            },
        {
            proof {
                let k = choose|k: nat| pow2(k) == self.line_size as int;
                lemma_pow2_pos(k);
                if k >= 64 {                            // pow2(k) would exceed usize::MAX
                    lemma2_to64();
                    if k > 64 { lemma_pow2_strictly_increases(64, k); }
                    assert(false);
                }
                let mask = (self.line_size - 1) as u64; 
                let res = addr & !mask;
                assert(low_bits_mask(k) == mask);       // low_bits_mask(k) == pow2(k) - 1 == mask holds from the choice of k 
                lemma_u64_low_bits_mask_is_mod(res, k); // res & mask == res & low_bits_mask(k) == res % pow2(k) == res % self.line_size
                lemma_u64_low_bits_mask_is_mod(addr, k); // addr & mask == addr % self.line_size
                assert(res & mask == 0  
                    && res + (addr & mask) == addr
                ) by (bit_vector) // res & mask == addr & !mask & mask == addr & 0 == 0
                    requires res == addr & !mask;

            }
            addr & !(self.line_size as u64 - 1)
        }

        proof fn read_write_facts(&self, base: u64) 
            requires {
                &&& self.valid()
                &&& base % (self.line_size as u64) == 0 
            },
            ensures {
                &&& base % (self.line_size as u64) == 0 
                &&& (base % (self.line_size as u64)) % (self.line_size as u64) == 0
                &&& self.line_size > 0 
            },
        {
            let n = self.line_size as u64;
            let k = choose|k: nat| pow2(k) == self.line_size as int;
            lemma_pow2_pos(k);
            lemma_mod_mod(base as int, n as int, 1 as int);

        }

        pub fn read_line(&mut self, base: u64) -> (res: Vec<u8>)
            requires {
                &&& self.valid() 
                &&& base % (self.line_size as u64) == 0 
                &&& self.reads < u64::MAX
            },
            ensures {
                &&& *final(self) == (Memory { reads: (old(self).reads + 1) as u64, ..*old(self) })
                &&& res@ == old(self)@[base]
            },
        {
            proof {
                self.read_write_facts(base);
            }
            debug_assert_eq!(base, self.line_base(base), "unaligned line read");
            self.reads += 1;
            match self.lines.get(&base) {
                Some(line) => {
                    // get's spec: line@ == self.lines@[base]; valid() then gives the length
                    assert(self.lines@.contains_key(base));
                    line.clone()
                }
                None => vec![0; self.line_size],
            }
        }

        /// Write a full line (counted as a memory access).
        pub fn write_line(&mut self, base: u64, data: &[u8]) 
            requires {
                &&& self.valid()
                &&& base % (self.line_size as u64) == 0 
                &&& self.writes < u64::MAX
                &&& data.len() == self.line_size
            },
            ensures {
                // only `writes` and `lines` change; `lines` is described via the view below
                &&& *final(self) == (Memory {
                        writes: (old(self).writes + 1) as u64,
                        lines: final(self).lines,
                        ..*old(self)
                    })
                &&& final(self).valid()
                &&& final(self)@ == old(self)@.insert(base, data@)
            },
        {
            proof {
                self.read_write_facts(base);
            }
            debug_assert_eq!(base, self.line_base(base), "unaligned line write");
            debug_assert_eq!(data.len(), self.line_size, "line data has wrong length");
            self.writes += 1;
            self.lines.insert(base, vstd::slice::slice_to_vec(data));
        }

        /// Abstract byte at `a`: offset `a % line_size` within the line containing `a`.
        pub open spec fn byte(&self, a: u64) -> u8 {
            let n = self.line_size as u64;
            self.line_view((a - a % n) as u64)[(a % n) as int]
        }

        /// Write bytes directly into memory, bypassing the cache and access counters.
        /// Intended for setting up initial state (e.g. a secret for Spectre).
        pub fn load(&mut self, addr: u64, bytes: &[u8])
            requires {
                &&& self.valid()
                &&& addr + bytes.len() <= u64::MAX
            },
            ensures {
                // only `lines` changes; counters and config are untouched
                &&& *final(self) == (Memory { lines: final(self).lines, ..*old(self) })
                &&& final(self).valid()
                // bytes in [addr, addr + len) are overwritten, everything else is unchanged
                &&& forall|a: u64| #[trigger] final(self).byte(a) ==
                        if addr <= a < addr + bytes.len() {
                            bytes@[a - addr]
                        } else {
                            old(self).byte(a)
                        }
            },
        {
            for (i, &b) in it: bytes.iter().enumerate()
                invariant
                    addr + bytes.len() <= u64::MAX,
                    self.valid(),
                    *self == (Memory { lines: self.lines, ..*old(self) }),
                    forall|a: u64| #[trigger] self.byte(a) ==
                        if addr <= a < addr + it.index() {
                            bytes@[a - addr]
                        } else {
                            old(self).byte(a)
                        },
            {
                let ghost prev = *self;
                assert(i == it.index() && b == bytes@[i as int]);
                let a = addr + i as u64;
                let base = self.line_base(a);
                proof { self.read_write_facts(base); }
                let line_size = self.line_size;
                let line = self.lines.entry(base).or_insert_with(
                    || -> (v: Vec<u8>) ensures v@ =~= Seq::new(line_size as nat, |_i: int| 0u8) {
                        vec![0; line_size]
                    },
                );
                assert(line@ =~= prev.line_view(base));
                line[(a - base) as usize] = b;
                proof {
                    let n = self.line_size as u64;
                    // only the line at `base` changed, and only at offset `a - base`
                    assert forall|a2: u64| #[trigger] self.byte(a2) ==
                        (if a2 == a { b } else { prev.byte(a2) })
                    by {
                        assert(a2 % n <= a2) by (nonlinear_arith) requires n > 0;
                        if (a2 - a2 % n) as u64 == base && a2 % n == a - base {
                            assert(a2 == a);
                        }
                    }
                }
            }
        }


        /// Read one byte without timing or counters (debugging/tests).
        /// Note: this sees memory only, not newer dirty data still in the cache.
        pub fn peek(&self, addr: u64) -> (res: u8)
            requires self.valid(),
            ensures 
                res == self@[(addr - addr % (self.line_size as u64)) as u64][(addr % (self.line_size as u64)) as int]
        {
            let base = self.line_base(addr);
            proof { self.read_write_facts(base); }
            self.lines
                .get(&base)
                .map(|line: &Vec<u8>| -> (r: u8)
                    requires base <= addr, (addr - base) < line.len()
                    ensures r == line@[(addr - base) as int]
                {
                    line[(addr - base) as usize]
                })
                .unwrap_or(0)
        }
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


