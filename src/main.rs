// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(pointer_is_aligned_to)]
#![feature(ptr_mask)]
#![feature(sync_unsafe_cell)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(not(any(test, clippy)), no_std)]
#![cfg_attr(not(test), no_main)]
#![forbid(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod allocator;
mod bldb;
mod clock;
mod cons;
mod cpio;
mod cpuid;
mod gpio;
mod idt;
mod io;
mod iomux;
mod loader;
mod mem;
mod mmu;
mod pci;
mod ramdisk;
mod repl;
mod result;
mod smn;
mod uart;
mod ufs;

/// The main entry point, called from assembler.
#[unsafe(no_mangle)]
pub(crate) extern "C" fn entry(config: &mut bldb::Config) {
    println!();
    println!("Oxide Boot Loader/Debugger");
    println!("{config:#x?}");
    repl::run(config);
    panic!("main returning");
}

mod no_std {
    #[cfg(not(any(test, clippy)))]
    #[panic_handler]
    pub fn panic(info: &core::panic::PanicInfo) -> ! {
        crate::println!("Panic: {:#?}", info);
        unsafe {
            crate::bldb::dnr();
        }
    }
}
#[cfg(test)]
mod fakes;
