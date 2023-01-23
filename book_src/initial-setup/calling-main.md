
# Calling `main`

A program that does nothing except not crash isn't much of a program.
Slightly more pressing is that defining `__start` within each and every example isn't a great way to do things.
First let's put the `__start` function in `lib.rs` so that all our examples will automatically get it when they link with our library.

```rust
// lib.rs
#![no_std]
#![feature(naked_functions)]

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
```

This is *mostly* the same as before.
The difference is that we've replaced the `1: b 1b` loop with a call to a function named `main`.

```arm
ldr r0, =main
bx r0
```

* `ldr reg, =symbol` is a "pseudo-instruction" that the assembler supports.
  The `ldr` instruction will "load-register".
  You can load an "immediate" value, which is a small value encoded within the instruction,
  or you can load from an address stored in a register.
  If you load from an address in a register you can also apply an offset.
  This is how pointers to struct fields work: a base pointer and then some offset based on the field's position within a struct.
  When we use `ldr reg, =symbol` the assembler will insert the address of `main` at the end of the function and then use the `pc` value (the program counter) as the base address to offset from.
  It may sound complicated, but all of the lookups and math are handled for us by the assembler and linker.
  We just write the  name of the symbol we want to end up in the register and it'll happen.
  We could also write any large constant that doesn't fit in an immediate this way and the assembler will help us out.
* `bx r0` is a "branch-exchange" to the address stored in `r0`.
  That's where we put the address of our `main` function, so the end of `__start` will branch-exchange to `main`.
  The "branch-exchange" is a special type of branch that lets the CPU switch between ARM code (a32) and Thumb code (t32).
  On the GBA's CPU it's the *only* way for user code to change code modes.
  Later ARM CPUs relaxed this restriction, but on the GBA we've got to use `bx`.
  Since `__start` is always a32 code, and the `main` function is almost always t32 code, we need to use `bx` and not just `b`.
  Also of note is that `b` branches to a *label*, while `bx` branch-exchanges to a *register*.
  Sometimes, like now, it's an extra step to load a function's address into a register when we want to use `bx`.

So now we have a `__start` function that will call a `main` function.
But where's `main`?
It's not in the library at all.
It might seem like a problem to call a function that doesn't exist yet, but it's okay.
As long as *somewhere* defines `main` when actually linking an executable, the linker will connect everything together just fine.
If we try to make an example that doesn't define a `main` we'll get a linker error and the build will fail.

Now let's have a look at our second example.
We'll call it `min2.rs`.

First, we need the example to link to our library.
Normally this would happen automatically when the example calls any functions in the library or uses any types from the library.
Right now the library is nearly empty though, there's nothing to call, nothing to `use`.
We can force the library to be linked in with an `extern crate` statement,

```rust
// in min2.rs
extern crate gba_from_scratch_rs;
```

This won't be necessary in the future, but for now it will get the job done.

Next we need that `main` function.

```rust
// in min2.rs
#[no_mangle]
extern "C" fn main() -> ! {
  unsafe {
    (0x0500_0000 as *mut u16).write_volatile(0b11111);
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}
```

Like happened with `__start`, we need `main` to be `#[no_mangle]`.
This will allow the library to link with it during the linking of the executable.
Also like with `__start`, the return type of `main` will be `-> !`, because we should never return from `main` back to `__start`.

The actual code in the body of `main` is two volatile writes.
These are to Memory-mapped IO (MMIO) addresses, which are how the CPU controls the rest of the hardware.
Most of learning to use the GBA well boils down to learning what all the MMIO controls allow for.

The first `write_volatile` call sets the color of the backdrop, the color that's shown for a pixel when nothing else is shown there.
Color values on the GBA are 5 bits per channel, red as the lowest bits, then green, then blue.

The second `write_volatile` sets the "display control" to 0.
When the BIOS first transfers control to our program, the display control is set with the "forced blank" bit on.
This is why the `min1` example was all white, because display was forced to be "blank".
When we write 0 to the display control this disables the forced blank bit, allowing normal video display.
Since we haven't set anything else anywhere, the whole screen will be the backdrop color (which we set to red).

This means that our entire example does nothing but turn the screen red.
Still not very interactive, but it's some progress.
