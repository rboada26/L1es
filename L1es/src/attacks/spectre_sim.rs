use crate::cache::Cache;
use crate::attacks::covert_channel::ProbeArray;

/// Victim's public array: the bounds check only allows indices into this.
pub const ARRAY1_ADDR: u64 = 0x1000;
pub const ARRAY1_LEN: usize = 16;

/// Secret stored elsewhere in the victim's address space.
pub const SECRET_ADDR: u64 = 0x8000;

/// Probe array for the cache covert channel.
pub const PROBE_BASE: u64 = 0x100000;

/// In-bounds calls used to train the branch predictor before each attack.
const TRAINING_ROUNDS: usize = 5;

pub struct SpectreSimulator {
    pub cache: Cache,           // cache backed by simulated memory
    pub secret_len: usize,
    pub probe: ProbeArray,      // for cache covert channel
    pub branch_predictor: BranchPredictor,
    pub verbose: bool,          // print every step of the victim function
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

            self.history.remove(0); // drop the oldest outcome
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

    /// Lay out the victim's memory: public array1 contents, then the secret.
    pub fn new(secret: &[u8]) -> Self {

        let mut cache = Cache::new(64, 8, 64);

        let public: Vec<u8> = (1..=ARRAY1_LEN as u8).collect();
        cache.preload(ARRAY1_ADDR, &public);
        cache.preload(SECRET_ADDR, secret);

        SpectreSimulator {
            cache,
            secret_len: secret.len(),
            probe: ProbeArray::new(PROBE_BASE),
            branch_predictor: BranchPredictor::new(),
            verbose: true,
        }
    }

    fn bounds_check(&self, index: usize) -> bool {
        index < ARRAY1_LEN
    }

    /// Out-of-bounds index into array1 that points at secret[offset].
    pub fn malicious_index(offset: usize) -> usize {
        (SECRET_ADDR - ARRAY1_ADDR) as usize + offset
    }

    // sim victim function vulnerable to Spectre attack:
    //     if (x < array1_size) y = probe[array1[x] * 4096];
    // checks the bounds but can be speculatively bypassed
    pub fn victim_function(&mut self, user_index: usize) -> Option<u8> {

        let verbose = self.verbose;
        if verbose { println!("Victim function called with index: {}", user_index); }

        let (predicted_in_bounds, confidence) = self.branch_predictor.predict();
        if verbose {
            println!("Branch predictor : {} with confidence: {:.2}",
                if predicted_in_bounds {"IN BOUNDS"} else {"OUT OF BOUNDS"}, confidence);
        }

        let addr = ARRAY1_ADDR + user_index as u64;

        if confidence > 0.7 && predicted_in_bounds {

            if verbose { println!("SPECULATIVE EXECUTION: Proceeding with memory access..."); }

            // speculatively read array1[x] from simulated memory, wherever it points
            let (byte, _) = self.cache.read(addr);

            // and leave a secret-dependent footprint in the cache
            self.probe.encode(&mut self.cache, byte);
            if verbose {
                println!("SIDE EFFECT: Read 0x{:x}, cached probe[{}] at address 0x{:x}",
                    addr, byte, self.probe.addr(byte));
            }

        }

        // actual bounds check, architectural verification
        let actual_in_bounds = self.bounds_check(user_index);
        self.branch_predictor.train(actual_in_bounds);

        if actual_in_bounds {
            if verbose { println!("ARCHITECTURAL: Access allowed, returning data"); }

            Some(self.cache.read(addr).0)

        } else {
            if verbose {
                println!("ARCHITECTURAL: Access denied, speculation effects rolled back");
                println!("(however side effects remain)");
            }

            None
        }
    }

    /// Full attack on one secret byte: train, flush the probe array, call the
    /// victim out of bounds, then reload the probe array.
    pub fn leak_byte(&mut self, offset: usize) -> Option<u8> {

        for i in 0..TRAINING_ROUNDS {
            self.victim_function(i % ARRAY1_LEN);
        }

        // training also touched probe lines (for array1's public values)
        self.probe.flush_all(&mut self.cache);

        let result = self.victim_function(Self::malicious_index(offset));
        debug_assert!(result.is_none(), "out-of-bounds call must be rejected");

        self.extract_secret_byte()
    }

    // attacker probes cache to extract speculative side effects
    pub fn extract_secret_byte(&mut self) -> Option<u8> {

        if self.verbose { println!("ATTACKER: Probing cache to extract secret..."); }

        match self.probe.reload(&mut self.cache).as_slice() {
            [byte] => {
                if self.verbose {
                    println!("ATTACKER: Found cached value! Secret byte = 0x{:02x} ('{}')",
                        byte, printable(*byte));
                }
                Some(*byte)
            }
            [] => {
                println!("ATTACKER: No probe line was cached - no secret extracted");
                None
            }
            hits => {
                println!("ATTACKER: Ambiguous result, {} probe lines cached: {:?}", hits.len(), hits);
                None
            }
        }
    }

    /// The secret as it sits in memory (for display and checking only;
    /// the attack never uses this).
    pub fn secret_in_memory(&self) -> Vec<u8> {
        (0..self.secret_len as u64)
            .map(|i| self.cache.memory.peek(SECRET_ADDR + i))
            .collect()
    }

}

fn printable(b: u8) -> char {
    if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' }
}

pub fn demo_spectre_attack(secret: &str) {

    println!("\n---- Spectre Attack Simulation ----");
    println!("Simulating speculative execution vulnerability...\n");

    let mut spectre = SpectreSimulator::new(secret.as_bytes());

    println!("Memory layout:");
    println!("  array1 (public, {} bytes) at 0x{:x}", ARRAY1_LEN, ARRAY1_ADDR);
    println!("  secret ({} bytes)         at 0x{:x}: {:?}",
        spectre.secret_len, SECRET_ADDR, String::from_utf8_lossy(&spectre.secret_in_memory()));
    println!("  probe array (256 x 4KB)   at 0x{:x}\n", PROBE_BASE);

    if spectre.secret_len == 0 {
        println!("Secret is empty - nothing to leak");
        return;
    }

    println!("--- Phase 1: Training Branch Predictor ---");
    for i in 0..10 {
        println!("Training iteration {}: bounds check with valid index", i);
        let value = spectre.victim_function(i % ARRAY1_LEN);
        println!("Returned array1[{}] = {:?}", i % ARRAY1_LEN, value);
    }
    spectre.probe.flush_all(&mut spectre.cache);

    println!("\n--- Phase 2: Spectre Attack ---");
    let attack_index = SpectreSimulator::malicious_index(0);
    println!("Calling victim with index {} (array1 + {} = secret[0], out of bounds)",
        attack_index, attack_index);

    let result = spectre.victim_function(attack_index);
    println!("Architectural result: {:?}", result);

    println!("\n--- Phase 3: Cache Side Channel Extraction ---");
    if let Some(leaked_byte) = spectre.extract_secret_byte() {
        println!(" SUCCESS: Spectre attack leaked secret data from memory!");
        println!("   Leaked byte: 0x{:02x} ('{}')", leaked_byte, printable(leaked_byte));
    } else {
        println!(" Attack failed - no secret data extracted");
    }

    println!("\n--- Phase 4: Multi-byte Extraction ---");
    println!("Leaking all {} secret bytes (train, flush, attack, reload)...", spectre.secret_len);
    spectre.verbose = false;

    let leaked: Vec<Option<u8>> = (0..spectre.secret_len)
        .map(|offset| {
            let byte = spectre.leak_byte(offset);
            match byte {
                Some(b) => println!("  secret[{:2}] = 0x{:02x} ('{}')", offset, b, printable(b)),
                None => println!("  secret[{:2}] = (failed)", offset),
            }
            byte
        })
        .collect();

    let actual = spectre.secret_in_memory();
    let correct = leaked.iter().zip(&actual).filter(|(l, a)| **l == Some(**a)).count();
    let shown: String = leaked.iter().map(|b| b.map_or('?', printable)).collect();
    println!("\nOriginal: {:?}", String::from_utf8_lossy(&actual));
    println!("Leaked:   {:?}", shown);
    println!("Accuracy: {}/{} bytes correct ({:.1}%)",
        correct, actual.len(), correct as f64 / actual.len() as f64 * 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaks_entire_secret_from_memory() {
        let secret = b"SECRET_PASS_123!";
        let mut s = SpectreSimulator::new(secret);
        s.verbose = false;
        let leaked: Vec<_> = (0..secret.len()).map(|i| s.leak_byte(i)).collect();
        let expected: Vec<_> = secret.iter().map(|&b| Some(b)).collect();
        assert_eq!(leaked, expected);
    }

    #[test]
    fn out_of_bounds_is_rejected_architecturally() {
        let mut s = SpectreSimulator::new(b"x");
        s.verbose = false;
        assert_eq!(s.victim_function(SpectreSimulator::malicious_index(0)), None);
    }

    #[test]
    fn in_bounds_returns_public_array_from_memory() {
        let mut s = SpectreSimulator::new(b"x");
        s.verbose = false;
        assert_eq!(s.victim_function(0), Some(1));
        assert_eq!(s.victim_function(ARRAY1_LEN - 1), Some(ARRAY1_LEN as u8));
    }

    #[test]
    fn no_leak_without_speculation() {
        // An untrained, pessimistic predictor never speculates, so nothing leaks.
        let mut s = SpectreSimulator::new(b"S");
        s.verbose = false;
        s.branch_predictor.history = vec![false; 10];
        s.branch_predictor.confidence = 1.0;
        s.probe.flush_all(&mut s.cache);
        s.victim_function(SpectreSimulator::malicious_index(0));
        assert_eq!(s.extract_secret_byte(), None);
    }
}
