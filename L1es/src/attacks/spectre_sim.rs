use crate::cache::Cache;

pub struct SpectreSimulator {
    pub cache: Cache,
    pub secret_data: Vec<u8>,
    pub probe_array: Vec<u64>, // for cache convert channel 
    pub branch_predictor: BranchPredictor,
}

pub struct BranchPredictor {
    pub history: Vec<bool>, // recent branch outcomes
    pub confidence: f64,  // prediction confidence
}

impl BranchPredictor {
    pub fn new() -> Self {

        BranchPredictor {
            history: vec![true; 10], // initially biased to "taken"
            confidence: 0.9
        }
    }
    pub fn train(&mut self, outcome: bool) {

        self.history.push(outcome);
        if self.history.len() > 10 {

            self.history.remove(10);
        }

        // update confidence based on recent accuracy
        let recent_correct = self.history.iter().filter(|&&x| x == outcome).count();
        self.confidence = recent_correct as f64 / self.history.len() as f64;

    }

    pub fn predict(&self) -> (bool, f64) {
        let prediction = self.history.iter().filter(|&&x| x).count() > self.history.len() / 2;
        (prediction, self.confidence)
    }
}

impl SpectreSimulator {

    pub fn new() -> Self {

        let cache = Cache::new(64,8,64);

        let secret_data = b"SECRET_PASS_123!".to_vec();

        let mut probe_array = Vec::new();
        for i in 0..256 {
            probe_array.push(0x100000 + (i as u64 * 4096));
        }
        
        SpectreSimulator {
            cache,
            secret_data,
            probe_array,
            branch_predictor: BranchPredictor::new(),
        }
    }

    fn bounds_check(&self, index: usize) -> bool {
        index < self.secret_data.len()
    }


    // sim victim function vulnerable to Spectre attack 
    // checks the bounds but can be speculatively bypassed
    pub fn victim_function(&mut self, user_index: usize) -> Option<u8> {

        println!("Victim function called with index: {}", user_index);
        
        let (predicted_in_bounds, confidence) = self.branch_predictor.predict();
        println!("Branch predictor : {} with confidence: {:.2}",
            if predicted_in_bounds {"IN BOUNDS"} else {"OUT OF BOUNDS"}, confidence);

        if confidence > 0.7 && predicted_in_bounds {

            println!("SPECULATIVE EXECUTION: Proceeding with memory access...");

            // speculatively access secret data 
            let secret_byte = if user_index < self.secret_data.len() {
                self.secret_data[user_index]
            } else {0xFF}; // simulated out of bounds kernel data

            let probe_addr = self.probe_array[secret_byte as usize];
            self.cache.access(probe_addr);
            println!("SIDE EFFECT: Cache probe_array[{}] at address 0x{:x}",
                secret_byte, probe_addr);

        }

        // actual bounds check, architectural verification
        let actual_in_bounds = self.bounds_check(user_index);
        self.branch_predictor.train(actual_in_bounds);

        if actual_in_bounds {
            println!("ARCHITECTURAL: Access allowed, returning data");

            Some(self.secret_data[user_index])

        } else {
            println!("ARCHITECTURAL: Access denied, speculation effects rolled back");
            println!("(however side effects remain)");

            None
        }
    }

    // attacker probes cache to extract speculative side efeects 
    pub fn extract_secret_byte(&mut self) -> Option<u8> {

        println!("ATTACKER: Probing cache to extract secret...");
        
        // Simple approach: check the last few cached addresses
        for (byte_val, &addr) in self.probe_array.iter().enumerate().take(256) {
            let (hit, time) = self.cache.access(addr);
   /*         
            if byte_val == 69 || byte_val == 83 || byte_val == 67 || byte_val == 82 || byte_val == 255 {
                println!("  Checking critical byte {}: hit={}, time={}", byte_val, hit, time);
            }
*/             
            // Look for recently cached values (either hit=true or time=1)
            if hit || time == 1 {
                println!("ATTACKER: Found cached value! Secret byte = 0x{:02x} ('{}')", 
                         byte_val, if byte_val <= 127 { byte_val as u8 as char } else { '?' });
                return Some(byte_val as u8);
            }
        }
        
        println!("ATTACKER: No secret extracted - trying alternative approach");
        
        
        None
    }
            
}

pub fn demo_spectre_attack() {

    println!("\n---- Spectre Attack Simulation ----");
    println!("Simulating speculative execution vulnerability...\n");

    let mut spectre = SpectreSimulator::new();

    println!("Secret data: {:?}", String::from_utf8_lossy(&spectre.secret_data));
    println!("Probe array: {} entries, 4KB apart\n", spectre.probe_array.len());

     println!("--- Phase 1: Training Branch Predictor ---");
    for i in 0..10 {
        println!("Training iteration {}: bounds check with valid index", i);
        spectre.victim_function(i % 5); // Valid indices to train predictor
    }


    println!("\n--- Phase 2: Spectre Attack ---");
    println!("Attempting to read secret data at out-of-bounds index...");

    // Try to read secret at index that should be out of bounds
    let attack_index = spectre.secret_data.len() + 5; // Out of bounds
    let result = spectre.victim_function(attack_index);

    println!("Architectural result: {:?}", result);

    println!("\n--- Phase 3: Cache Side Channel Extraction ---");
    if let Some(leaked_byte) = spectre.extract_secret_byte() {
        println!(" SUCCESS: Spectre attack leaked secret data!");
        println!("   Leaked byte: 0x{:02x} ('{}')", leaked_byte, leaked_byte as char);
    } else {
        println!(" Attack failed - no secret data extracted");
    }

    println!("\n--- Phase 4: Multi-byte Extraction ---");
    println!("Extracting multiple secret bytes...");
 
    for target_offset in 0..5 {
        println!("\nTargeting secret[{}]:", target_offset);

        // Re-train predictor
        for _ in 0..5 {
            spectre.victim_function(target_offset);
        }

        // Attack with out-of-bounds access
        let oob_index = spectre.secret_data.len() + target_offset;
        spectre.victim_function(oob_index);

        // Extract
        if let Some(byte) = spectre.extract_secret_byte() {
            println!("   Extracted: 0x{:02x} ('{}')", byte, byte as char);
        }
    }

}
