
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

## Crate Inner Attributes

```rust
#![no_std]
#![no_main]
#![feature(naked_functions)]
```

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

## Panic Handler

```rust
#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
```

## Building an ELF and Fixing a ROM

todo
