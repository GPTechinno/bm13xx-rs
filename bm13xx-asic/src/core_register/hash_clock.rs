use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCtrl(pub u8);
impl_boilerplate_for_core_reg!(HashClockCtrl);

impl HashClockCtrl {
    pub const ID: u8 = 0x05;

    // const CLOCK_CTRL_OFFSET: u8 = 0;

    // const CLOCK_CTRL_MASK: u8 = 0xff;
}

impl ::core::fmt::Display for HashClockCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashClockCtrl").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashClockCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HashClockCtrl {{ }}",);
    }
}

/// # Hash Clock Counter core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCounter(pub u8);
impl_boilerplate_for_core_reg!(HashClockCounter);

impl HashClockCounter {
    pub const ID: u8 = 0x06;

    // const CLOCK_CNT_OFFSET: u8 = 0;

    // const CLOCK_CNT_MASK: u8 = 0xff;
}

impl ::core::fmt::Display for HashClockCounter {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashClockCounter").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashClockCounter {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HashClockCounter {{ }}",);
    }
}
