use crate::register::Register;

/// # Time Out register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TimeOut(pub u32);
impl_boilerplate_for!(TimeOut);

impl TimeOut {
    pub const ADDR: u8 = 0x5C;

    // const TMOUT_OFFSET: u8 = 0;

    // const TMOUT_MASK: u32 = 0xffff;
}

impl core::fmt::Display for TimeOut {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimeOut").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TimeOut {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "TimeOut {{  }}",);
    }
}
