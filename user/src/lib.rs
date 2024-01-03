#![no_std]

use core::panic::PanicInfo;

pub mod syscall_internal;
pub mod print;
pub mod syscall;
pub mod constant;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
    }
}
