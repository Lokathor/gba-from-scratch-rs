
# Initial Setup

> If you want to make an apple pie from scratch, you must first invent the universe.

In other words, we've got a lot of setup ahead of us.

# System Tooling Setup

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

The `rust-src` component will be specific to Nightly or Stable, so be sure to set Nightly before installing it.
Otherwise you'll just have to install it a second time after setting Nightly for use.

The rustup commands should work just as well on Windows and Mac.
If you're on a Linux that isn't Ubuntu then the binutils package could be under some other name.

**Note:** `rustup default nightly` is a global command and will affect all your rust stuff.
If you want to sets nightly for *only* your GBA project the consider a [toolchain file](https://rust-lang.github.io/rustup/overrides.html?#the-toolchain-file) instead.

We'll also need a tool called `gbafix`, which can patch up the ROM's header data.
There's a [C version](https://github.com/devkitPro/gba-tools) of that if you want,
but there's also a rust one too, and you can just get the Rust version via `cargo install`.

```sh
cargo install gbafix
```

# Project Configuration

In addition to the system level setup, we'll want to set a few project configuration files before we begin working on the actual code.

## `Cargo.toml`

Running the usual `cargo init` will make a default `Cargo.toml` file.
This is mostly fine, but I strongly suggest that we alter the "dev" build profile slightly.
The dev profile is what's used for debug builds.
Normally the dev profile is set for a low level of optimization and using incremental building.
This lets debug builds complete quickly, though the resulting programs run an order of magnitude (or more) slower than a release build.
The GBA has a very weak CPU, so taking such a performance hit, even just in debug builds, is very bad for us.

We want to turn the `opt-level` up to 3, and turn `incremental` off.

Also, we can use a package override to turn `debug-assertions` off in our dependencies.
We won't have any normal dependencies, but we'll be using the `build-std` nightly feature of `cargo`.
When `cargo` builds the standard library it'll pull in `compiler_builtins`, and we want that to avoid those debug asserts.

```toml
[profile.dev]
opt-level = 3
incremental = false

[profile.dev.package."*"]
debug-assertions = false
```

## `.cargo/config.toml`

In our project folder make a `.cargo/` folder, then make a `cargo.toml` file inside.
This lets us change a few more cargo defaults so that we don't need to pass as many command line arguments all the time.

First of all, our default target for all of our builds will be `thumbv4t-none-eabi`.
There's two targets built in to `rustc` that could work on the GBA:

* `thumbv4t-none-eabi` produces "thumb code" by default
* `armv4t-none-eabi` produces "arm code" by default

We'll cover the thumb/arm differences later,
for now just know that we'll want a majority of our code to be thumb code so we'll be building with `thumbv4t-none-eabi`.
We can set this with `--target=thumbv4t-none-eabi`, but we can also use a `[build]` entry.

```toml
[build]
target = "thumbv4t-none-eabi"
```

The thing is, there's no standard library shipped for this target, it's only "[Tier 3](https://doc.rust-lang.org/rustc/target-tier-policy.html)".
This isn't a very big problem, because a Nightly feature of `cargo` lets us build it ourselves.
It makes a fresh project build take an extra few seconds, but that's it.
We could do this with `-Zbuild-std=core`, or we can use an `[unstable]` entry.

```toml
[unstable]
build-std = ["core"]
```

Last up is that we want to set some per-target details.
With a `runner` we can name a program that will run our programs.
This is designed to support emulators and simulators and the like, which is exactly what we want.
My GBA emulator of choice is [mGBA](https://mgba.io/) because it supports running ELF files directly.
This greatly simplifies the process of running our game in the emulator during development.
We set mGBA as the `runner`, then `cargo` passes our ELF formatted executable as an arg to mGBA, and things will "just work".
On Linux and Mac you'll get two executables when you install mGBA: `mgba` will have no GUI controls, and `mgba-qt` will have GUI controls.
On Windows there's just one executable: `mgba.exe`.
Personally, on my Windows machine I just made a copy of `mgba.exe` called `mgba-qt.exe` so that "mgba-qt" works all the time.

In addition to the `runner`, we need to set some special `rustflags`.
There's a special argument we need to pass to the linker.
We do this with `-Clink-arg=`, followed by the argument.
The linker argument itself that we need to pass will be to set the linker script.
We do this with `-T`, followed by the script's filename.
I'm going to suggest having a folder called `linker_scripts/`, and then a script called `mono_boot.ld`.
That means that our `[target.thumbv4t-none-eabi]` entry will look like this

```toml
[target.thumbv4t-none-eabi]
runner = "mgba-qt"
rustflags = ["-Clink-arg=-Tlinker_scripts/mono_boot.ld"]
```

The linker script file itself is complicated enough to deserve its own sub-header.

## `linker_scripts/mono_boot.ld`

In a folder called `linker_sctipts/` we want a file called `mono_boot.ld`.
This is a configuration file for the linker.

You're probably used to hearing about the compiler compiling your code.
Actually there's one other very important step.
The compiler makes one or more "object files".
Once all the object files are created the linker is called and it "links" those object files into the actual executable file.
The linker program is generally called `ld`, and newer linkers usually have a close name such as `lld`, `gold`, `mold`, etc.
The "ld" is short for, and I wish I were making this up, "Link eDitor".

We'll be using the linker from the ARM binutils.
It has a [sizable manual](https://sourceware.org/binutils/docs/ld/) that you can read yourself if you want.
Linker scripts *can* get quite complex, but we will have a fairly simple script.
We will also use just one linker script for most of this project.

The script is called `mono_boot.ld` because it's intended to work with the GBA's normal boot process where there's one cartridge per GBA.
It is also possible to make "multiboot" ROMs, which let one GBA send code to several others via the link cable, so everyone can play a game off of just one cartridge.
That's neat, but the downside is that all code and data for the download players has to fit into RAM only, which isn't much space at all.
We'll be sticking to making just normal ROMs for almost everything we make here, but might cover how to do multiboot eventually.

The actual content of the linker script is long enough that I won't paste it all in here.
Just get it out of the github repo: [mono_boot.ld](https://github.com/Lokathor/gba-from-scratch-rs/blob/main/linker_scripts/mono_boot.ld)
It's not essential that you fully understand everything about it, since you generally won't need to reconfigure the linker script.
That said, we'll briefly touch on each part of what it does.

### What The Linker Does

To make sense of what the linker script is doing we should probably first cover what the linker is trying to do.

The linker gets as input a number of "object files", which contain compiled code.
The code is sorted into a bunch of "sections".
Usually there's one section per function that was compiled.
Global variables and read-only data can also be among the sections.
These all make up the "input sections".

The linker's job is to re-arrange all of the input sections into the correct output sections.
Then an executable file is written to disk with all the data sorted correctly.

The linker script tells the linker how input sections are mapped to output sections.

For most common platforms there are default linker scripts.
There's a default way to link together a program for Windows or Android or things like that.
When your platform is more obscure, or if you want to do anything unusual, then you have to provide your own linker script.
Since the GBA is sufficiency obscure, we'll need to provide the linker script.

### Entry

The [Entry](https://sourceware.org/binutils/docs/ld/Entry-Point.html) is the name of the function execution should start at when the program begins.
The linker will store the address of the entry function we pick in the `e_entry` field of the program's ELF metadata when we compile an executable.

This doesn't affect anything at all if we run our program on actual hardware, the hardware doesn't even use the ELF format.
However, when we emulate the program in mGBA it will expect `e_entry` to be one of several possible addresses, and it rejects our program otherwise.
Right now we'll pick the name `__start` as the entry point function, and then later on we'll use other steps to make sure the `__start` function ends up where we want.
The name `__start` is just a conventional name to use, you could potentially use some other name if you really wanted to.

### Memory

The [Memory](https://sourceware.org/binutils/docs/ld/MEMORY.html) description tells the linker what the available memory of the GBA is like.
The names for each region are up to us, they just need to match the same names in out Sections description.
For each region we specify if the memory is readable, writable, and/or executable.
This affects where stuff is placed if none of the Sections rules covers an input section.
Also we give the base address of each memory region, as well as how many bytes big that region is.
The address allows the linker to hardcode the jumps from point to point within the program.
The size allows the linker to report an error if we put too much into a single memory region.

### Sections

Finally, the [Sections](https://sourceware.org/binutils/docs/ld/SECTIONS.html) information is the longest part of the file.
