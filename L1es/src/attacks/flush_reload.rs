use crate::cache::Cache;

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
            
            let (hit, time) = cache.access(addr);
            self.acess_times.push(time);

            // fast access is a hit and means that victim accessed the address 
            // slow access is a miss, victim did not access the address 

            let victim_accessed = hit && time < 50;
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
            cache.access(0x10000); // Secret lookup table entry 1
            cache.access(0x10040); // Secret lookup table entry 2
        },
        "pattern2" => {
            println!("Victim: Different access pattern");
            cache.access(0x10080); // Different secret data
            cache.access(0x100C0);
        },
        "noisy" => {
            println!("Victim: Many random accesses (noisy)");
            for i in 0..10 {
                cache.access(0x20000 + i * 64); // Random addresses
            }
            cache.access(0x10000); // Hidden among noise
        },
        _=> println!("Unkown victim scenario"),
    }
}


pub fn demo_flush_reload() {

    println!("\n---- Flush+Reload Attack Demo ----");

    let mut cache = Cache::new(16,4,64);
    println!("Cache: {} sets, {}-way associative", cache.num_sets, cache.associativity);

    let monitored_addrs = vec![
        0x10000, 
        0x10040,  
        0x10080, 
        0x100C0, 
        0x10100,
    ];

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
