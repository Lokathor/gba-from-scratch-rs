#![no_std]
#![no_main]
#![feature(naked_functions)]

#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".text.gba_rom_header"]
unsafe extern "C" fn __start() -> ! {
  core::arch::asm! {
    "b 1f",
    ".space 0xE0",
    "1:",
    "ldr r0, =main",
    "bx r0",
    options(noreturn)
  }
}

#[no_mangle]
extern "C" fn main() -> ! {
  unsafe {
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
