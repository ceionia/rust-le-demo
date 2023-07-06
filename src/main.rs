#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

use core::arch::asm;

extern crate alloc;

mod dpmi;
mod dpmi_alloc;
mod panic;

fn main() {
    println!("Hello, world!");
    debug_trap!();
    println!("Done.");
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! debug_trap {
    () => { unsafe { asm!("int 0x1"); } };
}

#[no_mangle]
pub extern "C" fn start() {
    unsafe { asm!(
        "push ds",
        "pop es",
    ); }
    main();
    dpmi::dpmi_exit();
}
