use core::arch::asm;
use riscv::register::*;
use crate::{arch, clint, cpu, info, mem, plic, print, println, process, trap, uart, virtio};
use crate::arch::hart_id;

#[no_mangle]
extern "C" fn kinit() {
    // We created kinit, which runs in super-duper mode
    // 3 called "machine mode".
    // The job of kinit() is to get us into supervisor mode
    // as soon as possible.
    // Interrupts are disabled for the duration of kinit()

    unsafe {
        // configure Physical Memory Protection to give supervisor mode
        // access to all of physical memory.
        pmpaddr0::write(0x3fffffffffffff);
        pmpcfg0::write(0xf);
        // next mode is supervisor mode
        mstatus::set_mpp(mstatus::MPP::Supervisor);
        // mret jump to kmain
        mepc::write(kmain as usize);
        // disable paging
        asm!("csrw satp, zero");
        // delegate all interrupts and exceptions to supervisor mode
        asm!("li t0, 0xffff");
        asm!("csrw medeleg, t0");
        asm!("li t0, 0xffff");
        asm!("csrw mideleg, t0");
        // save cpuid to tp
        asm!("csrr a1, mhartid");
        asm!("mv tp, a1");
        // set up timer interrupt
        clint::timer_init();
        // switch to supervisor mode
        asm!("mret");
    }
}

/// Controls whether other harts may start boot procedure
static mut MAY_BOOT: bool = false;

#[no_mangle]
extern "C" fn kmain() {
    if hart_id() == 0 {
        unsafe { uart::init(); }
        info!("booting LTOS on hart {}...", hart_id());
        info!("  UART... \x1b[0;32minitialized\x1b[0m");
        unsafe { mem::init(); }
        info!("  kernel page table... \x1b[0;32minitialized\x1b[0m");
        unsafe { virtio::init(); }
        info!("  virt-io... \x1b[0;32minitialized\x1b[0m");
        unsafe { plic::init(); }
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        mem::hartinit();
        info!("kernel page table configured");
        info!("  Trap... \x1b[0;32minitialized\x1b[0m");
        info!("  Timer... \x1b[0;32minitialized\x1b[0m");
        plic::hartinit();
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        unsafe { trap::hartinit(); }
        info!("  Interrupt... \x1b[0;32minitialized\x1b[0m");
        process::init_proc();
        info!("  Process... \x1b[0;32minitialized\x1b[0m");
        unsafe {
            asm!("fence");
            MAY_BOOT = true
        }
    } else {
        loop {
            if unsafe { MAY_BOOT } == true {
                break;
            }
        }
        info!("hart {} booting", hart_id());
        mem::hartinit();
        unsafe { trap::hartinit(); }
        plic::hartinit();
    }

    // cpu::wait_forever();
    process::scheduler()
}