use crate::set::CacheSet;

pub struct Cache {

    pub sets: Vec<CacheSet>,
    pub num_sets: usize,     // count of CacheSets
    pub associativity: usize,  // ways per set
    pub line_size: usize,       //   bytes per line
    pub index_bits: usize,     // bits for the set index 
    pub offset_bits: usize,    // bits for byte offset within the given line
    pub timestamp: u64,     // timestamp for the LRU 

}


impl Cache {

    pub fn new(num_sets: usize, associativity: usize, line_size: usize) -> Self {

        let mut sets = Vec::with_capacity(num_sets);

        for _ in 0..num_sets {
            sets.push(CacheSet::new(associativity, line_size));
        }
    }


    let index_bits = (num_sets as f64).log2() as usize;
    let offset_bits = (line_size as f64).log2() as usize;

    Cache {

        sets,
        num_sets,
        associativity,
        line_size,
        index_bits,
        offset_bits,
        timestamp: 0,
    }



    // access the cache at a given address, returns the hit and the address time cycle
    pub fn access(&mut self, address: u64) -> (bool, u64) {

        self.timestamp += 1;

        let (tag, index, offset) = self.decompose_address(address);
        let (hit, _evicted) = self.sets[index].access(tag, self.timestamp);

        let access_time = if hit { 1 } else { 100 };

        (hit, access_time)
    }

    // decompose the address into (tag, index, offset)
    fn decompose_address(&self, address: u64) -> (u64, usize, usize) {

        let offset = (address & ((1 << self.offset_bits) -1)) as usize;
        
        let index = ((address >> self.offset_bits) & ((1 << self.index_bits) -1)) as usize;

        let tag = address >> (self.offset_bits + self.index_bits);

        (tag, index, offset)

    }

    // flush a specific address for flush reload attacks
    pub fn flush(&mut self, address: u64) -> bool {

        let (tag, index, _offset) = self.decompose_address(address);
        self.sets[index].flush(tag)
    }

    // overall cache stats
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
            hit_rate: if total_accesses == 0 { 0.0 } else {total_hits as f64 / total_accesses as f64},
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {

    pub total_accesses: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
}
