use crate::register::Register;

/// # Version Rolling register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct VersionRolling(pub u32);
impl_boilerplate_for!(VersionRolling);

impl VersionRolling {
    pub const ADDR: u8 = 0xA4;

    const EN_OFFSET: u8 = 31;
    const VERS_MASK_OFFSET: u8 = 0;

    const EN_MASK: u32 = 0x1;
    const VERS_MASK_MASK: u32 = 0xffff;

    /// ## Get the Version Rolling enabled state.
    ///
    /// This returns an `bool` with the Version Rolling enabled state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::VersionRolling;
    ///
    /// let vers_roll = VersionRolling(0x0000_FFFF);
    // assert!(!vers_roll.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        (self.0 >> Self::EN_OFFSET) & Self::EN_MASK == Self::EN_MASK
    }

    /// ## Get the chip identifier.
    ///
    /// This returns an `u32` with the Version Rolling Mask value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::VersionRolling;
    ///
    /// let vers_roll = VersionRolling(0x0000_FFFF);
    /// assert_eq!(vers_roll.mask(), 0x1fff_e000);
    /// ```
    pub const fn mask(&self) -> u32 {
        ((self.0 >> Self::VERS_MASK_OFFSET) & Self::VERS_MASK_MASK) << 13
    }
}

impl core::fmt::Display for VersionRolling {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VersionRolling").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for VersionRolling {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "VersionRolling {{  }}",);
    }
}
