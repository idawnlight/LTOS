//! This module is automatically generated with `symbols_gen.rs.py`,
//! which contains all linker script symbols in `kernel.ld` and a wrapper function
//! to safely get them.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern "C" { static _heap_start: usize; }
#[inline] pub fn HEAP_START() -> usize { unsafe { &_heap_start as *const _ as _ } }
extern "C" { static _heap_size: usize; }
#[inline] pub fn HEAP_SIZE() -> usize { unsafe { &_heap_size as *const _ as _ } }
extern "C" { static _text_start: usize; }
#[inline] pub fn TEXT_START() -> usize { unsafe { &_text_start as *const _ as _ } }
extern "C" { static _text_end: usize; }
#[inline] pub fn TEXT_END() -> usize { unsafe { &_text_end as *const _ as _ } }
extern "C" { static _rodata_start: usize; }
#[inline] pub fn RODATA_START() -> usize { unsafe { &_rodata_start as *const _ as _ } }
extern "C" { static _rodata_end: usize; }
#[inline] pub fn RODATA_END() -> usize { unsafe { &_rodata_end as *const _ as _ } }
extern "C" { static _data_start: usize; }
#[inline] pub fn DATA_START() -> usize { unsafe { &_data_start as *const _ as _ } }
extern "C" { static _data_end: usize; }
#[inline] pub fn DATA_END() -> usize { unsafe { &_data_end as *const _ as _ } }
extern "C" { static _bss_start: usize; }
#[inline] pub fn BSS_START() -> usize { unsafe { &_bss_start as *const _ as _ } }
extern "C" { static _bss_end: usize; }
#[inline] pub fn BSS_END() -> usize { unsafe { &_bss_end as *const _ as _ } }
extern "C" { static _kernel_stack_start: usize; }
#[inline] pub fn KERNEL_STACK_START() -> usize { unsafe { &_kernel_stack_start as *const _ as _ } }
extern "C" { static _kernel_stack_end: usize; }
#[inline] pub fn KERNEL_STACK_END() -> usize { unsafe { &_kernel_stack_end as *const _ as _ } }
extern "C" { static _trampoline_text_start: usize; }
#[inline] pub fn TRAMPOLINE_TEXT_START() -> usize { unsafe { &_trampoline_text_start as *const _ as _ } }
