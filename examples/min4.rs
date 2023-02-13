#![no_std]
#![no_main]

use gba_from_scratch_rs::prelude::*;

#[no_mangle]
extern "C" fn main() -> ! {
  BACKDROP_COLOR.write(Color(0b11111));

  DISPCNT.write(DisplayControl::new());
  loop {}
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
