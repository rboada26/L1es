use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "l1es")]
#[command(about = "Cache simulator and timing-based side-channel attack demonstration")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Output directory for results
    #[arg(short, long, default_value = "results")]
    pub output: PathBuf,
    
    /// Generate detailed CSV reports
    #[arg(long)]
    pub csv: bool,
    
    /// Generate JSON reports
    #[arg(long)]
    pub json: bool,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run basic cache functionality tests
    Basic {
        /// Cache size in KB
        #[arg(short, long, default_value = "32")]
        size: usize,
        
        /// Cache line size in bytes
        #[arg(short, long, default_value = "64")]
        line_size: usize,
        
        /// Associativity (ways)
        #[arg(short, long, default_value = "4")]
        ways: usize,
    },
    
    /// Compare different cache configurations
    Compare {
        /// Include all replacement policies
        #[arg(long)]
        all_policies: bool,
        
        /// Include all associativities
        #[arg(long)]
        all_ways: bool,
    },
    
    /// Run attack simulations
    Attack {
        #[command(subcommand)]
        attack_type: AttackCommands,
    },
    
    /// Run comprehensive benchmark suite
    Benchmark {
        /// Number of iterations for each test
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
        
        /// Save detailed timing data
        #[arg(long)]
        detailed: bool,
    },
    
    /// Generate attack effectiveness report
    Report {
        /// Cache configurations to analyze
        #[arg(long, value_delimiter = ',')]
        configs: Vec<String>,
        
        /// Attack scenarios to test
        #[arg(long, value_delimiter = ',')]
        scenarios: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AttackCommands {
    /// Prime+Probe attack demonstration
    PrimeProbe {
        /// Target cache set
        #[arg(short, long, default_value = "3")]
        target_set: usize,
    },
    
    /// Flush+Reload attack demonstration
    FlushReload {
        /// Number of monitored addresses
        #[arg(short, long, default_value = "5")]
        addresses: usize,
    },
    
    /// Spectre attack simulation
    Spectre {
        /// Secret data to leak
        #[arg(short, long, default_value = "SECRET_DATA")]
        secret: String,
    },
    
    /// Meltdown attack simulation
    Meltdown {
        /// Kernel data to extract
        #[arg(short, long, default_value = "KERNEL_SECRET")]
        kernel_data: String,
    },
    
    /// Run all attack demonstrations
    All,
}
