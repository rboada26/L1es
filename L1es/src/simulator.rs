use crate::cache::Cache;

pub fn run_basic_test(){

    println!("---- Cache Simulator Basic Test ----");
    
    // small cache: 4 sets, 2-way associative, and 64-byte-lines
    let mut cache = Cache::new(4,2,64);

    println!("Cache configuration: {} sets, {}-way, {}-byte-lines",
        cache.num_sets, cache.associativity, cache.line_size);

    // Test hit and miss pattern
    println!("\n ---- Test 1: Basic Access Pattern ----");
    test_basic_pattern(&mut cache);

    //Test cache set conflicts, important for prime+probe

    println!("\n ---- Test 2: Set conflicts ----");
     test_set_conflicts(&mut cache);

     //Test flush operation, important for flush+reload 
     println!("\n ---- Test 3: Flush Operation ----");
     test_flush(&mut cache);

     let stats = cache.stats();

     println!("\n---- Final Cache Stats ----");
     println!("Total accesses: {}", stats.total_accesses);
     println!("Hits: {}, Misses {}",stats.total_hits, stats.total_misses);
     println!("Hit rate: {:.2}%", stats.hit_rate*100.0);

}

fn test_basic_pattern(cache: &mut Cache){

    let addresses = [0x1000, 0x2000, 0x1000, 0x3000, 0x2000];

    for addr in addresses {

        let (hit, cycles) = cache.access(addr);

        println!("Access 0x{:x}: {} ({} cycles)", addr, if hit {"HIT"} else {"MISS"}, cycles);

    }

}


fn test_set_conflicts(cache: &mut Cache){

    // These addresses should map to the same cache set
    // With 4 sets and 64-byte lines: set_index = (addr >> 6) & 3

    let same_set_addrs = [0x0000, 0x0100, 0x0200, 0x0300]; // all map to set 0

    println!("Accessing addresses that map to same set: ");
    for addr in same_set_addrs {

        let (hit, cycles) = cache.access(addr);
        println!("Access 0x{:x}: {} ({} cycles)", 
                 addr, if hit { "HIT" } else { "MISS" }, cycles);
    }


}

fn test_flush(cache: &mut Cache){

    let addr = 0x5000;

    // Access to load into cache
    let (hit1, _) = cache.access(addr);
    println!("Initial access 0x{:x}: {}", addr, if hit1 { "HIT" } else { "MISS" });
    
    // Access again - should be hit
    let (hit2, _) = cache.access(addr);
    println!("Second access 0x{:x}: {}", addr, if hit2 { "HIT" } else { "MISS" });
    
    // Flush the address
    let flushed = cache.flush(addr);
    println!("Flush 0x{:x}: {}", addr, if flushed { "SUCCESS" } else { "NOT FOUND" });
    
    // Access again - should be miss
    let (hit3, _) = cache.access(addr);
    println!("After flush 0x{:x}: {}", addr, if hit3 { "HIT" } else { "MISS" });

}

fn main(){
    run_basic_test();
}
