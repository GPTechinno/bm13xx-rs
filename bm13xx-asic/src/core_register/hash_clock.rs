use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCtrl(pub u8);
impl_boilerplate_for_core_reg!(HashClockCtrl);

impl HashClockCtrl {
    pub const ID: u8 = 5;

    const EN_OFFSET: u8 = 6;
    const PLL_SRC_OFFSET: u8 = 0;

    const EN_MASK: u8 = 0b1;
    const PLL_SRC_MASK: u8 = 0b1;

    /// ## Handle the enable field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::HashClockCtrl;
    ///
    /// let mut hash_clock_ctrl = HashClockCtrl(0x40); // BM1366 default value
    /// assert!(hash_clock_ctrl.enabled());
    /// assert!(!hash_clock_ctrl.disable().enabled());
    /// assert!(hash_clock_ctrl.enable().enabled());
    /// ```
    pub const fn enabled(&self) -> bool {
        (self.0 >> Self::EN_OFFSET) & Self::EN_MASK != 0
    }
    pub fn enable(&mut self) -> &mut Self {
        self.0 |= Self::EN_MASK << Self::EN_OFFSET;
        self
    }
    pub fn disable(&mut self) -> &mut Self {
        self.0 &= !(Self::EN_MASK << Self::EN_OFFSET);
        self
    }

    /// ## Handle the PLL source field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::HashClockCtrl;
    ///
    /// let mut hash_clock_ctrl = HashClockCtrl(0x40); // BM1366 default value
    /// assert_eq!(hash_clock_ctrl.pll_source(), 0);
    /// assert_eq!(hash_clock_ctrl.set_pll_source(1).pll_source(), 1); // max value
    /// assert_eq!(hash_clock_ctrl.set_pll_source(2).pll_source(), 0); // out of bound value
    /// ```
    pub const fn pll_source(&self) -> u8 {
        (self.0 >> Self::PLL_SRC_OFFSET) & Self::PLL_SRC_MASK
    }
    pub fn set_pll_source(&mut self, pll_id: u8) -> &mut Self {
        self.0 &= !(Self::PLL_SRC_MASK << Self::PLL_SRC_OFFSET);
        self.0 |= (pll_id & Self::PLL_SRC_MASK) << Self::PLL_SRC_OFFSET;
        self
    }
}

impl ::core::fmt::Display for HashClockCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashClockCtrl")
            .field("enabled", &self.enabled())
            .field("pll_source", &self.pll_source())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashClockCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "HashClockCtrl {{ enabled: {}, pll_source: {} }}",
            self.enabled(),
            self.pll_source(),
        );
    }
}

/// # Hash Clock Counter core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCounter(pub u8);
impl_boilerplate_for_core_reg!(HashClockCounter);

impl HashClockCounter {
    pub const ID: u8 = 6;

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
