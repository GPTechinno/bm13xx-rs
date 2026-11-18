use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
///
/// Core register 5 of the older numbering (BM1397, BM1366). On the BM1368 and later the hash clock control is [`HashClockCtrlV2`](crate::core_register::HashClockCtrlV2), core register 3.
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
///
/// Core register 6 of the older numbering (BM1397). On the BM1368 and later the counter is [`HashClockCounterV2`](crate::core_register::HashClockCounterV2), core register 4, and core register 6 is [`PgMont`](crate::core_register::PgMont).
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

/// # Hash Clock Ctrl core register (`HASH_CLOCK_CTRL`)
///
/// Core register numbering of the BM1368 and later.
///
/// Core register 3 on the BM1368 and later; `HashClockCtrl` is core register 5 of the older chips.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCtrlV2(pub u8);
impl_boilerplate_for_core_reg!(HashClockCtrlV2);

impl HashClockCtrlV2 {
    pub const ID: u8 = 3;

    const CLOCK_CTRL_OFFSET: u8 = 0;

    const CLOCK_CTRL_MASK: u8 = 0b11111;

    /// ## Handle the `CLOCK_CTRL` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::HashClockCtrlV2;
    ///
    /// let mut reg = HashClockCtrlV2(0x00); // default value
    /// assert_eq!(reg.clock_ctrl(), 0x00);
    /// assert_eq!(reg.set_clock_ctrl(0x1f).clock_ctrl(), 0x1f);
    /// assert_eq!(reg.set_clock_ctrl(0x20).clock_ctrl(), 0x00); // out of bound value
    /// ```
    pub const fn clock_ctrl(&self) -> u8 {
        (self.0 >> Self::CLOCK_CTRL_OFFSET) & Self::CLOCK_CTRL_MASK
    }
    pub fn set_clock_ctrl(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CLOCK_CTRL_MASK << Self::CLOCK_CTRL_OFFSET);
        self.0 |= (value & Self::CLOCK_CTRL_MASK) << Self::CLOCK_CTRL_OFFSET;
        self
    }
}

impl ::core::fmt::Display for HashClockCtrlV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashClockCtrlV2")
            .field("clock_ctrl", &self.clock_ctrl())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashClockCtrlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "HashClockCtrlV2 {{ clock_ctrl: {} }}",
            self.clock_ctrl(),
        );
    }
}

/// # Hash Clock Counter core register (`HASH_CLOCK_COUNTER`)
///
/// Core register numbering of the BM1368 and later.
///
/// Core register 4 on the BM1368 and later; `HashClockCounter` is core register 6 of the older chips.
///
/// Bits 15:8 (`PIPE63_CLOCK_CNT`) lie beyond the 8 bits a core register write carries.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashClockCounterV2(pub u8);
impl_boilerplate_for_core_reg!(HashClockCounterV2);

impl HashClockCounterV2 {
    pub const ID: u8 = 4;

    const HASH_CLOCK_CNT_OFFSET: u8 = 0;

    const HASH_CLOCK_CNT_MASK: u8 = 0b11111111;

    /// ## Handle the `HASH_CLOCK_CNT` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::HashClockCounterV2;
    ///
    /// let mut reg = HashClockCounterV2(0x00); // default value
    /// assert_eq!(reg.hash_clock_cnt(), 0x00);
    /// assert_eq!(reg.set_hash_clock_cnt(0x5a).hash_clock_cnt(), 0x5a);
    /// ```
    pub const fn hash_clock_cnt(&self) -> u8 {
        (self.0 >> Self::HASH_CLOCK_CNT_OFFSET) & Self::HASH_CLOCK_CNT_MASK
    }
    pub fn set_hash_clock_cnt(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::HASH_CLOCK_CNT_MASK << Self::HASH_CLOCK_CNT_OFFSET);
        self.0 |= (value & Self::HASH_CLOCK_CNT_MASK) << Self::HASH_CLOCK_CNT_OFFSET;
        self
    }
}

impl ::core::fmt::Display for HashClockCounterV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashClockCounterV2")
            .field("hash_clock_cnt", &self.hash_clock_cnt())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashClockCounterV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "HashClockCounterV2 {{ hash_clock_cnt: {} }}",
            self.hash_clock_cnt(),
        );
    }
}
