// trap.rs
// Trap routines
// Stephen Marz
// 10 October 2019

use crate::cpu::{sstatus_write, TrapFrame};
use crate::{print, println, plic, uart, cpu};
use crate::intr::devintr;
use crate::intr::Intr::Timer;
use crate::syscall::do_syscall;

#[no_mangle]
extern "C" fn m_trap(epc: usize,
                     tval: usize,
                     cause: usize,
                     hart: usize,
                     _status: usize,
                     frame: &mut TrapFrame)
                     -> usize
{
    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async = { cause >> 63 & 1 == 1 };
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous trap
        match cause_num {
            3 => {
                // Machine software
                println!("Machine software interrupt CPU#{}", hart);
            }
            7 => unsafe {
                // Machine timer
                let mtimecmp = 0x0200_4000 as *mut u64;
                let mtime = 0x0200_bff8 as *const u64;
                // The frequency given by QEMU is 10_000_000 Hz, so this sets
                // the next interrupt to fire one second from now.
                mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
                println!("Machine timer interrupt CPU#{}", hart);
            },
            11 => {
                // Machine externa
                println!("Machine external interrupt CPU#{}", hart);
            },
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    } else {
        // Synchronous trap
        match cause_num {
            2 => {
                // Illegal instruction
                panic!("Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n", hart, epc, tval);
            }
            8 => {
                // Environment (system) call from User mode
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc = do_syscall(return_pc, frame);
            }
            9 => {
                // Environment (system) call from Supervisor mode
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc = do_syscall(return_pc, frame);
            }
            11 => {
                // Environment (system) call from Machine mode
                panic!("E-call from Machine mode! CPU#{} -> 0x{:08x}\n", hart, epc);
            }
            // Page faults
            12 => {
                // Instruction page fault
                println!("Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            }
            13 => {
                // Load page fault
                println!("Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            }
            15 => {
                // Store page fault
                println!("Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            }
            _ => {
                panic!("Unhandled sync trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    };
    // Finally, return the updated program counter
    return_pc
}

/// Process interrupt from supervisor mode
#[no_mangle]
extern "C" fn kerneltrap() {
    use riscv::register;

    let epc = register::sepc::read();
    let tval = register::stval::read();
    let cause = register::scause::read();
    // let hart = arch::hart_id();
    let hart = 0;
    let sstatus = register::sstatus::read();
    let sstatus_bits = cpu::sstatus_read();

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
    if cpu::intr_get() {
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
        // if my_cpu().scheduler_context.regs[0] != 0 {
        //     let p = &my_cpu().process;
        //     if let Some(p) = p {
        //         if p.state == process::ProcessState::RUNNING {
        //             yield_cpu();
        //         }
        //     }
        // }
    }

    register::sepc::write(epc);
    sstatus_write(sstatus_bits);
}

/// Initialize supervisor-mode trap
pub unsafe fn hartinit() {
    use riscv::register::*;
    stvec::write(crate::symbols::kernelvec as usize, stvec::TrapMode::Direct);
}