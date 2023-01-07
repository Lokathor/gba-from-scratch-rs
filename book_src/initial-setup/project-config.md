
# Project Configuration

We'll need to have a few more files than just a `cargo init` gives us.

## `.cargo/config.toml`

First we'll need to configure `cargo` itself a little bit.
We have to make a `.cargo/` folder at the project root, and put a `config.toml` inside.
We fill it out like this:

```toml
[build]
target = "thumbv4t-none-eabi"

[unstable]
build-std = ["core"]

[target.thumbv4t-none-eabi]
runner = "mgba-qt"
rustflags = ["-Clink-arg=-Tlinker_scripts/mono_boot.ld"]
```

The `target` key sets the default `--target` to use when we don't specify one ourselves.
This lets us build with just `cargo build` instead of having to write `cargo build --target thumbv4t-none-eabi` every time.

There's two targets built in to `rustc` that could work on the GBA:

* `thumbv4t-none-eabi` produces "thumb code" by default
* `armv4t-none-eabi` produces "arm code" by default

We'll cover the thumb/arm differences later,
for now just know that we'll want a majority of our code to be thumb code so we'll be building with `thumbv4t-none-eabi`.

The `build-std` key tells cargo that we'll need it to build the standard library (specifically the `core` crate) when we compile.
This is needed because the `thumbv4t-none-eabi` target is only "[Tier 3](https://doc.rust-lang.org/rustc/target-tier-policy.html)".
That means that `rustc` has a description file for how the target works, but `rustup` doesn't ship a pre-built copy of the standard library for the target.
It takes a few extra seconds to build the `core` crate ourselves, though once it's been done we won't notice the difference otherwise.

The `runner` key sets a binary that will run our programs when we use `cargo run`.
Here I'm setting "mgba-qt" because I've found that [mGBA](https://mgba.io/) is a pretty good emulator to develop with.
On Mac and Linux there's two versions of mGBA, a bare-bones command line version "mgba" and a Qt based version with GUI controls "mgba-qt".
On Windows there's just one version called "mgba.exe".
I actually use *all three* operating systems to develop on often enough, so what I've done on my Windows machine is make a copy of "mgba.exe" called "mgba-qt.exe",
and then I don't have to ever change the cargo setting.

The `rustflags` key adds to the `RUSTFLAGS` environment variable.
In this case, `-Clink-arg=` defines something that's passed to the linker, and `-Tlinker_scripts/mono_boot.ld` is what the linker sees.
The linker script tells the linker how to create a binary after all the code is compiled.
It's a complicated enough subject that it'll get its own subsection.

## `Cargo.toml`

Next, we'll want to tune the `dev` build profile a bit.
This is what's used for "debug" builds.
We need to turn the optimization all the way on even for our debug builds, because the GBA has a very slow CPU compared to what a Desktop does.
We'll also turn off debug-assertions for packages we depend on, which doesn't affect `core`, but does affect 

```toml
[profile.dev]
opt-level = 3
incremental = false

[profile.dev.package."*"]
debug-assertions = false
```

## `linker_scripts/mono_boot.ld`

todo
