
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
Normally Rust would apply name mangling to a function's name so that its symbol name doesn't clash with any other (which would cause a build error).
Using `#[no_mangle]` stops the name mangling from happening on a particular function or static.
It's the same idea as how `#[naked]` changes the (usually useful) default option into a weirder option.
We also use `#[link_section=]` to put this function into the ".text.gba_rom_header" section.
Section names can be alphanumeric, and also underscores and periods.
The exact name we're using is because of what the linker script is expecting.
The linker script will place the ".text.gba_rom_header" *input* section at the very start of the ".text" *output* section during linking.
That means that we'll end up with our `__start` function, and its header data, at the very start of the ROM.

The last attribute, `#[instruction_set(arm::a32)]`, sets the function to be encoded as `a32` instructions.
Because we're building using the `thumbv4t-none-eabi` target, all functions will *default* to being `t32`.
The `instruction_set` attribute lets us override the default and have an `a32` function.
Normally our functions can be `a32` or `t32`, and `t32` tends to have better performance by default.
Because of details about the GBA's boot up process, the `__start` function always has to be `a32` specifically.
We can cover the boot up more later on.

With all of the attributes out of the way, we get to declare the function itself: `unsafe extern "C" fn __start() -> !`
We're gonna mark `__start` as `unsafe` because no one should be calling it from Rust.
Our expectation is that only the BIOS itself will call `__start`, so we'll just call the fn `unsafe` and say no one else can call it.
Similarly, because the BIOS will effectively be calling the function, we declare that the function has the `extern "C"` ABI.
It's Undefined Behavior (UB) for external code to call a Rust ABI function, so we have to use the C ABI.
It's just the rules.
Finally, we never return from `__start`, so we put down the return type as `!`.
Because the function is a `#[naked]` function the compiler can't *actually* check that we never return from the function.
Still, it serves as a mild reminder of our intent, to other programmers, or to our future selves.

The body of a `#[naked]` function has to be a single [asm!](https://doc.rust-lang.org/reference/inline-assembly.html) block.
In this case, we're writing ARM assembly:

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

Long story short: our assembly block just jumps into an infinite loop.
It's not very exciting, but the emulator at least won't crash bt trying to execute a blank part of the ROM or anything.

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

## Finished

That all there is to it!
You're a beginner GBA programmer!
