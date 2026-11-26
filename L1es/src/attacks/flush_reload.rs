use crate::cache::Cache;

/// Shared lookup table (e.g. a shared library's data) mapped by both
/// victim and attacker. Entries are one cache line apart.
pub const SHARED_TABLE_ADDR: u64 = 0x10000;
pub const SHARED_TABLE_ENTRIES: usize = 5;
pub const ENTRY_STRIDE: u64 = 64;

/// Reads faster than this (in cycles) count as cache hits.
pub const HIT_THRESHOLD: u64 = 50;

/// Address of shared table entry `i`.
pub fn table_entry(i: usize) -> u64 {
    SHARED_TABLE_ADDR + i as u64 * ENTRY_STRIDE
}

/// Fill the shared table in simulated memory with recognisable values.
pub fn setup_shared_memory(cache: &mut Cache) {
    for i in 0..SHARED_TABLE_ENTRIES {
        cache.preload(table_entry(i), &[0xA0 + i as u8]);
    }
}

/// Victim reads a shared table entry - returns the value it read.
fn victim_lookup(cache: &mut Cache, i: usize) -> u8 {
    let (value, _) = cache.read(table_entry(i));
    println!("Victim: read table[{}] = 0x{:02x}", i, value);
    value
}

pub struct FlushReloadAttack {

    pub monitored_addresses: Vec<u64>,
    pub acess_times: Vec<u64>,
    pub detection_results: Vec<bool>,
}

impl FlushReloadAttack {

    // TODO: new(), flush(), reload(), get_summary(), sim_victim_scenario(), demo_flush_reload()

    pub fn new(addresses: Vec<u64>) -> Self {

        FlushReloadAttack {
            monitored_addresses: addresses,
            acess_times: Vec::new(),
            detection_results: Vec::new(),
        }
    }

    // remove monitored_addresses
    pub fn flush(&self, cache: &mut Cache) {

        for &addr in &self.monitored_addresses {
            cache.flush(addr);
        }
        println!("Flush: Removed: {} addresses from cache", self.monitored_addresses.len());
    }

    // access monitored addresses and measure timing
    pub fn reload(&mut self, cache: &mut Cache) -> Vec<bool> {

        self.acess_times.clear();
        self.detection_results.clear();

        println!("Reload: Checking which addresses were accessed by victim");

        for &addr in &self.monitored_addresses {
            
            // the attacker only sees how long the read took
            let (_value, time) = cache.read(addr);
            self.acess_times.push(time);

            // fast access is a hit and means that victim accessed the address 
            // slow access is a miss, victim did not access the address 

            let victim_accessed = time < HIT_THRESHOLD;
            self.detection_results.push(victim_accessed);

            println!("Address 0x{:x}: {} cycles -> {}", addr, time,
                if victim_accessed {"ACCESSED by the victim"} else {"NOT accessed by the victim"});

        }

        self.detection_results.clone()

    }


    // detection results
    pub fn get_summary(&self) -> (usize, usize) {
        let accessed_count = self.detection_results.iter().filter(|&&x| x).count();
        let total_count = self.detection_results.len();
        (accessed_count, total_count)
    }
}

pub fn sim_victim_scenario(cache: &mut Cache, scenario: &str) {

    match scenario {

        "none" => {
            println!("Victim: No memory access");
        },
        "pattern1" => {
            println!("Victim: Accessing secret data at specific addresses");
            victim_lookup(cache, 0); // Secret lookup table entry 1
            victim_lookup(cache, 1); // Secret lookup table entry 2
        },
        "pattern2" => {
            println!("Victim: Different access pattern");
            victim_lookup(cache, 2); // Different secret data
            victim_lookup(cache, 3);
        },
        "noisy" => {
            println!("Victim: Many random accesses (noisy)");
            for i in 0..10 {
                cache.read(0x20000 + i * 64); // Victim's private data
            }
            victim_lookup(cache, 0); // Hidden among noise
        },
        _=> println!("Unkown victim scenario"),
    }
}


pub fn demo_flush_reload() {

    println!("\n---- Flush+Reload Attack Demo ----");

    let mut cache = Cache::new(16,4,64);
    println!("Cache: {} sets, {}-way associative", cache.num_sets, cache.associativity);

    setup_shared_memory(&mut cache);
    println!("Shared table: {} entries at 0x{:x}", SHARED_TABLE_ENTRIES, SHARED_TABLE_ADDR);

    let monitored_addrs: Vec<u64> = (0..SHARED_TABLE_ENTRIES).map(table_entry).collect();

    println!("Monitoring {} specific addresses for victim access", monitored_addrs.len());

    let scenarios = vec![
        ("none", "No victim activity"),
        ("pattern1", "Victim accesses entries 0 and 1"),
        ("pattern2", "Victim accesses entries 2 and 3"), 
        ("noisy", "Victim access hidden in noise"),
    ];

    for (scenario, description) in scenarios {

        println!("\n ---- Scenario: {} ----", description);

        let mut attack = FlushReloadAttack::new(monitored_addrs.clone());

        attack.flush(&mut cache);

        sim_victim_scenario(&mut cache, scenario);

        let results = attack.reload(&mut cache);

        let (accessed, total) = attack.get_summary();
        println!("RESULT: {}/{} monitored addresses were accessed by victim", accessed, total);

        if accessed > 0 {
            println!("FLUSH+RELOAD momory pattern!");
            println!("Access pattern: {:?}", results);
            
        } else {
            println!("NO victim activity detected!");
        }
    }

    println!("\n---- Attack Comparison ----");
    println!("Prime+Probe: Detects cache SET usage (coarse-grained)");
    println!("Flush+Reload: Detects specific ADDRESS access (fine-grained)");
    println!("Flush+Reload requires shared memory but gives exact information!");

}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(scenario: &str) -> Vec<bool> {
        let mut cache = Cache::new(16, 4, 64);
        setup_shared_memory(&mut cache);
        let mut attack =
            FlushReloadAttack::new((0..SHARED_TABLE_ENTRIES).map(table_entry).collect());
        attack.flush(&mut cache);
        sim_victim_scenario(&mut cache, scenario);
        attack.reload(&mut cache)
    }

    #[test]
    fn detects_exactly_the_entries_the_victim_read() {
        assert_eq!(run("none"), vec![false; 5]);
        assert_eq!(run("pattern1"), vec![true, true, false, false, false]);
        assert_eq!(run("pattern2"), vec![false, false, true, true, false]);
        assert_eq!(run("noisy"), vec![true, false, false, false, false]);
    }

    #[test]
    fn victim_reads_shared_values_from_memory() {
        let mut cache = Cache::new(16, 4, 64);
        setup_shared_memory(&mut cache);
        assert_eq!(victim_lookup(&mut cache, 3), 0xA3);
    }
}
