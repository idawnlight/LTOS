use core::arch::asm;
use core::sync::atomic::Ordering;
use core::time::Duration;
use riscv::register::*;

/// Get current time from MMIO
pub fn time() -> Duration {
    let mtime = crate::clint::CLINT_MTIME_BASE as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}

/// Enable interrupt
pub fn intr_on() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
        sstatus::set_sie();
    }
}

/// Turn off interrupt
pub fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}

/// Check if interrupt is enabled
pub fn intr_get() -> bool {
    sstatus::read().sie()
}

#[inline(always)]
#[allow(unused_assignments)]
pub fn hart_id() -> usize {
    let rval: usize;
    unsafe {
        asm!(
        "mv {0}, tp",
        out(reg) rval,
        );
    }
    rval
}

#[inline(always)]
pub fn __sync_synchronize() {
    core::sync::atomic::compiler_fence(Ordering::SeqCst);
    unsafe { asm!("fence"); }
}

#[inline(always)]
pub fn __sync_lock_test_and_set(a: &u32, mut b: u32) -> u32 {
    unsafe {
        asm!(
        "amoswap.w.aq {0}, {1}, ({2})",
        out(reg) b,
        in(reg) b,
        in(reg) a,
        );
    }
    b
}

#[inline(always)]
pub fn __sync_lock_release(a: &u32) {
    unsafe {
        asm!(
        "amoswap.w zero, zero, ({0})",
        in(reg) a,
        );
    }
}