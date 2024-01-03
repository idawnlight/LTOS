#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(inline_const)]

extern crate alloc;

use core::arch::asm;

pub mod uart;
pub mod page;
pub mod trap;
pub mod plic;
pub mod syscall;
pub mod clint;
pub mod intr;
pub mod start;
pub mod spinlock;
pub mod arch;
pub mod process;
pub mod symbols;

pub mod print;
pub mod mem;
pub mod virtio;
pub mod file;
pub mod elf;
pub mod test;

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////
#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}

#[no_mangle]
extern "C"
fn abort() -> ! {
    loop {
        unsafe {
            // The asm! syntax has changed in Rust.
            // For the old, you can use llvm_asm!, but the
            // new syntax kicks ass--when we actually get to use it.
            asm!("wfi");
        }
    }
}