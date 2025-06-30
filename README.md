# L1ES - Cache Simulator and Side-Channel Attack Demonstration

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A comprehensive cache simulator and timing-based side-channel attack demonstration tool designed for security research, education, and microarchitectural analysis.

## 🌟 Features

### Cache Simulation
- 🔧 **Multiple Cache Architectures**: Direct-mapped, set-associative, fully associative
- ⚙️ **Configurable Parameters**: Cache size, line size, associativity, replacement policies
- 📈 **Replacement Policies**: LRU, FIFO, Random
- 📊 **Real-time Statistics**: Hit rates, access times, cache pressure analysis

### Side-Channel Attack Simulations
- 🎯 **Prime+Probe**: Cache set monitoring and eviction-based attacks
- 🔄 **Flush+Reload**: Shared memory timing attacks
- 🧠 **Spectre**: Speculative execution vulnerability simulation with branch prediction
- 💥 **Meltdown**: Out-of-order execution privilege bypass demonstration

### Data Analysis & Export
- 📊 **CSV Export**: Detailed numerical data for statistical analysis
- 📋 **JSON Export**: Structured data with comprehensive metadata
- 📈 **Timing Analysis**: Per-access timing data for research
- 📝 **Comprehensive Reports**: Attack effectiveness and configuration comparisons

## 🚀 Installation

### Prerequisites
- [Rust](https://rustup.rs/) (1.70.0 or later)

### From Source
```bash
# Clone the repository
git clone https://github.com/rboada26/L1es
cd L1es

# Build and install
cargo build --release
cargo install --path .

# Verify installation
l1es --version
```

## 📖 Usage

### Basic Commands

#### Cache Configuration Testing
```bash
# Test a 32KB, 4-way cache
l1es basic --size 32 --ways 4

# Export detailed timing data
l1es --csv --json basic --size 16 --ways 2

# Verbose output with statistics
l1es --verbose basic --size 64 --ways 8 --line-size 128
```

#### Cache Architecture Comparison
```bash
# Compare different cache configurations
l1es compare --all-policies

# Detailed comparison with data export
l1es --csv --verbose compare --all-ways

# Custom output directory
l1es --output research_data compare --all-policies
```

### Attack Simulations

#### Individual Attacks
```bash
# Prime+Probe attack on cache set 5
l1es attack prime-probe --target-set 5

# Flush+Reload monitoring 10 addresses
l1es attack flush-reload --addresses 10

# Spectre attack with custom secret
l1es attack spectre --secret "CONFIDENTIAL_DATA"

# Meltdown simulation
l1es attack meltdown --kernel-data "KERNEL_SECRET"
```

#### Comprehensive Attack Analysis
```bash
# Run all attacks with full data export
l1es --csv --json --verbose attack all

# Attack analysis with custom output
l1es --csv --output attack_analysis attack all
```

### Advanced Usage

#### Benchmarking
```bash
# Performance benchmark suite
l1es benchmark --iterations 5000 --detailed

# Custom benchmark with timing analysis
l1es --csv benchmark --iterations 1000 --detailed
```

#### Research Reports
```bash
# Generate attack effectiveness report
l1es report --configs "direct,2way,4way" --scenarios "aes,rsa"

# Comprehensive vulnerability analysis
l1es --json --verbose report --configs "all" --scenarios "crypto"
```

## 📊 Output Formats

### CSV Output
Detailed numerical data suitable for statistical analysis:
```csv
timestamp,config_name,test_name,hit_rate,miss_rate,avg_access_time,cache_size_kb,associativity,attack_success,leaked_bytes,total_accesses
2024-01-01T12:00:00Z,"4-way 32KB LRU",basic_test,0.75,0.25,25.5,32,4,N/A,0,1000
```

### JSON Output
Structured data with comprehensive metadata:
```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "config_name": "4-way 32KB LRU",
  "cache_stats": {
    "hit_rate": 0.75,
    "total_accesses": 1000,
    "average_access_time": 25.5
  },
  "attack_results": {
    "attack_type": "prime_probe",
    "success_rate": 0.85,
    "leaked_bytes": 12
  }
}
```

### Timing Data
Per-access timing information:
```csv
simulation_id,address,access_time,cache_hit,timestamp
0,0x1000,100,false,1
0,0x1000,1,true,2
```

## 🎓 Research Applications

### Security Research
- **Vulnerability Assessment**: Analyze cache-based side-channel susceptibility
- **Attack Development**: Test and refine timing-based attack techniques
- **Defense Evaluation**: Assess mitigation strategy effectiveness

### Education
- **Computer Architecture**: Demonstrate cache behavior and design trade-offs
- **Security Education**: Interactive side-channel attack learning
- **Performance Analysis**: Cache optimization and tuning

### Industry Applications
- **Processor Design**: Cache architecture security analysis
- **Software Security**: Application vulnerability assessment
- **Performance Engineering**: Cache-aware optimization

## 🔬 Technical Details

### Supported Cache Configurations
- **Direct-Mapped**: 1-way associative, simple and fast
- **Set-Associative**: 2-way, 4-way, 8-way, 16-way configurations
- **Fully Associative**: Maximum flexibility, complex replacement

### Attack Implementations
- **Prime+Probe**: Eviction set generation, cache set monitoring
- **Flush+Reload**: Shared memory exploitation, precise timing
- **Spectre**: Branch prediction training, speculative execution
- **Meltdown**: Privilege bypass simulation, out-of-order effects

### Timing Model
- **Cache Hit**: 1-3 cycles (configurable)
- **Cache Miss**: 100-300 cycles (configurable)
- **Memory Hierarchy**: L1, L2, L3 simulation support

## 🛠️ Development

### Building from Source
```bash
# Clone and build
git clone https://github.com/rboada26/L1es
cd L1es
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```


### Related Tools
- [Mastik](https://cs.adelaide.edu.au/~yval/Mastik/) - Micro-architectural attack toolkit
- [Intel Pin](https://software.intel.com/content/www/us/en/develop/articles/pin-a-dynamic-binary-instrumentation-tool.html) - Dynamic instrumentation
- [Cachegrind](https://valgrind.org/docs/manual/cg-manual.html) - Cache profiling

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This tool is designed for educational and research purposes only. Users are responsible for ensuring ethical and legal use of this software. The authors are not responsible for any misuse or damage caused by this tool.


## 🎯 Roadmap

- [ ] GPU cache simulation
- [ ] Network-based attacks
- [ ] Real hardware integration
- [ ] Web-based visualization
- [ ] ARM architecture support
- [ ] Cross-core attack simulation

