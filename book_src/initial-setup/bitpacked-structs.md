
# Bitpacked Structs

We've learned about using volatile to access MMIO, but that's just the storing and loading part.
The actual data that we store and load needs to be talked about too.
Most of the settings don't need an entire byte to express, so it saves tons of space if we pack them together.
Say we have a `u16`, normally we'd think of it as being one number.
But there's 16 bits in there, and we don't need to treat them as a single large group.
We can divide up the bits so that different spans of bits will mean different things.
When data is packed within sub-spans of an integer's bits it's called "bitpacked".

Let's have an example.
We've mentioned the Display Control before, that it's at `0x0400_0000`.
If we check in [GBATEK](https://problemkaputt.de/gbatek.htm) (the usual place for GBA hardware info) we can find the display control summary:

```
4000000h - DISPCNT - LCD Control (Read/Write)
  Bit   Expl.
  0-2   BG Mode                (0-5=Video Mode 0-5, 6-7=Prohibited)
  3     Reserved / CGB Mode    (0=GBA, 1=CGB; can be set only by BIOS opcodes)
  4     Display Frame Select   (0-1=Frame 0-1) (for BG Modes 4,5 only)
  5     H-Blank Interval Free  (1=Allow access to OAM during H-Blank)
  6     OBJ Character VRAM Mapping (0=Two dimensional, 1=One dimensional)
  7     Forced Blank           (1=Allow FAST access to VRAM,Palette,OAM)
  8     Screen Display BG0  (0=Off, 1=On)
  9     Screen Display BG1  (0=Off, 1=On)
  10    Screen Display BG2  (0=Off, 1=On)
  11    Screen Display BG3  (0=Off, 1=On)
  12    Screen Display OBJ  (0=Off, 1=On)
  13    Window 0 Display Flag   (0=Off, 1=On)
  14    Window 1 Display Flag   (0=Off, 1=On)
  15    OBJ Window Display Flag (0=Off, 1=On)
```

For the Display Control, bits 0, 1, and 2 will combine to mean one value, the background mode.
Then bits 4 through 15 will each be a different boolean flag.

To model all this in Rust, we'll first make a "newtype".
This is the common pattern where we take one type of data and give it a `repr(transparent)` wrapping.
This lets us associate new methods and trait impls with the type while keeping the wrapped data fully compatible with foreign code.
Usually "foreign code" means C code, but in our case it means MMIO as well.

First we wrap a `u16` under a new name.
Let's have a new module named `display_control` and start it off with this struct:

```rust
// in display_control.rs
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DisplayControl(u16);
```

Now we want methods for getting and setting the video mode.
Also, we will need a method for making a starting value to run our getters and setters on.

```rust
impl DisplayControl {
  pub const fn new() -> Self {
    Self(0)
  }
  pub const fn video_mode(self) -> u16 {
    self.0 & 0b111
  }
  pub const fn with_video_mode(self, mode: u16) -> Self {
    assert!(mode < 6);
    Self((self.0 & !0b111) | mode)
  }
}
```

We can make a getter for any span of bits by doing a bitwise "and" using a bit mask with those bits.
For bits 0 through 2 that means `0b111`.

We can make a setter for any span of bits by first clearing the old value (`old & !mask`),
then merging the cleared value with the new value (`cleared | new`).

If the bit span doesn't start at 0 we also have to shift the output down, or input up, appropriately.
Here's the background control settings:

```
4000008h - BG0CNT - BG0 Control (R/W) (BG Modes 0,1 only)
400000Ah - BG1CNT - BG1 Control (R/W) (BG Modes 0,1 only)
400000Ch - BG2CNT - BG2 Control (R/W) (BG Modes 0,1,2 only)
400000Eh - BG3CNT - BG3 Control (R/W) (BG Modes 0,2 only)
  Bit   Expl.
  0-1   BG Priority           (0-3, 0=Highest)
  2-3   Character Base Block  (0-3, in units of 16 KBytes) (=BG Tile Data)
  4-5   Not used (must be zero) (except in NDS mode: MSBs of char base)
  6     Mosaic                (0=Disable, 1=Enable)
  7     Colors/Palettes       (0=16/16, 1=256/1)
  8-12  Screen Base Block     (0-31, in units of 2 KBytes) (=BG Map Data)
  13    BG0/BG1: Not used (except in NDS mode: Ext Palette Slot for BG0/BG1)
  13    BG2/BG3: Display Area Overflow (0=Transparent, 1=Wraparound)
  14-15 Screen Size (0-3)
```

In this case, the "character base block" and "screen base block" should be treated as numbers, so we need to shift the bits.
If we had a `BackgroundControl` type the methods for "character base block" might be like this:

```rust
impl BackgroundControl {
  pub const fn character_base_block(self) -> u16 {
    (self.0 & 0b1100) >> 2
  }
  pub const fn with_character_base_block(self, block: u16) -> Self {
    assert!(block < 4);
    Self((self.0 & !0b1100) | (block << 2))
  }
}
```

Once we've got a `DisplayControl` type defined, we also want to declare a VolAddress value for it, like we did with the backdrop color.
First we add the `display_control` module to our `prelude`, then we can make an entry using it in our `mmio` module.
The usual name for this address is "DISPCNT", so we'll go with that:

```rust
// mmio.rs
pub const DISPCNT: VolAddress<DisplayControl, Safe, Safe> =
  unsafe { VolAddress::new(0x0400_0000) };
```

From here, mapping out the rest of the GBA's MMIO "just" requires reading GBATEK and making all the necessary data types.
It can take a while, but that's about all there is to it.
In the actual `gba` crate, there's a lot of macros and helper functions to speed up the work and avoid any copy/paste typos.

Note that not all of the MMIO controls are going to be safe to read and/or write.
It's unsafe to use MMIO controls that can modify the memory that Rust sees or cause UB in any other way.
On the GBA that means the Direct Memory Access (DMA) units.
They're special hardware which can copy data around, and if you point them at the wrong spot they'll copy right over top of Rust's memory.
We'll cover them specifically in a future lesson.

As the end of the lesson, we can write a new example which cuts out our last `unsafe` block:

```rust
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
```

It still doesn't do anything but draw a plain red screen, but every time we cut down on `unsafe` it's still a win.
