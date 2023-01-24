# Volatile Ops

In the previous lesson we used the `write_volatile` method of some pointers.
There's also a `read_volatile` method to go with it as well.
These both perform a "volatile access".
Volatile memory accesses are special because they tell the compiler that a special side effect is going on.
It's a side effect that that compiler *does not* know about the specifics of.
This means that the compiler isn't free to alter the memory accesses themselves, because that would alter when the side effects happen too.

Let's have some examples of optimization affecting memory accesses:

```rust
fn do_thing(y: &mut i32) {
  *y = 6;
  *y = 7;
}
```

When we have code like this, the compiler will generally eliminate the step where 6 is written to `y`.
Instead, it will just write the value 7 to `y`.
That's normally great, because the best way to make a function fast is to cut down on the work it does.
However, if there was a side effect of the write, like if writing the value to a *special* address changed the hardware state via MMIO,
then losing some of our side effects would mean we lose some of our hardware state.

The same thing can be the case with reading memory as well:

```rust
fn loop_until_it_is_10(y: &mut i32) {
  while *y != 10 {}
}
```

With standard memory the compiler would be totally justified in making this function read `y` just once to check for 10, and then infinitely loop after that if it wasn't 10.
Because if it's not 10 the first time, it won't be 10 the second time through the loop, or the next, or the next.
Except, what if we had a *special* MMIO address that gives the CPU some sort of info about the outside state, like button state or display state.
If that were the case, you'd definitely need to actually read the value and check against 10 every time the programmer said to.

Not only can the compiler eliminate memory reads and writes, but it can also even *add* memory reads in some situations.
If there's a reference (that doesn't contain an `UnsafeCell` based type), the compiler is allowed to "speculatively" read from the reference.

```rust
fn might_read_ahead_of_time(y: &i32, z: i32) -> i32 {
  if complex_condition_check(z) {
    1 + *y
  } else {
    1 + z
  }
}
```

According to the code we wrote, `y` is only supposed to be read from if the check function returns `true`.
However, because `y` is a reference, the compiler is *allowed* to read from `y` before the check function is called.
Memory is extremely slow to access, and modern CPUs can perform other work while they wait for memory accesses to complete.
In many cases it's *great* if the compiler can start the read of `y` earlier and have that happen while the check computation is going.
If it turns out the check fails, then the read result can just be discarded anyway.
But, again, if there's MMIO side effects involved, then reading memory earlier than expected might do something bad.

So this is where "volatile" comes in.
Volatile means "please, just do it how I said to, trust me".

For all of our MMIO stuff:

1) We **don't** want to use references.
2) We **do** want to use volatile.

Unfortunately for us, the fundamental volatile read and write operations are defined for raw pointers.
Raw pointers are unsafe to use because anyone can just *make up* any raw pointer they want to at any time.
If you just pick a random pointer address and write some value to it, that could mess up anything at all.

We will be doing *a lot* of MMIO access, so we want as much of it as possible to all be done in safe code.
If we always work directly on raw pointers we'll have unsafe stuff spread everywhere.
It'll be super hard to tell where the actual dangerous parts of the program are.

And the thing is, most MMIO access is totally safe.
Writing a new backdrop color, or a new display control setting, can be done at any time.
There's a small number of cases where MMIO is not safe to do at any time, but those are rare.

So what we'll do is use the "Unsafe Creation, Then Safe Use" pattern.
Basically we'll make an alternate pointer type for our volatile access stuff.
It'll still be unsafe to make these volatile pointers, but once made they can be used as much as necessary from safe code.

## The `VolAddress` type

If you go use the [gba](https://docs.rs/gba) crate, it defines MMIO addresses using the [voladdress](https://docs.rs/voladdress) crate.
I promised that we'd write all our code from scratch, and we will do so, but we'll be keeping the design of our volatile pointer type very close to how the `voladdress` crate has it.
It's a lot less confusing to move between a tutorial and bigger "real projects" when the basic support layers are the same.
I wouldn't suggest that we use differently designed `Option` and `Result` types, so we should probably keep our `VolAddress` design similar as well.

Since the null address at 0 will never be a valid address for volatile reads and writes, let's wrap the [NonZeroUsize](https://doc.rust-lang.org/nightly/core/num/struct.NonZeroUsize.html) type.
That way `VolAddress` and `Option<VolAddress>` will both fit in a single CPU register.
We know that the address can point to different types of things, so we've got to make it a generic type.
Our first attempt might look like this:

```rust
// in voladdress.rs

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VolAddress<T> {
  address: usize,
}
```

but this doesn't work, we get a rather interesting error:

```
error[E0392]: parameter `T` is never used    
 --> src\voladdress.rs:8:23
  |
8 | pub struct VolAddress<T> {
  |                       ^ unused parameter
  |
  = help: consider removing `T`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `T` to be a const parameter, use `const T: usize` instead
```

If there's going to be a generic target type `T` then we need to use it somewhere in the fields of the struct.
It's just a rule for how things work.
There's no actual `T` value we're storing, so let's use that [PhantomData](https://doc.rust-lang.org/nightly/core/marker/struct.PhantomData.html) type it suggested.

```rust
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VolAddress<T> {
  address: NonZeroUsize,
  target: PhantomData<T>,
}
```

Now we've "used" the `T` in the `PhantomData`.
The `PhantomData` itself is a magical part of the language, it makes any generic parameter count as used without holding anything at all.

As I mentioned before, we'll have volatile addresses that are *usually* safe to read and write, but not always.
We can make extra generic parameters to help handle this too.

```rust
pub struct Safe;
pub struct Unsafe;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VolAddress<T, R, W> {
  address: NonZeroUsize,
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}
```

Now lets add a `read` method for our `VolAddress`:

```rust
impl<T, W> VolAddress<T, Safe, W>
where
  T: Copy,
{
  #[inline]
  pub fn read(self) -> T {
    unsafe { (self.address.get() as *const T).read_volatile() }
  }
}

impl<T, W> VolAddress<T, Unsafe, W>
where
  T: Copy,
{
  #[inline]
  pub unsafe fn read(self) -> T {
    (self.address.get() as *const T).read_volatile()
  }
}
```

For a moment it might seem like we're defining `read` twice.
However, note that the `R` parameter of the type is different for each impl block.

* If the address is always safe to read then we make `R` be `Safe`, and the first method will be available.
* If the address is unsafe to read we fill `R` with `Unsafe`, and the second method will be available instead.
* If the address shouldn't be read at all then `R` can be `()` and then there won't be a `read` method.

If that doesn't make sense right away that's okay.
When I first came up with the design it took me about 5 minutes to let it sink in that Rust would let me do it like this.
But, once you've understood it, I think it's a pretty great way to ensure the correct access is happening using the type system instead of runtime checks.

We can pull the same trick with `write` as well, using our `W` parameter:

```rust
impl<T, R> VolAddress<T, R, Safe>
where
  T: Copy,
{
  #[inline]
  pub fn write(self, t: T) {
    unsafe { (self.address.get() as *mut T).write_volatile(t) }
  }
}

impl<T, R> VolAddress<T, R, Unsafe>
where
  T: Copy,
{
  #[inline]
  pub unsafe fn write(self, t: T) {
    (self.address.get() as *mut T).write_volatile(t)
  }
}
```

The last thing we'll define right now is converting between `usize` values and `VolAddress` values:

```rust
impl<T, R, W> VolAddress<T, R, W> where T: Copy {
  #[inline]
  #[must_use]
  pub const unsafe fn new(address: usize) -> Self {
    assert!(address != 0);
    Self {
      address: NonZeroUsize::new_unchecked(address),
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData
    }
  }

  #[inline]
  #[must_use]
  pub const fn as_usize(self) -> usize {
    self.address.get()
  }
}
```

## Defining An MMIO Address

Now that we've got a basic working `VolAddress` type let's put it to use.
The two addresses we've seen so far are the backdrop color and the display control.
Defining the display control would involve a lot of bitwise ops stuff, so we'll put that off for later.

Let's handle the backdrop address by first defining a type for colors.

```rust
// New library module: color.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color(pub u16);
```

Any time we define a type for use with MMIO, we want to be sure it's a `#[repr(transparent)]` wrapper around one of the 1, 2, or 4 byte int types (`u8`, `i8`, `u16`, `i16`, `u32`, or `i32`).
That way the actual memory access can always happen as a single instruction.
If a type's size doesn't allow it to be accessed from memory as a single instruction then the compiler is free to just "do its best".
The code will do some instruction sequence that eventually gets a value, but with MMIO we want to have high confidence about exactly what's going on.
That's why we're sticking to only wrapping the native int types with our MMIO types.

Anyway, a `Color` is 5 bits per channel, and any bit pattern is valid, so we can make the field be `pub`.
We could have all sorts of other methods, but just the base definition is sufficient to define our backdrop address.
Let's try to keep our MMIO definitions in a single file if we can.

```rust
// New library module: mmio.rs

pub const BACKDROP_COLOR: VolAddress<Color, Safe, Safe> = unsafe { VolAddress::new(0x0500_0000) };
```

And since we're starting to have a lot of modules let's make a prelude for our library.

```rust
// New library module: prelude.rs

pub use crate::color::*;
pub use crate::mmio::*;
pub use crate::voladdress::*;
```

So hopefully our examples can just import the library prelude, instead of importing all sorts of individual little types.
Because we're gonna have a lot of types as we go.

And every time we add one of these library modules we need to update the parent module, which is `lib.rs` in this case

```rust
// in lib.rs
pub mod voladdress;
pub mod color;
pub mod mmio;
pub mod prelude;
```

From now on when there's new module files remember to declare the module in the parent, even if I don't say that part.

Okay, now that everything is all set in the library, we can make a new `min3.rs` example.

```rust
// New Example: min3.rs
#![no_std]
#![no_main]

use gba_from_scratch_rs::prelude::*;

#[no_mangle]
extern "C" fn main() -> ! {
  BACKDROP_COLOR.write(Color(0b11111));
  
  unsafe {
    (0x0400_0000 as *mut u16).write_volatile(0);
  }
  loop {}
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
  loop {}
}
```

And this new example... turns the screen red.
Exactly the same as the last example.
It might seem like we said a lot and didn't accomplish much, but we've moved part of the program out of the `unsafe` block.
It's always worth it when you can learn a bit and then move code out of an `unsafe` block and over to the safe portion of the program.
