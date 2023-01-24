#![no_std]
#![no_main]

use gba_from_scratch_rs::prelude::*;

#[no_mangle]
extern "C" fn main() -> ! {
  BACKDROP_COLOR.write(Color(0b11111));
  
  unsafe {
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
