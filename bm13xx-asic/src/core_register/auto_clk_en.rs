use crate::core_register::CoreRegister;

/// # Auto Clock Enable core register (`AUTO_CLK_EN`)
///
/// Core register numbering of the BM1368 and later.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AutoClkEn(pub u8);
impl_boilerplate_for_core_reg!(AutoClkEn);

impl AutoClkEn {
    pub const ID: u8 = 8;

    const AUTO_CLK_EN_OFFSET: u8 = 0;
    const MODE_MASK_OFFSET: u8 = 5;

    const AUTO_CLK_EN_MASK: u8 = 0b1;
    const MODE_MASK_MASK: u8 = 0b11;

    /// ## Handle the `AUTO_CLK_EN` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::AutoClkEn;
    ///
    /// let mut reg = AutoClkEn(0x00); // default value
    /// assert_eq!(reg.auto_clk_en(), false);
    /// assert!(reg.set_auto_clk_en(true).auto_clk_en());
    /// assert!(!reg.set_auto_clk_en(false).auto_clk_en());
    /// ```
    pub const fn auto_clk_en(&self) -> bool {
        (self.0 >> Self::AUTO_CLK_EN_OFFSET) & Self::AUTO_CLK_EN_MASK != 0
    }
    pub fn set_auto_clk_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::AUTO_CLK_EN_MASK << Self::AUTO_CLK_EN_OFFSET);
        self.0 |= (value as u8) << Self::AUTO_CLK_EN_OFFSET;
        self
    }

    /// ## Handle the `MODE_MASK` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::AutoClkEn;
    ///
    /// let mut reg = AutoClkEn(0x00); // default value
    /// assert_eq!(reg.mode_mask(), 0x00);
    /// assert_eq!(reg.set_mode_mask(0x03).mode_mask(), 0x03);
    /// assert_eq!(reg.set_mode_mask(0x04).mode_mask(), 0x00); // out of bound value
    /// ```
    pub const fn mode_mask(&self) -> u8 {
        (self.0 >> Self::MODE_MASK_OFFSET) & Self::MODE_MASK_MASK
    }
    pub fn set_mode_mask(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::MODE_MASK_MASK << Self::MODE_MASK_OFFSET);
        self.0 |= (value & Self::MODE_MASK_MASK) << Self::MODE_MASK_OFFSET;
        self
    }
}

impl ::core::fmt::Display for AutoClkEn {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("AutoClkEn")
            .field("auto_clk_en", &self.auto_clk_en())
            .field("mode_mask", &self.mode_mask())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AutoClkEn {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "AutoClkEn {{ auto_clk_en: {}, mode_mask: {} }}",
            self.auto_clk_en(),
            self.mode_mask(),
        );
    }
}
