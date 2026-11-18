use crate::core_register::CoreRegister;

/// # Clock Select Ctrl core register (`CLOCK_SELECT_CTRL`)
///
/// Core register numbering of the BM1368 and later.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClockSelectCtrl(pub u8);
impl_boilerplate_for_core_reg!(ClockSelectCtrl);

impl ClockSelectCtrl {
    pub const ID: u8 = 11;

    const SWPF_MODE_OFFSET: u8 = 0;
    const CLK_SEL_OFFSET: u8 = 1;
    const CLK_SEL_MASK_OFFSET: u8 = 3;

    const SWPF_MODE_MASK: u8 = 0b1;
    const CLK_SEL_MASK: u8 = 0b11;
    const CLK_SEL_MASK_MASK: u8 = 0b1;

    /// ## Handle the `SWPF_MODE` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::ClockSelectCtrl;
    ///
    /// let mut reg = ClockSelectCtrl(0x00); // default value
    /// assert_eq!(reg.swpf_mode(), false);
    /// assert!(reg.set_swpf_mode(true).swpf_mode());
    /// assert!(!reg.set_swpf_mode(false).swpf_mode());
    /// ```
    pub const fn swpf_mode(&self) -> bool {
        (self.0 >> Self::SWPF_MODE_OFFSET) & Self::SWPF_MODE_MASK != 0
    }
    pub fn set_swpf_mode(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::SWPF_MODE_MASK << Self::SWPF_MODE_OFFSET);
        self.0 |= (value as u8) << Self::SWPF_MODE_OFFSET;
        self
    }

    /// ## Handle the `CLK_SEL` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::ClockSelectCtrl;
    ///
    /// let mut reg = ClockSelectCtrl(0x00); // default value
    /// assert_eq!(reg.clk_sel(), 0x00);
    /// assert_eq!(reg.set_clk_sel(0x03).clk_sel(), 0x03);
    /// assert_eq!(reg.set_clk_sel(0x04).clk_sel(), 0x00); // out of bound value
    /// ```
    pub const fn clk_sel(&self) -> u8 {
        (self.0 >> Self::CLK_SEL_OFFSET) & Self::CLK_SEL_MASK
    }
    pub fn set_clk_sel(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CLK_SEL_MASK << Self::CLK_SEL_OFFSET);
        self.0 |= (value & Self::CLK_SEL_MASK) << Self::CLK_SEL_OFFSET;
        self
    }

    /// ## Handle the `CLK_SEL_MASK` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::ClockSelectCtrl;
    ///
    /// let mut reg = ClockSelectCtrl(0x00); // default value
    /// assert_eq!(reg.clk_sel_mask(), false);
    /// assert!(reg.set_clk_sel_mask(true).clk_sel_mask());
    /// assert!(!reg.set_clk_sel_mask(false).clk_sel_mask());
    /// ```
    pub const fn clk_sel_mask(&self) -> bool {
        (self.0 >> Self::CLK_SEL_MASK_OFFSET) & Self::CLK_SEL_MASK_MASK != 0
    }
    pub fn set_clk_sel_mask(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::CLK_SEL_MASK_MASK << Self::CLK_SEL_MASK_OFFSET);
        self.0 |= (value as u8) << Self::CLK_SEL_MASK_OFFSET;
        self
    }
}

impl ::core::fmt::Display for ClockSelectCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockSelectCtrl")
            .field("swpf_mode", &self.swpf_mode())
            .field("clk_sel", &self.clk_sel())
            .field("clk_sel_mask", &self.clk_sel_mask())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockSelectCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockSelectCtrl {{ swpf_mode: {}, clk_sel: {}, clk_sel_mask: {} }}",
            self.swpf_mode(),
            self.clk_sel(),
            self.clk_sel_mask(),
        );
    }
}
