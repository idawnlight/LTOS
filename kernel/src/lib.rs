#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(inline_const)]

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use core::arch::asm;
use riscv::register;
use riscv::register::{mepc, mstatus, sie, sstatus};

pub mod uart;
pub mod page;
pub mod trap;
pub mod cpu;
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
extern "C" {
    fn make_syscall(a: usize) -> usize;
}
fn rust_make_syscall(a: usize) -> usize {
    unsafe {
        make_syscall(a)
    }
}

// ///////////////////////////////////
// / ENTRY POINT
// ///////////////////////////////////
// #[no_mangle]
extern "C" fn kinit() {
    // We created kinit, which runs in super-duper mode
    // 3 called "machine mode".
    // The job of kinit() is to get us into supervisor mode
    // as soon as possible.
    // Interrupts are disabled for the duration of kinit()
    uart::Uart::new(0x1000_0000).init();
    println!("=== Now in kinit() ===");
    // page::init();
    // kmem::init();
    //
    // // Map heap allocations
    // let root_ptr = kmem::get_page_table();
    // let root_u = root_ptr as usize;
    // let mut root = unsafe { root_ptr.as_mut().unwrap() };
    // let kheap_head = kmem::get_head() as usize;
    // let total_pages = kmem::get_num_allocations();
    // unsafe {
    //     println!("TEXT:   0x{:x} -> 0x{:x}", TEXT_START, TEXT_END);
    //     println!("RODATA: 0x{:x} -> 0x{:x}", RODATA_START, RODATA_END);
    //     println!("DATA:   0x{:x} -> 0x{:x}", DATA_START, DATA_END);
    //     println!("BSS:    0x{:x} -> 0x{:x}", BSS_START, BSS_END);
    //     println!("STACK:  0x{:x} -> 0x{:x}", KERNEL_STACK_START, KERNEL_STACK_END);
    //     println!("HEAP:   0x{:x} -> 0x{:x}", kheap_head, kheap_head + total_pages * 4096);
    // }
    // id_map_range(
    //     &mut root,
    //     kheap_head,
    //     kheap_head + total_pages * 4096,
    //     page::EntryBits::ReadWrite.val(),
    // );
    // // Using statics is inherently unsafe.
    // unsafe {
    //     // Map heap descriptors
    //     let num_pages = HEAP_SIZE / page::PAGE_SIZE;
    //     id_map_range(
    //         &mut root,
    //         HEAP_START,
    //         HEAP_START + num_pages,
    //         page::EntryBits::ReadWrite.val(),
    //     );
    //     // Map executable section
    //     id_map_range(
    //         &mut root,
    //         TEXT_START,
    //         TEXT_END,
    //         page::EntryBits::ReadExecute.val(),
    //     );
    //     // Map rodata section
    //     // We put the ROdata section into the text section, so they can
    //     // potentially overlap however, we only care that it's read
    //     // only.
    //     id_map_range(
    //         &mut root,
    //         RODATA_START,
    //         RODATA_END,
    //         page::EntryBits::ReadExecute.val(),
    //     );
    //     // Map data section
    //     id_map_range(
    //         &mut root,
    //         DATA_START,
    //         DATA_END,
    //         page::EntryBits::ReadWrite.val(),
    //     );
    //     // Map bss section
    //     id_map_range(
    //         &mut root,
    //         BSS_START,
    //         BSS_END,
    //         page::EntryBits::ReadWrite.val(),
    //     );
    //     // Map kernel stack
    //     id_map_range(
    //         &mut root,
    //         KERNEL_STACK_START,
    //         KERNEL_STACK_END,
    //         page::EntryBits::ReadWrite.val(),
    //     );
    // }
    //
    // // UART
    // page::map(
	//     &mut root,
	//     0x1000_0000,
	//     0x1000_0000,
	//     page::EntryBits::ReadWrite.val(),
	//     0,
    // );
    //
    // // CLINT
    // //  -> MSIP
    // page::map(
	//     &mut root,
	//     0x0200_0000,
	//     0x0200_0000,
	//     page::EntryBits::ReadWrite.val(),
	//     0,
    // );
    // //  -> MTIMECMP
    // page::map(
	//     &mut root,
	//     0x0200_b000,
	//     0x0200_b000,
	//     page::EntryBits::ReadWrite.val(),
	//     0,
    // );
    // //  -> MTIME
    // page::map(
	//     &mut root,
	//     0x0200_c000,
	//     0x0200_c000,
	//     page::EntryBits::ReadWrite.val(),
	//     0,
    // );
    // // PLIC
    // id_map_range(
    //     &mut root,
    //     0x0c00_0000,
    //     0x0c00_2001,
    //     page::EntryBits::ReadWrite.val(),
    // );
    // id_map_range(
    //     &mut root,
    //     0x0c20_0000,
    //     0x0c20_8001,
    //     page::EntryBits::ReadWrite.val(),
    // );
    // // When we return from here, we'll go back to boot.S and switch into
    // // supervisor mode We will return the SATP register to be written when
    // // we return. root_u is the root page table's address. When stored into
    // // the SATP register, this is divided by 4 KiB (right shift by 12 bits).
    // // We enable the MMU by setting mode 8. Bits 63, 62, 61, 60 determine
    // // the mode.
    // // 0 = Bare (no translation)
    // // 8 = Sv39
    // // 9 = Sv48
    // // build_satp has these parameters: mode, asid, page table address.
    // let satp_value = cpu::build_satp(cpu::SatpMode::Sv39, 0, root_u);
    // unsafe {
    //     // We have to store the kernel's table. The tables will be moved
    //     // back and forth between the kernel's table and user
    //     // applicatons' tables. Note that we're writing the physical address
    //     // of the trap frame.
    //     cpu::mscratch_write(
    //         (&mut cpu::KERNEL_TRAP_FRAME[0]
    //             as *mut cpu::TrapFrame)
    //             as usize,
    //     );
    //     cpu::sscratch_write(cpu::mscratch_read());
    //     cpu::KERNEL_TRAP_FRAME[0].satp = satp_value;
    //     // Move the stack pointer to the very bottom. The stack is
    //     // actually in a non-mapped page. The stack is decrement-before
    //     // push and increment after pop. Therefore, the stack will be
    //     // allocated (decremented) before it is stored.
    //     cpu::KERNEL_TRAP_FRAME[0].trap_stack =
    //         page::zalloc(1).add(page::PAGE_SIZE);
    //     id_map_range(
    //         &mut root,
    //         cpu::KERNEL_TRAP_FRAME[0].trap_stack
    //             .sub(page::PAGE_SIZE,)
    //             as usize,
    //         cpu::KERNEL_TRAP_FRAME[0].trap_stack as usize,
    //         page::EntryBits::ReadWrite.val(),
    //     );
    //     // The trap frame itself is stored in the mscratch register.
    //     id_map_range(
    //         &mut root,
    //         cpu::mscratch_read(),
    //         cpu::mscratch_read()
    //             + core::mem::size_of::<cpu::TrapFrame,>(),
    //         page::EntryBits::ReadWrite.val(),
    //     );
    //     page::print_page_allocations();
    //     let p = cpu::KERNEL_TRAP_FRAME[0].trap_stack as usize - 1;
    //     let m = page::virt_to_phys(&root, p).unwrap_or(0);
    //     println!("Walk 0x{:x} = 0x{:x}", p, m);
    //     println!("KERNEL_TRAP_FRAME[0] = 0x{:x}", &mut cpu::KERNEL_TRAP_FRAME[0] as *mut cpu::TrapFrame as usize);
    // }
    // // The following shows how we're going to walk to translate a virtual
    // // address into a physical address. We will use this whenever a user
    // // space application requires services. Since the user space application
    // // only knows virtual addresses, we have to translate silently behind
    // // the scenes.
    // println!("Setting 0x{:x}", satp_value);
    // println!("Scratch reg = 0x{:x}", cpu::mscratch_read());
    // println!();
    //
    // // Set up virtio. This requires a working heap and page-grained allocator.
    // virtio::probe();
    // // This just tests the block device. We know that it connects backwards (8, 7, ..., 1).
    // let buffer = kmem::kmalloc(1024);
    // // Offset 1024 is the first block, which is the superblock. In the minix 3 file system, the first
    // // block is the "boot block", which in our case will be 0.
    // block::read(1, buffer, 512, 1024);
    // let mut i = 0;
    // loop {
    //     if i > 100_000_000 {
    //         break;
    //     }
    //     i += 1;
    // }
    // println!("Test hdd.dsk:");
    // unsafe {
    //     print!("  ");
    //     for i in 0..16 {
    //         print!("{:02x} ", buffer.add(i).read());
    //     }
    //     println!();
    //     print!("  ");
    //     for i in 0..16 {
    //         print!("{:02x} ", buffer.add(16+i).read());
    //     }
    //     println!();
    //     print!("  ");
    //     for i in 0..16 {
    //         print!("{:02x} ", buffer.add(32+i).read());
    //     }
    //     println!();
    //     print!("  ");
    //     for i in 0..16 {
    //         print!("{:02x} ", buffer.add(48+i).read());
    //     }
    //     println!();
    //     buffer.add(0).write(0xaa);
    //     buffer.add(1).write(0xbb);
    //     buffer.add(2).write(0x7a);
    // }
    // block::write(8, buffer, 512, 0);
    // // Free the testing buffer.
    // kmem::kfree(buffer);

    // cpu::satp_write(satp_value);
    // cpu::satp_fence_asid(0);

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    cpu::pmpaddr0_write(0x3fffffffffffff);
    cpu::pmpcfg0_write(0xf);

    unsafe {
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

#[no_mangle]
extern "C" fn kinit_hart(hartid: usize) {
    // All non-0 harts initialize here.
    unsafe {
        // We have to store the kernel's table. The tables will be moved
        // back and forth between the kernel's table and user
        // applicatons' tables.
        cpu::mscratch_write(
            (&mut cpu::KERNEL_TRAP_FRAME[hartid]
                as *mut cpu::TrapFrame)
                as usize,
        );
        // Copy the same mscratch over to the supervisor version of the
        // same register.
        cpu::sscratch_write(cpu::mscratch_read());
        cpu::KERNEL_TRAP_FRAME[hartid].hartid = hartid;
        // We can't do the following until zalloc() is locked, but we
        // don't have locks, yet :( cpu::KERNEL_TRAP_FRAME[hartid].satp
        // = cpu::KERNEL_TRAP_FRAME[0].satp;
        // cpu::KERNEL_TRAP_FRAME[hartid].trap_stack = page::zalloc(1);
    }
}

// #[no_mangle]
extern "C" fn kmain() {
    println!("=== Now in kmain() ===");
    // // kmain() starts in supervisor mode. So, we should have the trap
    // // vector setup and the MMU turned on when we get here.
    //
    // // Create a new scope so that we can test the global allocator and
    // // deallocator
    // {
    //     // We have the global allocator, so let's see if that works!
    //     let k = Box::<u32>::new(100);
    //     println!("Boxed value = {}", *k);
    //     // The following comes from the Rust documentation:
    //     // some bytes, in a vector
    //     let sparkle_heart = vec![240, 159, 146, 150];
    //     // We know these bytes are valid, so we'll use `unwrap()`.
    //     // This will MOVE the vector.
    //     let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
    //     println!("String = {}", sparkle_heart);
    //     println!("\nAllocations of a box, vector, and string");
    //     kmem::print_table();
    // }
    // println!("\nEverything should now be free:");
    // kmem::print_table();
    // println!();
    //
    // // If we get here, the Box, vec, and String should all be freed since
    // // they go out of scope. This calls their "Drop" trait.
    // println!("Interrupts are handled");
    // println!();
    //
    // // Let's set up the interrupt system via the PLIC. We have to set the threshold to something
    // // that won't mask all interrupts.
    // println!("Setting up interrupts and PLIC...");
    // // We lower the threshold wall so our interrupts can jump over it.
    // plic::set_threshold(0);
    // println!("Threshold lowered");
    // // VIRTIO = [1..8]
    // // UART0 = 10
    // // PCIE = [32..35]
    // // Enable the UART interrupt.
    // plic::enable(10);
    // println!("Interrupts enabled");
    // plic::set_priority(10, 1);
    // println!("Priority set");
    // println!("UART interrupts have been enabled and are awaiting your command");
    //
    // // rust_make_syscall(1);
    //
    // cpu::wait_forever()
}