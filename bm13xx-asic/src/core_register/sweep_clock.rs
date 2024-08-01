use crate::core_register::CoreRegister;

/// # Sweep Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SweepClockCtrl(pub u8);
impl_boilerplate_for_core_reg!(SweepClockCtrl);

impl SweepClockCtrl {
    pub const ID: u8 = 7;

    // const SWPF_MODE_OFFSET: u8 = 7;
    // const CLK_SEL_OFFSET: u8 = 0;

    // const SWPF_MODE_MASK: u8 = 0b1;
    // const CLK_SEL_MASK: u8 = 0b1111;
}

impl ::core::fmt::Display for SweepClockCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("SweepClockCtrl").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for SweepClockCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "SweepClockCtrl {{ }}",);
    }
}
