use core::{marker::PhantomData, num::NonZeroUsize};

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
