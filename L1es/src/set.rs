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

        self.access_count += 1;

        for line in &mut self.lines {

            if line.is_hit(tag) {

                line.update(tag, timestamp);
                self.hit_count += 1;
                return (true, None);
            }
        }
        self.miss_count += 1;
        let evicted_tag = self.replace_line(tag, timestamp);
        (false, evicted_tag)

    }

    //replace lien by doing LRU policy, returns evicted tag if any 
    fn replace_line(&mut self, tag: u64, timestamp: u64) -> Option<u64> {

        for line in &mut self.lines {

            if !line.valid {
                line.update(tag, timestamp);
                return None;
            }
        }

        // All the lines are valid:

        let mut lru_idx = 0;

        let mut oldest_time = self.lines[0].last_access;

        for (i, line) in self.lines.iter().enumerate() {

            if line.last_access < oldest_time {

                oldest_time = line.last_access;
                lru_idx = i;
            }
        }

        let evicted_tag = self.lines[lru_idx].tag;

        self.lines[lru_idx].update(tag, timestamp);
        Some(evicted_tag)

    }

    // flush a specific tag from this set, this is for flush+reload attacks
    pub fn flush(&mut self, tag: u64) -> bool {

        for line in &mut self.lines {

            if line.is_hit(tag) {
                line.invalidate();
                return true;
            }
        }
        false

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
