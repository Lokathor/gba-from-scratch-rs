
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
My GBA emulator of choice is [mGBA](https://mgba.io/), and `0.10` is the latest version of mGBA as I write this.
On Linux and Mac you'll get two executables: `mgba` will have no GUI controls, and `mgba-qt` will have GUI controls.
On Windows there's just one executable: `mgba.exe`.
Personally, on my Windows machine I just made a copy of `mgba.exe` called `mgba-qt.exe`.
Then I didn't have to worry about the per-OS difference, I just set the runner as "mgba-qt" and it works.

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

todo
