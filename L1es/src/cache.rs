use crate::set::CacheSet;

pub struct Cache {

    pub sets: Vec<CacheSet>,
    pub num_sets: usize,     // count of CacheSets
    pub associativity: usize,  // ways per set
    pub line_size: usize,       //   bytes per line
    pub index_bits: usize,     // bits for the set index 
    pub offset_bits: usize,    // bits for byte offeset within the given line
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
}

// TODO: implement: access, decompose_address, stats, and debug for Cache stats
