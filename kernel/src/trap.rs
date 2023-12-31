use crate::{println, arch, page, syscall, process};
use crate::arch::hart_id;
use crate::intr::devintr;
use crate::intr::Intr::Timer;
use crate::process::{my_cpu, my_proc, Register, yield_cpu};
use crate::symbols::{kernelvec, TRAMPOLINE_START, TRAMPOLINE_TEXT_START, TRAPFRAME_START, userret, uservec};

/// Process interrupt from supervisor mode
#[no_mangle]
extern "C" fn kerneltrap() {
    use riscv::register;

    let epc = register::sepc::read();
    let tval = register::stval::read();
    let cause = register::scause::read();
    let hart = hart_id();
    let sstatus = register::sstatus::read();
    let sstatus_bits = arch::sstatus_read();

    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async = cause.is_interrupt();
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause.code();
    if sstatus.spp() != register::sstatus::SPP::Supervisor {
        panic!("not from supervisor mode, async {}, hart {}, {:x}, epc {:x}, tval {}", is_async, hart, cause_num, epc, tval);
    }
    if arch::intr_get() {
        panic!("interrupt not disabled");
    }

    let dev_intr;

    if is_async {
        dev_intr = devintr();
        if dev_intr.is_none() {
            panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
        }
    } else {
        // Synchronous trap
        match cause_num {
            2 => {
                // Illegal instruction
                panic!(
                    "Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            8 => {
                // Environment (system) call from User mode
                panic!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
            }
            9 => {
                // Environment (system) call from Supervisor mode
                panic!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
            }
            11 => {
                // Environment (system) call from Machine mode
                panic!("E-call from Machine mode! CPU#{} -> 0x{:08x}\n", hart, epc);
            }
            // Page faults
            12 => {
                // Instruction page fault
                panic!(
                    "Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            }
            13 => {
                // Load page fault
                panic!(
                    "Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            }
            15 => {
                // Store page fault
                panic!(
                    "Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
            }
            _ => {
                panic!("Unhandled sync trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    };

    if dev_intr == Some(Timer) {
        println!("timer interrupt, supervisor");
        if my_cpu().scheduler_context.regs[0] != 0 {
            let p = &my_cpu().process;
            if let Some(p) = p {
                if p.state == process::ProcessState::RUNNING {
                    yield_cpu();
                }
            }
        }
    }

    register::sepc::write(epc);
    arch::sstatus_write(sstatus_bits);
}

/// Called by `uservec` in `trampoline.S`, return from user space.
#[no_mangle]
pub extern "C" fn usertrap() -> ! {
    use riscv::register::*;
    if sstatus::read().spp() != sstatus::SPP::User {
        panic!("not from user mode");
    }
    unsafe {
        stvec::write(kernelvec as usize, stvec::TrapMode::Direct);
    }
    let p = my_proc();
    p.trapframe.epc = sepc::read();
    let scause = scause::read().bits();

    let mut intr = None;
    if scause == 8 {
        p.trapframe.epc += 4;
        arch::intr_on();
        p.trapframe.regs[Register::a0 as usize] = syscall::syscall() as usize;
    } else {
        intr = devintr();
        if intr.is_none() { panic!("unexpected scause {:x}", scause) }
    }

    if intr == Some(Timer) {
        yield_cpu();
    }

    usertrapret();
}

/// Jump to user space through trampoline after trapframe is properly set. Calls `userret` in `trampoline.S`.
#[inline]
fn trampoline_userret(tf: usize, satp_val: usize) -> ! {
    let userret_offset = userret as usize - TRAMPOLINE_TEXT_START();
    let fn_addr = (TRAMPOLINE_START + userret_offset) as *const ();
    let fn_addr: extern "C" fn(usize, usize) -> ! = unsafe { core::mem::transmute(fn_addr) };
    (fn_addr)(tf, satp_val)
}

/// Jump to user space through trampoline
///
/// As this function is called by `forkret`, where there may
/// be memory leak related to RAII, content of `usertrapret`
/// should be wrapped in brackets so that all objects are
/// dropped before jumping to trampoline.
pub fn usertrapret() -> ! {
    let satp_val: usize;
    {
        use riscv::register::*;
        arch::intr_off();

        // send syscalls, interrupts, and exceptions to trampoline.S
        unsafe {
            stvec::write(
                (uservec as usize - TRAMPOLINE_TEXT_START()) + TRAMPOLINE_START,
                stvec::TrapMode::Direct,
            );
        }

        // set up trapframe values that uservec will need when
        // the process next re-enters the kernel.
        let p = my_proc();
        p.trapframe.satp = satp::read().bits();
        p.trapframe.sp = p.kstack_sp;
        p.trapframe.trap = usertrap as usize;
        p.trapframe.hartid = hart_id();

        // println!("trap 0x{:x}", proc_cpu.process.trapframe.trap);

        // set S Previous Privilege mode to User.
        unsafe {
            sstatus::set_spie();
            sstatus::set_spp(sstatus::SPP::User);
        }

        // set S Exception Program Counter to the saved user pc.
        sepc::write(p.trapframe.epc);

        // tell trampoline.S the user page table to switch to.
        let root_ppn = &mut *p.pgtable as *mut page::Table as usize;
        satp_val = arch::build_satp(8, 0, root_ppn);
    }
    // jump to trampoline.S at the top of memory, which
    // switches to the user page table, restores user registers,
    // and switches to user mode with sret.
    trampoline_userret(TRAPFRAME_START, satp_val)
}

/// Initialize supervisor-mode trap
pub unsafe fn hartinit() {
    use riscv::register::*;
    stvec::write(kernelvec as usize, stvec::TrapMode::Direct);
}