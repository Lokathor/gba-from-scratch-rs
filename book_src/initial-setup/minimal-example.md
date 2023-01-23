
# Minimal Example

The most minimal example we could start with doesn't do anything except avoid crashing.
Still, it's a useful way to check that our tools and project are configured properly.

```rust
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
    "b 1b",
    options(noreturn)
  }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
```

You can put this in your project's `examples/` directory.
We'll call it `min1.rs`, and `cargo run --example min1` should build the example and run it in your runner.
If everything went fine, mGBA should open up and show a white screen and then do nothing, without any error messages.

If there's any problems already that's okay!
There's a lot of small details involved in something like this.
When I was setting things up for this book, and trying to write this minimal example, I messed it up at least three times.
Don't get discouraged too easily!
Double check all the file names, all the file contents, what you typed in the command line, and so on.
You can [open an issue](https://github.com/Lokathor/gba-from-scratch-rs/issues) if you need to ask for help.
Alternately, you can try asking in the [GBA Dev Discord](https://discord.io/gbadev).

Let's cover each part of the example one at a time.

## Crate Inner Attributes

```rust
#![no_std]
#![no_main]
#![feature(naked_functions)]
```

First we've got several "inner" attributes.
They're like normal attributes but they go just inside of what they affect, rather than just outside of it.
There's no way to have text "outside" of a file, so we have to use inner attributes here.
Inner attributes always have to go first, before any other items like functions or structs or anything like that.
The ordering of the attributes themselves doesn't matter,
I just like sorting them from shortest to longest because it usually looks nice.

First is [no_std](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute).
This prevents the compiler from automatically linking our program with the `std` and `alloc` crates.
The GBA doesn't support `std` at all, so we'd get a build error if we didn't have `no_std`.
It's possible to use `alloc`, but we'd have to do more setup to make it work, so for now we can't.
Even without `std` available, we can still use a fairly good amount of Rust through the `core` crate.

Next is the [no_main](https://doc.rust-lang.org/reference/crates-and-source-files.html#the-no_main-attribute) attribute.
Normally Rust would provide the actual `main` symbol which the OS calls at the start of the process.
That function does some environment setup and then calls the main function of the Rust code we wrote.
The compiler doesn't know how to write a `main` for the GBA though.
So, we just tell it to not even try, and we'll handle that ourselves.

Finally, we have a Nightly feature: [naked functions](https://github.com/rust-lang/rust/issues/90957).
Normally a function has the "prologue" and "epilogue" (the intro and outro) handled by the compiler.
This is for the best, because among other things it's what allows inlining to work.
However, on rare occasions we'll need to have the compiler step back and let us take complete control.
With the `naked_functions` Nightly feature, we'll be able to use `#[naked]` on a function when needed.

## Entry Point

```rust
#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".text.gba_rom_header"]
unsafe extern "C" fn __start() -> ! {
  core::arch::asm! {
    "b 1f",
    ".space 0xE0",
    "1:",
    "b 1b",
    options(noreturn)
  }
}
```

This is our `__start` function.
Normally we'd want to define one in a library, but for now it's fine to define it right in the example.

We call it `__start` and use `#[no_mangle]` to match the `ENTRY` part of our linker script.
Normally Rust would apply name mangling to a function's name so that its symbol name doesn't clash with any other (because name clashes cause a build error).
Using `#[no_mangle]` stops the name mangling from happening on a particular function or static (but we should limit how much we use it, to avoid those build errors).
It's the same idea as how `#[naked]` changes the (usually useful) default option into a weirder option.
We also use `#[link_section=]` to put this function into the ".text.gba_rom_header" section (also because of the linker script).
Section names can be alphanumeric, and they also allow underscores and periods.
The linker script will place the ".text.gba_rom_header" *input* section at the very start of the ".text" *output* section during linking.
That means that we'll end up with our `__start` function, and its header data, at the very start of the ROM.

The last attribute, `#[instruction_set(arm::a32)]`, sets the function to be encoded as `a32` instructions.
Because we're building using the `thumbv4t-none-eabi` target, all functions will *default* to being `t32`.
The `instruction_set` attribute lets us override the default and have an `a32` function.
Normally our functions can be `a32` or `t32`, and `t32` tends to have better performance by default.
Because of details about the GBA's boot up process (which we'll talk about below), the `__start` function always has to be `a32` specifically.

With all of the attributes out of the way, we get to declare the function itself: `unsafe extern "C" fn __start() -> !`
We're gonna mark `__start` as `unsafe` because honestly no one should be calling it from Rust.
Our expectation is that only the BIOS itself will call `__start`, and only the once at boot.
So we'll just call the fn `unsafe` and say no one else can call it.
Similarly, because the BIOS will effectively be calling the function, we declare that the function has the `extern "C"` ABI.
It's Undefined Behavior (UB) for any code other than a Rust function to call a Rust ABI function, so we have to use the C ABI for `__start`.
It's just the rules.
Finally, we should never return from `__start`, so we put down the return type as `!`.
Actually this ends up being just a hint to future readers, because the compiler can't check the body of an inline assembly block.
Still, it's probably a useful hint, and I'm sure that future readers of the code (which might be ourselves!) will appreciate our guidance.

The body of a `#[naked]` function has to be a single [asm!](https://doc.rust-lang.org/reference/inline-assembly.html) block.
Assembly blocks can be multi-line string literals, or they can be a list of string literals (the list is joined with newlines during compilation).
When we put the lines together the assembly looks like this:

```arm
b 1f
.space 0xE0
1:
b 1b
```

* The first line is a "branch" (`b`) to the `1` label that's "forward" from this instruction (`1f`). In other words, the label on the third line of our asm block, where it says `1:`.
* The second line is a directive to add blank space to the output. We can tell it's a directive because it starts with a dot. The `.space` directive just adds blank space to the program. In this case, 0xE0 (224 decimal) bytes of blank space goes right after the branch instruction. That's where the header data goes. For initial development with an emulator it's fine to have a blank header, so more on header stuff will come later. Blank space isn't useful code though, which is why the previous line jumps over the blank space to the label.
* The third line `1:` is a label. We can tell it's a label because it ends with a `:` character. Within an `asm!` block you should only use numeric labels, which are always reusable without clashing. If you use `global_asm!`, or if you write an external assembly file, then you can also use alphanumeric labels if you're careful about which labels you export or not. A label marks the next instruction, either on the same line or on a future line. There can be more than one label pointing to the same instruction, and there can be blank lines between the label and the instruction. The number `1` itself isn't significant, we could pick any (positive, small-ish) number we like.
* The fourth line is a branch to the `1` label that is "back" from this instruction (`1b`). This jumps to the label on line 3. That label refers to this instruction. So this instruction just jumps to this instruction, over and over. It's like writing `loop {}`, just the assembly version of it.

In summary: our assembly block just jumps into an infinite loop.
It's not very exciting, but the emulator at least won't crash by trying to execute a blank part of the ROM or whatever.

## Panic Handler

```rust
#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
```

Even if the program itself can't actually panic, Rust demands that you define a panic handler function.
During compilation a panic handler that never gets called will be removed, the same as any function that's never called gets removed, but we have to define and then let the compiler decide for itself that it's unused.
As our programs grow we might want to have a panic handler that does something with the panic info.
Send a message to mGBA's debug output, or show something on the screen, or something like that.
Right now we'll start with a "do nothing at all" panic handle though.

## Why Does This Work?

The GBA has an ARM7TDMI CPU.
The ARM7TDMI uses the ARMv4T CPU architecture.
When the CPU turns on, the `pc` register ("program counter") is reset to 0, and the CPU begins executing address 0.
This part is just how any ARM CPU works, it's not GBA specific.

Address 0 on the GBA is in the BIOS memory.
That's the "Basic Input Output System".
The GBA's BIOS data is built into the device itself, separate from whichever cartridge you insert.
The BIOS has the code to play that [startup animation](https://www.youtube.com/watch?v=6_ZD3FxMcvQ), you know the one, with the nice sound.
It also checks the ROM's header data, and will lock the system if the header is incorrect.
We'll talk about header stuff more in a moment, but mGBA doesn't do the header check so for now it's fine to have a blank header.

After the BIOS has done all of its work it will branch to the start of the ROM, `0x0800_0000`.
That's where the linker script puts the start of the ".text" section.
And at the *very start* of the ".text" section will be the ".text.gba_rom_header" section.
The Rust compiler won't ever pick that for any code on its own, and we've only assigned one thing to use that section: our `__start` function.
So our `__start` function is going to be at the very very start of the ROM.
When the BIOS branches to the start of the ROM, it's branching to the `__start` function.
Then all the stuff we talked about up above with the four lines of assembly will happen.

And that's the basic idea of how this is working so far.
I still haven't explained the full story on the `instruction_set` and `a32` stuff,
but we'll save that for the next example.
For now, you can already call yourself a beginner GBA programmer!
