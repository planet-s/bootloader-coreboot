#![no_std]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(naked_functions)]

#[macro_use]
extern crate bitflags;
extern crate coreboot_table;
extern crate spin;
extern crate syscall;

use core::slice;

use coreboot_table::{Mapper, PhysicalAddress, VirtualAddress, Table};

#[macro_use]
pub mod arch;

pub mod devices;
pub mod externs;
pub mod panic;

struct IdentityMapper;

impl Mapper for IdentityMapper {
    unsafe fn map_aligned(&mut self, address: PhysicalAddress, _size: usize) -> Result<VirtualAddress, &'static str> {
        Ok(VirtualAddress(address.0))
    }

    unsafe fn unmap_aligned(&mut self, _address: VirtualAddress) -> Result<(), &'static str> {
        Ok(())
    }

    fn page_size(&self) -> usize {
        4096
    }
}

#[naked]
#[no_mangle]
pub unsafe fn kstart() -> ! {
    asm!("
        cli
        cld
        mov esp, 0x7000
    " : : : : "intel", "volatile");
    kmain()
}

pub fn kmain() -> ! {
    println!("Test");

    let mut framebuffer_opt = None;

    let mut mapper = IdentityMapper;
    coreboot_table::tables(|table| {
        match table {
            Table::Framebuffer(framebuffer) => {
                println!("{:?}", framebuffer);
                framebuffer_opt = Some(framebuffer.clone());
            },
            Table::Memory(memory) => println!("{:?}", memory.ranges()),
            Table::Other(other) => println!("{:?}", other),
        }
        Ok(())
    }, &mut mapper).unwrap();

    if let Some(framebuffer) = framebuffer_opt {
        if framebuffer.bits_per_pixel == 32 {
            let x = framebuffer.x_resolution;
            let y = framebuffer.y_resolution;

            println!("Framebuffer of resolution {}x{}", x, y);

            let size = framebuffer.bytes_per_line as usize * y as usize;

            let address = unsafe {
                mapper.map(
                    PhysicalAddress(framebuffer.physical_address as usize),
                    size
                ).unwrap()
            };

            let buf = unsafe {
                slice::from_raw_parts_mut(
                    address.0 as *mut u8,
                    size
                )
            };

            for i in 0..buf.len() {
                buf[i] = (i % 256) as u8;
            }
        } else {
            println!("Unsupported framebuffer bits per pixel {}", framebuffer.bits_per_pixel);
        }
    }

    println!("Halt");

    loop {}
}
