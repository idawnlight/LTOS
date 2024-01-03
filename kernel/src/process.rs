pub mod cpu;

use crate::arch;
use crate::process::cpu::CPU;
use crate::symbols::NCPUS;

/// An array holding all CPU information
static mut CPUS: [CPU; NCPUS] = [const { CPU::zero() }; NCPUS];

/// Get CPU object of current hart.
pub fn my_cpu() -> &'static mut CPU {
    unsafe { &mut CPUS[arch::hart_id()] }
}