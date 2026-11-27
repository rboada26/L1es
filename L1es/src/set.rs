// src/set.rs

use vstd::prelude::*;
use crate::line::CacheLine;
use crate::cache_config::ReplacementPolicy;

verus! {

pub struct CacheSet {
    pub lines: Vec<CacheLine>,
    pub associativity: usize,        // Number of lines in this set (e.g., 8-way)
    pub access_count: u64,           // Total accesses to this set
    pub hit_count: u64,              // Cache hits in this set
    pub miss_count: u64,             // Cache misses in this set
    pub replacement_policy: ReplacementPolicy, // How to choose victim for replacement
    pub fifo_index: usize,           // For FIFO replacement
    pub line_size: Ghost<usize>,     // spec-only: every line's data has this length
}

impl CacheSet {

    pub open spec fn valid(&self) -> bool{
        forall |i: int|  
            0 <= i < self.lines.len()
            ==> #[trigger] self.lines[i].data.len() == self.line_size@
    }

    /// Create a new cache set with given associativity and line size
    pub fn new(associativity: usize, line_size: usize, policy: ReplacementPolicy) -> (res: Self) 
        ensures {
            &&& res == CacheSet {
                associativity: associativity,
                access_count: 0,
                hit_count: 0,
                miss_count: 0,
                replacement_policy: policy,
                fifo_index: 0,
                lines: res.lines,
                line_size: Ghost(line_size),
            }  
            &&& forall|i: int| 0 <= i < res.lines.len() ==> #[trigger] res.lines@[i].new_cache_line(line_size)
            &&& res.lines.len() == associativity as int
            &&& res.valid()
        },    
    {
        let mut lines: Vec<CacheLine> = Vec::with_capacity(associativity);
        for k in 0..associativity
            invariant
                k <= associativity,
                lines.len() == k,  
                forall|i: int| 0 <= i < k ==> #[trigger] lines@[i].new_cache_line(line_size),
        {
            lines.push(CacheLine::new(line_size));
        }
        
        let res = CacheSet {
            lines,
            associativity,
            access_count: 0,
            hit_count: 0,
            miss_count: 0,
            replacement_policy: policy,
            fifo_index: 0,
            line_size: Ghost(line_size),
        };
        proof {
            assert forall|i: int| 0 <= i < res.lines.len()
                implies #[trigger] res.lines[i].data.len() == res.line_size@
            by {
                // fire the loop invariant's quantifier at i: new lines have data.len() == line_size
                assert(res.lines@[i].new_cache_line(line_size));
            }
        }
        res
        
    }

    /// Find the way holding `tag` without affecting statistics or LRU state.
    pub fn find(&self, tag: u64) -> (res: Option<usize>)
        ensures {
            match res {
                Some(i) => i < self.lines.len() && self.lines@[i as int].is_hit(tag),
                None => forall |i: int| 0 <= i < self.lines.len() ==> ! #[trigger] self.lines@[i as int].is_hit(tag)
            }
        },
    {
        self.lines.iter().position(|line: &CacheLine| -> (b: bool)
            ensures b == line.is_hit(tag)   
        {
            line.is_hit(tag)
        })
    }

    /// Look up `tag` as a counted access. Returns Some(way) on a hit (and
    /// touches the line), or None on a miss. On a miss the caller must pick a
    /// victim with `choose_victim` and fill it.
    pub fn lookup(&mut self, tag: u64, timestamp: u64) -> (res: Option<usize>)
        requires {
            &&& old(self).valid()
            &&& old(self).access_count < u64::MAX
            &&& old(self).hit_count < u64::MAX
            &&& old(self).miss_count < u64::MAX
            // `touch` bumps the hit line's own counter
            &&& forall|i: int| 0 <= i < old(self).lines.len() ==>
                    #[trigger] old(self).lines@[i].access_count < u64::MAX
        },
        ensures {
            &&& final(self).valid()
            &&& match res {
                // hit: way `w` held the tag; only that line is touched, hit_count goes up
                Some(w) => {
                    &&& w < old(self).lines.len()
                    &&& old(self).lines@[w as int].is_hit(tag)
                    &&& final(self).lines@ == old(self).lines@.update(w as int, CacheLine {
                            last_access: timestamp,
                            access_count: (old(self).lines@[w as int].access_count + 1) as u64,
                            ..old(self).lines@[w as int]
                        })
                    &&& *final(self) == (CacheSet {
                            access_count: (old(self).access_count + 1) as u64,
                            hit_count: (old(self).hit_count + 1) as u64,
                            lines: final(self).lines,
                            ..*old(self)
                        })
                },
                // miss: no line held the tag; lines untouched, miss_count goes up
                None => {
                    &&& forall|i: int| 0 <= i < old(self).lines.len() ==>
                            !#[trigger] old(self).lines@[i].is_hit(tag)
                    &&& *final(self) == (CacheSet {
                            access_count: (old(self).access_count + 1) as u64,
                            miss_count: (old(self).miss_count + 1) as u64,
                            ..*old(self)
                        })
                },
            }
        },
    {
        self.access_count += 1;

        if let Some(way) = self.find(tag) {
            self.lines[way].touch(timestamp);
            self.hit_count += 1;
            return Some(way);
        }

        self.miss_count += 1;
        None
    }

}

}

impl CacheSet {
    
    /// Choose the way to fill on a miss: an invalid line if there is one,
    /// otherwise a victim chosen by the replacement policy. Does not modify
    /// the victim; the caller writes it back (if dirty) and refills it.
    pub fn choose_victim(&mut self) -> usize {
        if let Some(way) = self.lines.iter().position(|line| !line.valid) {
            return way;
        }

        match self.replacement_policy {
            ReplacementPolicy::LRU => self.find_lru_victim(),
            ReplacementPolicy::FIFO => self.find_fifo_victim(),
            ReplacementPolicy::Random => self.find_random_victim(),
        }
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
