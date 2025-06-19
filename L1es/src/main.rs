mod line;
mod set;
mod cache;
mod simulator;

pub mod attacks {

    pub mod prime_probe;
}
fn main() {
    
    simulator::run_basic_test();

    attacks::prime_probe::demo_prime_probe();
}
