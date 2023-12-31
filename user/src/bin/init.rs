// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![no_main]
#![feature(format_args_nl)]

use user::println;
use user::syscall::{fork, open, dup, exec};

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    open("/console", 0);
    dup(0);
    dup(0);
    println!("Hello world from user mode, through /console and open/dup");
    println!("starting a fork...");
    let p = fork();
    if p == 0 {
        println!("calling test1 in child...");
        exec("/test1", &["test1", "test2"]);
    } else {
        loop {}
    }
}
