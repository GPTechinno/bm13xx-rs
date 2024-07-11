use crate::register::Register;

/// # PLL Divider registers
///
/// Used to set PLL parameters.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL0Divider(pub u32);
impl_boilerplate_for!(PLL0Divider);

impl PLL0Divider {
    pub const ADDR: u8 = 0x70;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL1Divider(pub u32);
impl_boilerplate_for!(PLL1Divider);

impl PLL1Divider {
    pub const ADDR: u8 = 0x74;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL2Divider(pub u32);
impl_boilerplate_for!(PLL2Divider);

impl PLL2Divider {
    pub const ADDR: u8 = 0x78;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL3Divider(pub u32);
impl_boilerplate_for!(PLL3Divider);

impl PLL3Divider {
    pub const ADDR: u8 = 0x7C;
}

trait PLLDividerRegister: Register {}

impl dyn PLLDividerRegister {
    // const PLLDIV3_OFFSET: u8 = 24;
    // const PLLDIV2_OFFSET: u8 = 16;
    // const PLLDIV1_OFFSET: u8 = 8;
    // const PLLDIV0_OFFSET: u8 = 0;

    // const PLLDIV3_MASK: u32 = 0b1111;
    // const PLLDIV2_MASK: u32 = 0b1111;
    // const PLLDIV1_MASK: u32 = 0b1111;
    // const PLLDIV0_MASK: u32 = 0b1111;
}

impl core::fmt::Display for dyn PLLDividerRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PLLDivider").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for dyn PLLDividerRegister {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "PLLDivider {{  }}",);
    }
}

impl PLLDividerRegister for PLL0Divider {}
impl PLLDividerRegister for PLL1Divider {}
impl PLLDividerRegister for PLL2Divider {}
impl PLLDividerRegister for PLL3Divider {}
