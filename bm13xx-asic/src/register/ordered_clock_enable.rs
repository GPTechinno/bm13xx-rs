use crate::register::Register;

/// # Ordered Clock Enable register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OrderedClockEnable(pub u32);
impl_boilerplate_for!(OrderedClockEnable);

impl OrderedClockEnable {
    pub const ADDR: u8 = 0x20;

    // const CLKEN_OFFSET: u8 = 0;

    // const CLKEN_MASK: u32 = 0xffff;
}

impl core::fmt::Display for OrderedClockEnable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrderedClockEnable").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for OrderedClockEnable {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "OrderedClockEnable {{  }}",);
    }
}
