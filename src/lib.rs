#![no_std]
#![feature(naked_functions)]

pub mod voladdress;
pub mod color;
pub mod mmio;
pub mod prelude;

#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".text.gba_rom_header"]
unsafe extern "C" fn __start() -> ! {
  core::arch::asm! {
    // jump over the header data itself
    "b 1f",
    ".space 0xE0",
    "1:",

    // call to `main`
    "ldr r0, =main",
    "bx r0",
    options(noreturn)
  }
}
