
# Initial Setup

> If you want to make an apple pie from scratch, you must first invent the universe.

In other words, we've got a lot of setup ahead of us.

## System Tooling Setup

Separately from setting up a specific project's configuration, you'll need to do a few "one time" things to make your machine ready.

First of all, we'll need to be using the Nightly rust compiler, because we'll be using a lot of unstable features.

Next, we'll need to have the `rust-src` rustup component available, because we'll have to build our own copy of the standard library.

Finally, we'll need the ARM binutils because LLVM's linker doesn't currently support the GBA's old CPU (but support is in the works).
You can get the ARM binutils for Windows and Mac from the [ARM Developer Website](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads).
If you're on Linux then you can probably find them in your package manager.
We want a set of binutils for `arm-none-eabi`.

Here's the lines to make the (ubuntu based) CI run:

```sh
rustup default nightly
rustup component add rust-src
sudo apt-get install -y binutils-arm-none-eabi
```

The rustup commands should work just as well on Windows and Mac.
If you're on a Linux that isn't ubuntu then the binutils package could be under some other name.

**Note:** `rustup default nightly` is a global command and will affect all your rust stuff.
If you want to sets nightly for *only* your GBA project the consider a [toolchain file](https://rust-lang.github.io/rustup/overrides.html?#the-toolchain-file) instead.

We'll also need a tool called `gbafix`, which can patch up the ROM's header data.
There's a [C version](https://github.com/devkitPro/gba-tools) of that if you want,
but there's also a rust one too, and you can just get the Rust version via `cargo install`.

```sh
cargo install gbafix
```
