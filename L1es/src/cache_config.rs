#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheType {
    DirectMapped,
    SetAssociative(usize),
    FullyAssociative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplacementPolicy {
    LRU,
    FIFO,
    Random,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_type: CacheType,
    pub total_size: usize,
    pub line_size: usize,
    pub replacement_policy: ReplacementPolicy,
    pub name: String,
}

impl CacheConfig {

    pub fn direct_mapped(total_size: usize, line_size: usize) -> Self {
        CacheConfig{
            cache_type: CacheType::DirectMapped,
            total_size,
            line_size,
            replacement_policy: ReplacementPolicy:LRU,
            name: format!("Direct Mapped {}KB, {}-byte lines", total_size / 1024, line_size),
        }
    }
    pub fn set_associative(total_size: usize, line_size: usize, associativity: usize, policy: ReplacementPolicy) -> Self {
        CacheConfig{
            cache_type: CacheType::SetAssociative(associativity),
            total_size,
            line_size,
            replacement_policy: policy,
            name: format!("{}-way {}KB, {}-byte lines, {:?}", associativity, total_size/1024, line_size, policy),
        }
    }

    pub fn fully_associative(total_size: usize, line_size: usize, policy: ReplacementPolicy) -> Self {
        CacheConfig {
            cache_type: CacheType::FullyAssociative,
            total_size,
            line_size,
            replacement_policy: policy,
            name: format!("Fully-associative {}KB, {}-byte lines, {:?}", total_size/1024, line_size, policy),
        }
    }

    pub fn num_sets(&self) -> usize {
        match self.cache_type {
            CacheType::DirectMapped => self.total_size / self.line_size,
            CacheType::SetAssociative(ways) => self.total_size / (self.line_size * ways),
            CacheType::FullyAssociative => 1,
        }
    }

    pub fn associativity(&self) -> usize {
        match self.cache_type {
            CacheType::DirectMapped => 1,
            CacheType::SetAssociative(ways) => ways,
            CacheType::FullyAssociative => self.total_size / self.line_size,
        }
    }
}

pub struct CacheConfigs;

impl CacheConfigs{
    pub fn get_test_configs() -> Vec<CacheConfig> {
        vec![
            CacheConfig::direct_mapped(16*1024, 64), // 16kb, 64 byte lines 
            CacheConfig::direct_mapped(32*1024, 64) , // 32kb, 64 byte lines

            CacheConfig::set_associative(32 * 1024, 64, 2, ReplacementPolicy::LRU),
            CacheConfig::set_associative(32 * 1024, 64, 4, ReplacementPolicy::LRU),
            CacheConfig::set_associative(32 * 1024, 64, 8, ReplacementPolicy::LRU),
            CacheConfig::set_associative(32 * 1024, 64, 4, ReplacementPolicy::FIFO),
            CacheConfig::set_associative(32 * 1024, 64, 4, ReplacementPolicy::Random),
            
            // Fully associative caches
            CacheConfig::fully_associative(8 * 1024, 64, ReplacementPolicy::LRU),
            CacheConfig::fully_associative(16 * 1024, 64, ReplacementPolicy::FIFO),
        ]
    }

    pub fn get_attack_configs() -> Vec<CacheConfig> {
        vec![
            CacheConfig::direct_mapped(16 * 1024, 64),
            
            // Low associativity - still vulnerable
            CacheConfig::set_associative(32 * 1024, 64, 2, ReplacementPolicy::LRU),
            
            // Higher associativity - harder to attack
            CacheConfig::set_associative(32 * 1024, 64, 8, ReplacementPolicy::LRU),
            
            // Fully associative - different attack characteristics
            CacheConfig::fully_associative(8 * 1024, 64, ReplacementPolicy::LRU),
        ]
    }
}
