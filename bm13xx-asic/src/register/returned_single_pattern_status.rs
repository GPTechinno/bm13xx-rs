use crate::register::Register;

/// # Returned Single Pattern Status register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ReturnedSinglePatternStatus(pub u32);
impl_boilerplate_for!(ReturnedSinglePatternStatus);

impl ReturnedSinglePatternStatus {
    pub const ADDR: u8 = 0xA0;

    // const RSPS_OFFSET: u8 = 0;

    // const RSPS_MASK: u32 = 0xffff_ffff;
}

impl core::fmt::Display for ReturnedSinglePatternStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReturnedSinglePatternStatus").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ReturnedSinglePatternStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ReturnedSinglePatternStatus {{  }}",);
    }
}
