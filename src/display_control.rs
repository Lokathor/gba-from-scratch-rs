#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DisplayControl(u16);
impl DisplayControl {
  #[inline]
  #[must_use]
  pub const fn new() -> Self {
    Self(0)
  }
  #[inline]
  #[must_use]
  pub const fn video_mode(self) -> u16 {
    self.0 & 0b111
  }
  #[inline]
  #[must_use]
  pub const fn with_video_mode(self, mode: u16) -> Self {
    assert!(mode < 6);
    Self((self.0 & !0b111) | mode)
  }
}
