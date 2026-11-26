use crate::cache::Cache;
use crate::attacks::covert_channel::ProbeArray;

/// Start of the kernel half of the address space: user code may not read here.
pub const KERNEL_SPACE_START: u64 = 0xffff_8000_0000_0000;

/// Where the kernel secret lives (inside the kernel's direct map).
pub const KERNEL_SECRET_ADDR: u64 = 0xffff_8880_0000_0000;

/// User-space probe array for the cache covert channel.
pub const PROBE_BASE: u64 = 0x200000;

pub struct MeltdownSimulator {
    pub cache: Cache,           // cache backed by simulated memory
    pub kernel_len: usize,
    pub probe: ProbeArray,
    pub page_fault_occured: bool,
}

impl MeltdownSimulator {

    /// Place `kernel_data` in simulated kernel memory.
    pub fn new(kernel_data: &[u8]) -> Self {

        let mut cache = Cache::new(64,8,64);
        cache.preload(KERNEL_SECRET_ADDR, kernel_data);

        MeltdownSimulator {

            cache,
            kernel_len: kernel_data.len(),
            probe: ProbeArray::new(PROBE_BASE),
            page_fault_occured: false,
        }
    }

    fn is_privileged(addr: u64) -> bool {
        addr >= KERNEL_SPACE_START
    }

    /// Architectural (permission-checked) read from user mode.
    pub fn user_read(&mut self, addr: u64) -> Option<u8> {

        if Self::is_privileged(addr) {
            println!("ERROR: Page fault (user access to kernel address 0x{:x})", addr);
            self.page_fault_occured = true;
            return None;
        }
        Some(self.cache.read(addr).0)
    }

    pub fn meltdown_attack(&mut self, kernel_offset: usize) -> bool {

        let addr = KERNEL_SECRET_ADDR + kernel_offset as u64;

        // start from a clean covert channel
        self.probe.flush_all(&mut self.cache);

        println!("---- Meltdown Attack Sequence ----");
        println!("1. CPU begins out-of-order execution...");
        println!("2. SPECULATIVE: Reading kernel address 0x{:x} from memory", addr);

        // The load executes before its permission check retires.
        let (secret_byte, _) = self.cache.read(addr);

        println!("3. SIDE EFFECT: Encoding secret in cache...");
        self.probe.encode(&mut self.cache, secret_byte);

        println!("   Cached probe[{}] at 0x{:x}", secret_byte, self.probe.addr(secret_byte));
        println!("4. PRIVILEGE CHECK: Kernel access denied!");
        println!("   Exception raised, but cache side effects remain");
        self.page_fault_occured = Self::is_privileged(addr);

        println!("5. EXCEPTION HANDLER: Recovering from fault...");
        println!("   (Architectural state rolled back, cache state preserved)");

        true

    }

    pub fn extract_kernel_byte(&mut self) -> Option<u8> {
        println!("6. EXTRACTION: Timing reads of the probe array...");

        match self.probe.reload(&mut self.cache).as_slice() {
            [byte] => {
                println!("   FOUND: Kernel byte = 0x{:02x} ('{}')", byte, printable(*byte));
                Some(*byte)
            }
            [] => {
                println!("   No probe line was cached");
                None
            }
            hits => {
                println!("   Ambiguous: {} probe lines cached: {:?}", hits.len(), hits);
                None
            }
        }
    }

    pub fn handle_exception(&mut self) {
        if self.page_fault_occured {
             println!("KERNEL: Page fault handler executed");
            println!("        Process continues after exception");
            self.page_fault_occured = false;
        }
    }

    /// Kernel memory contents (for display and checking only;
    /// the attack never uses this).
    pub fn kernel_memory(&self) -> Vec<u8> {
        (0..self.kernel_len as u64)
            .map(|i| self.cache.memory.peek(KERNEL_SECRET_ADDR + i))
            .collect()
    }
}

fn printable(b: u8) -> char {
    if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' }
}

pub fn demo_meltdown_attack(kernel_data: &str) {

    println!("---- Meltdown Attack Simulator ----");
    println!("Simulating privilege escalation via out-of-order execution...\n");

    let mut meltdown = MeltdownSimulator::new(kernel_data.as_bytes());
    let original = meltdown.kernel_memory();

    println!("Kernel memory at 0x{:x}: {:?}", KERNEL_SECRET_ADDR, String::from_utf8_lossy(&original));

    println!("\nUser process tries a normal read first:");
    let direct = meltdown.user_read(KERNEL_SECRET_ADDR);
    println!("Result: {:?}", direct);
    meltdown.handle_exception();

    println!("\nUser process attempting to read kernel memory via Meltdown...\n");

    let mut extracted_secret = Vec::new();

    for kernel_offset in 0..meltdown.kernel_len {
        println!("---- Extracting Kernel byte {} ----", kernel_offset);

        let attack_success = meltdown.meltdown_attack(kernel_offset);

        if attack_success {
            // extract from the cache side channel
            if let Some(leaked_byte) = meltdown.extract_kernel_byte() {

                extracted_secret.push(leaked_byte);
                println!("Successfully extracted kernel byte !");
            } else {
                extracted_secret.push(b'?');
                println!("Extraction Failed");
            }
        }

        meltdown.handle_exception();
        println!();
    }

    println!("---- Meltdown Attack Results ----");
    if !extracted_secret.is_empty() {
        println!("CRITICAL: Meltdown successfully extracted kernel data!");
        println!("Original: {:?}", String::from_utf8_lossy(&original));
        println!("Extracted: {:?}", String::from_utf8_lossy(&extracted_secret));

        let accuracy = extracted_secret.iter()
            .zip(original.iter())
            .filter(|(a, b)| a == b)
            .count();
        println!("   Accuracy: {}/{} bytes correct ({:.1}%)",
                 accuracy, extracted_secret.len(),
                 accuracy as f64 / extracted_secret.len() as f64 * 100.0);
    } else {
        println!("Attack failed - no kernel data extracted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaks_kernel_memory() {
        let secret = b"KERNEL_SECRET_KEY_123!";
        let mut m = MeltdownSimulator::new(secret);
        let leaked: Vec<_> = (0..secret.len())
            .map(|i| {
                m.meltdown_attack(i);
                m.extract_kernel_byte()
            })
            .collect();
        let expected: Vec<_> = secret.iter().map(|&b| Some(b)).collect();
        assert_eq!(leaked, expected);
    }

    #[test]
    fn user_read_of_kernel_address_faults() {
        let mut m = MeltdownSimulator::new(b"K");
        assert_eq!(m.user_read(KERNEL_SECRET_ADDR), None);
        assert!(m.page_fault_occured);
    }

    #[test]
    fn user_read_of_user_address_works() {
        let mut m = MeltdownSimulator::new(b"K");
        m.cache.preload(0x5000, &[7]);
        assert_eq!(m.user_read(0x5000), Some(7));
        assert!(!m.page_fault_occured);
    }
}
