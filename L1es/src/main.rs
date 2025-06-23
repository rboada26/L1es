mod line;
mod set;
mod cache;
mod simulator;

pub mod attacks {

    pub mod prime_probe;
    pub mod flush_reload;
    pub mod spectre_sim;
    pub mod meltdown_sim;
}
fn main() {
    
    simulator::run_basic_test();
/*
    attacks::prime_probe::demo_prime_probe();
    attacks::flush_reload::demo_flush_reload();
    attacks::spectre_sim::demo_spectre_attack();
*/
    attacks::meltdown_sim::demo_meltdown_attack();
}
