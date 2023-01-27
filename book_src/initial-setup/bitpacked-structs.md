
# Bitpacked Structs

We've learned about using volatile to access MMIO, but that's just the storing and loading part.
The actual data that we store and load needs to be talked about too.
Most of the settings don't need an entire byte to express, so it saves space if we pack them together.
Say we have a `u16`, normally we'd think of it as being one number.
But there's 16 bits in there, and we don't need to treat them as one large group.
