use crate::cache::Cache;

pub struct MeltdownSimulator {
    pub cache: Cache,
    pub kernel_memory: Vec<u8>,
    pub probe_array: Vec<u64>,
    pub page_fault_occured: bool,
}

impl MeltdownSimulator {
    
    pub fn new() -> Self {

        let cache = Cache::new(64,8,64);

        let kernel_memory = b"KERNEL_SECRET_KEY_123!".to_vec();

        let mut probe_array = Vec::new();
        for i in 0..256 {
            probe_array.push(0x200000 + (i as u64 * 4096));
        }

        MeltdownSimulator {

            cache,
            kernel_memory,
            probe_array,
            page_fault_occured: false,
        }
    }

    fn access_kernel_memory(&mut self, kernel_addr: usize) -> Option<u8> {

        println!("Accessing kernel memory at offset: {}", kernel_addr);

        if kernel_addr >= self.kernel_memory.len() {
            println!("ERROR: Seg Fault (invalid kernel address)");
            self.page_fault_occured = true;
            return None;
        }
        println!("Reading kernel byte...");
        Some(self.kernel_memory[kernel_addr])
    }

    pub fn meltdown_attack(&mut self, kernel_offset: usize) -> bool {

        println!("---- Meltdown Attack Sequence ----");
        println!("1. CPU begins out-of-order execution...");
        println!("2. SPECULATIVE: Accessing privileged kernel memory");
       
        let secret_byte = if kernel_offset < self.kernel_memory.len() {
            self.kernel_memory[kernel_offset]
        } else {
            0xDE
        };

        println!("3. SIDE EFFECT: Encoding secret in cache...");
        let probe_addr = self.probe_array[secret_byte as usize];
        self.cache.access(probe_addr);

        println!("Cached probe_array[{}] at 0x{:x}", secret_byte, probe_addr);
        println!("4. PRIVILEGE CHECK: Kernel access denied!");
        println!("   Exception raised, but cache side effects remain");
        self.page_fault_occured = true;

        println!("5. EXCEPTION HANDLER: Recovering from fault...");
        println!("   (Architectural state rolled back, cache state preserved)");
        
        true

    }

    pub fn extract_kernel_byte(&mut self) -> Option<u8> {
        println!("6. EXTRACTION: Probing cache for kernel secrets...");

        for &addr in &self.probe_array {
            self.cache.flush(addr);
        }

        for (byte_val, &addr) in self.probe_array.iter().enumerate() {
            let (hit, time) = self.cache.access(addr);

            if hit && time < 10 {
                 println!("   FOUND: Kernel byte = 0x{:02x} ('{}')", 
                         byte_val, byte_val as u8 as char);
                return Some(byte_val as u8);
            }
        }
        None

    }

    pub fn handle_exception(&mut self) {
        if self.page_fault_occured {
             println!("KERNEL: Page fault handler executed");
            println!("        Process continues after exception");
            self.page_fault_occured = false;
        }
    }
}


pub fn demo_meltdown_attack() {

    println!("---- Meltdown Attack Simulator ----");
    println!("Simulating privilege escalation via out-of-order execution...\n");
    
    let mut meltdown = MeltdownSimulator::new();

    println!("Kernel memory: {:?}", String::from_utf8_lossy(&meltdown.kernel_memory));
    println!("User process attempting to read kernel memory...\n");

    let mut extracted_secret = Vec::new();

    for kernel_offset in 0..std::cmp::min(10, meltdown.kernel_memory.len()) {
        println!("---- Extracting Kernel byte {} ----", kernel_offset);

        let attack_success = meltdown.meltdown_attack(kernel_offset);

        if attack_success {
            // extract from the cache side channel
            if let Some(leaked_byte) = meltdown.extract_kernel_byte() {

                extracted_secret.push(leaked_byte);
                println!("Successfully extracted kernel byte !");
            } else {
                println!("Extraction Failed");
            }
        }
        
        meltdown.handle_exception();
        println!();
    }

    println!("---- Meltdown Attack Results ----");
    if !extracted_secret.is_empty() {
        println!("CRITICAL: Meltdown successfully extracted kernel data!");
        println!("Original: {:?}", String::from_utf8_lossy(&meltdown.kernel_memory[..extracted_secret.len()]));
        println!("Extracted: {:?}", String::from_utf8_lossy(&extracted_secret));
        
        let accuracy = extracted_secret.iter()
            .zip(meltdown.kernel_memory.iter())
            .filter(|(a, b)| a == b)
            .count();
        println!("   Accuracy: {}/{} bytes correct ({:.1}%)", 
                 accuracy, extracted_secret.len(),
                 accuracy as f64 / extracted_secret.len() as f64 * 100.0);
    } else {
        println!("Attack failed - no kernel data extracted");
    }
}
