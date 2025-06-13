use crate::line::CacheLine;

pub struct CacheSet {

    pub lines: Vec<CacheLine>,
    pub associativity: usize,
    pub access_count: u64,
    pub hit_count: u64,
    pub miss_count: u64,
}

impl CacheSet{


    pub fn new(associativity: usize, line_size: usize) -> Self {

        let mut lines = Vec::with_capacity(associativity);

        for _ in 0..associativity {

            lines.push(CacheLine::new(line_size));
        }

        CacheSet{

            lines,
            associativity,
            access_count,
            hit_count: 0,
            miss_count: 0,
        }
    }


    // acess set with tag, returns (hit, evicted_tag)
    // hit is true if cache hit, false otherwise
    // evicted_tag = Some(tag) if a line was evicted, None otherwise
    pub fn access(&mut self, tag: u64, timestamp: u64) -> (bool, Option<u64>){


    }

    //replace lien by doing LRU policy, returns evicted tag if any 
    fn replace_line(&mut self, tag: u64, timestamp: u64) -> Option<u64> {


    }

    // flush a specific tag from this set, this is for flush+reload attacks
    pub fn flush(&mut self, tag: u64) -> bool {


    }

    // get the hit rate statistic 
    pub hit_rate(&self) -> f64{

        if self.access_count ==0 {
            0.0
        } else{
            self.hit_count as f64 / self.access_count as f64
        }

    }

    pub fn isFull(&self) -> bool {
        self.lines.iter().filter(|line| line.valid).count()
    }

    pub fn valid_lines(&self) -> usize {
        self.lines.iter().filter(|line| line.valid).count()
    }
}
