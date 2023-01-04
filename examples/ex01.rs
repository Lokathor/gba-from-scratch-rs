#![no_std]
#![no_main]
#![feature(naked_functions)]

#[naked]
#[no_mangle]
#[link_section = ".text.gba_rom_header"]
unsafe extern "C" fn __start() -> ! {
  core::arch::asm! {
    // place space for the header
    ".space 0xE0",

    // call to `main`
    "ldr r0, =main",
    "bx r0",
    options(noreturn)
  }
}

#[no_mangle]
extern "C" fn main() -> ! {
  unsafe {
    (0x0500_0000 as *mut u16).write_volatile(0b11111);
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
