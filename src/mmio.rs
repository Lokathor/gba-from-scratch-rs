use crate::prelude::*;

pub const DISPCNT: VolAddress<DisplayControl, Safe, Safe> =
  unsafe { VolAddress::new(0x0400_0000) };

pub const BACKDROP_COLOR: VolAddress<Color, Safe, Safe> =
  unsafe { VolAddress::new(0x0500_0000) };
