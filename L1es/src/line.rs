// simulate cache lines

pub struct CacheLine {

    pub valid: bool, // is the line valid 
    pub tag: u64, // Tag bits for address matching
    pub data: Vec<u8>, // cached data
    pub last_access: u64, // LRU replacement policy
    pub access_count: u64, // only for statistics and debug
}

impl CacheLine {

    //create new empty cache
    pub fn new(line_size: usize) -> Self {


    }

    // check if line contains current tag
    pub fn is_hit(&self, tag: u64) -> bool {


    }

    // if the cache missed update the line with new data
    pub fn update(&mut self, tag: u64, timestamp: u64) {


    }

    // invalidates the line for flushing 
    pub fn invalidate(&mut self) {



    }
}
