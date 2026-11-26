use crate::cache::Cache;

pub struct PrimeProbeAttack {

    pub target_set: usize,
    pub eviction_set: Vec<u64>,
    pub probe_times: Vec<u64>,
}

impl PrimeProbeAttack {

// TODO new(), gen_eviction_set(), prime(), probe(), sim_victim access(), demo_prime_probe()


    pub fn new(cache: &Cache, target_set: usize) -> Self {

        let eviction_set = Self::generate_eviction_set(cache, target_set);

        PrimeProbeAttack {

            target_set,
            eviction_set,
            probe_times: Vec::new(),
        }
    }

    fn generate_eviction_set(cache: &Cache, target_set: usize) -> Vec<u64> {

        let mut eviction_set = Vec::new();
        let set_stride = 1 << (cache.offset_bits + cache.index_bits);

        // exactly one address per way: more would evict our own lines and
        // look like victim activity even when the victim is idle
        for i in 0..cache.associativity {

             // Address formula: (i * set_stride) + (target_set << offset_bits)

            let addr = (i as u64 * set_stride as u64) + 
                ((target_set as u64) << cache.offset_bits);

            eviction_set.push(addr);
        }
        eviction_set
    }


    // fill with attacker data 
    pub fn prime(&self, cache: &mut Cache){

        for &addr in &self.eviction_set {
            cache.read(addr);
        }
    }

    //  Measure access times to detect victim activity
    pub fn probe(&mut self, cache: &mut Cache) -> bool {

        let mut total_time = 0;
        self.probe_times.clear();

        for &addr in &self.eviction_set {

            let (_value, time) = cache.read(addr);
            self.probe_times.push(time);
            total_time += time;
        }

        // if average is high then victim as likely acessed the set
        let avrg_time = total_time as f64 / self.eviction_set.len() as f64;
        let threshold = 50.0; // Between hit (1 cycle) and miss (100 cycles)

        avrg_time > threshold
    }

    // simulate victim accessing memory that maps to the target set
    pub fn simulate_victim_access(cache: &mut Cache, target_set: usize) {

        let victim_addr = ((target_set as u64) << cache.offset_bits) + 0x12345000;

        let (value, _) = cache.read(victim_addr);

        println!("Victim read 0x{:02x} from address 0x{:x} (maps to set {})",
            value, victim_addr, target_set);
    }
}

// demo of prime + probe attack

pub fn demo_prime_probe(){

    println!("\n=== Prime+Probe Attack Demo ===");
    
    // Create cache: 8 sets, 4-way associative, 64-byte lines
    let mut cache = Cache::new(8, 4, 64);
    let target_set = 3; // Monitor cache set 3

    // victim's private data, in simulated memory
    for set in 0..cache.num_sets {
        let addr = ((set as u64) << cache.offset_bits) + 0x12345000;
        cache.preload(addr, &[0x50 + set as u8]);
    }
    
    println!("Cache: {} sets, {}-way associative", cache.num_sets, cache.associativity);
    println!("Monitoring cache set {}", target_set);
    
    // Create attack
    let mut attack = PrimeProbeAttack::new(&cache, target_set);
    println!("Generated eviction set with {} addresses", attack.eviction_set.len());
    
    // Test 1: No victim activity
    println!("\n--- Test 1: No Victim Activity ---");
    attack.prime(&mut cache);
    println!("PRIME: Filled cache set {} with attacker data", target_set);
    
    let detected = attack.probe(&mut cache);
    println!("PROBE: Victim activity detected: {}", detected);
    println!("Probe times: {:?}", attack.probe_times);
    
    // Test 2: Victim accesses different cache set
    println!("\n--- Test 2: Victim Uses Different Set ---");
    attack.prime(&mut cache);
    PrimeProbeAttack::simulate_victim_access(&mut cache, 5); // Different set
    
    let detected = attack.probe(&mut cache);
    println!("PROBE: Victim activity detected: {}", detected);
    
    // Test 3: Victim accesses our monitored set
    println!("\n--- Test 3: Victim Uses Monitored Set ---");
    attack.prime(&mut cache);
    PrimeProbeAttack::simulate_victim_access(&mut cache, target_set); // Same set!
    
    let detected = attack.probe(&mut cache);
    println!("PROBE: Victim activity detected: {}", detected);
    println!("Probe times: {:?}", attack.probe_times);
    
    if detected {
        println!("SUCCESS: Prime+Probe detected victim memory access!");
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn detects(victim_set: Option<usize>) -> bool {
        let mut cache = Cache::new(8, 4, 64);
        let mut attack = PrimeProbeAttack::new(&cache, 3);
        attack.prime(&mut cache);
        if let Some(set) = victim_set {
            PrimeProbeAttack::simulate_victim_access(&mut cache, set);
        }
        attack.probe(&mut cache)
    }

    #[test]
    fn idle_victim_is_not_detected() {
        assert!(!detects(None));
    }

    #[test]
    fn victim_in_other_set_is_not_detected() {
        assert!(!detects(Some(5)));
    }

    #[test]
    fn victim_in_monitored_set_is_detected() {
        assert!(detects(Some(3)));
    }
}
