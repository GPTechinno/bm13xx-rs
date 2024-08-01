use crate::register::Register;

/// # Returned Group Pattern Status register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ReturnedGroupPatternStatus(pub u32);
impl_boilerplate_for!(ReturnedGroupPatternStatus);

impl ReturnedGroupPatternStatus {
    pub const ADDR: u8 = 0x98;

    // const RGPS3_OFFSET: u8 = 24;
    // const RGPS2_OFFSET: u8 = 16;
    // const RGPS1_OFFSET: u8 = 8;
    // const RGPS0_OFFSET: u8 = 0;

    // const RGPS3_MASK: u32 = 0b1111;
    // const RGPS2_MASK: u32 = 0b1111;
    // const RGPS1_MASK: u32 = 0b1111;
    // const RGPS0_MASK: u32 = 0b1111;
}

impl core::fmt::Display for ReturnedGroupPatternStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReturnedGroupPatternStatus").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ReturnedGroupPatternStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ReturnedGroupPatternStatus {{  }}",);
    }
}
