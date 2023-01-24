use crate::prelude::*;

pub const BACKDROP_COLOR: VolAddress<Color, Safe, Safe> = unsafe { VolAddress::new(0x0500_0000) };
