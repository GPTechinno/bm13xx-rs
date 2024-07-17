use crate::register::Register;

/// # Version Rolling register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct VersionRolling(pub u32);
impl_boilerplate_for!(VersionRolling);

impl VersionRolling {
    pub const ADDR: u8 = 0xA4;

    const EN_OFFSET: u8 = 31;
    const MASK_OFFSET: u8 = 0;

    const EN_MASK: u32 = 0x1;
    const MASK_MASK: u32 = 0xffff;

    /// ## Handle the enable field.
    ///
    /// Get and set the enabled state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::VersionRolling;
    ///
    /// let mut vers_roll = VersionRolling(0x0000_FFFF);
    /// assert!(!vers_roll.enabled());
    /// assert!(vers_roll.enable().enabled());
    /// assert!(!vers_roll.disable().enabled());
    /// ```
    pub const fn enabled(&self) -> bool {
        (self.0 >> Self::EN_OFFSET) & Self::EN_MASK == Self::EN_MASK
    }
    pub fn enable(&mut self) -> &mut Self {
        self.0 |= Self::EN_MASK << Self::EN_OFFSET;
        self
    }
    pub fn disable(&mut self) -> &mut Self {
        self.0 &= !(Self::EN_MASK << Self::EN_OFFSET);
        self
    }

    /// ## Handle the mask field.
    ///
    /// Get and set the mask value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::VersionRolling;
    ///
    /// let mut vers_roll = VersionRolling(0x0000_FFFF);
    /// assert_eq!(vers_roll.mask(), 0x1fff_e000);
    /// assert_eq!(vers_roll.set_mask(0x1fff_e000).mask(), 0x1fff_e000);
    /// ```
    pub const fn mask(&self) -> u32 {
        ((self.0 >> Self::MASK_OFFSET) & Self::MASK_MASK) << 13
    }
    pub fn set_mask(&mut self, mask: u32) -> &mut Self {
        self.0 &= !(Self::MASK_MASK << Self::MASK_OFFSET);
        self.0 |= ((mask >> 13) & Self::MASK_MASK) << Self::MASK_OFFSET;
        self
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
