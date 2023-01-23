#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate gba_from_scratch_rs;

#[no_mangle]
extern "C" fn main() -> ! {
  unsafe {
    (0x0500_0000 as *mut u16).write_volatile(0b11111);
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
