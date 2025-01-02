use crate::register::Register;

/// # Fast UART Configuration register
///
/// Used to configure UART settings.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FastUARTConfiguration(pub u32);
impl_boilerplate_for!(FastUARTConfiguration);

impl FastUARTConfiguration {
    pub const ADDR: u8 = 0x28;

    // const DIV4_ODDSET_OFFSET: u8 = 30;
    const PLL3_DIV4_OFFSET: u8 = 24;
    // const USRC_ODDSET_OFFSET: u8 = 22;
    // const USRC_DIV_OFFSET: u8 = 16;
    // const FORCE_CORE_EN_OFFSET: u8 = 15;
    // const CLKO_SEL_OFFSET: u8 = 14;
    // const CLKO_ODDSET_OFFSET: u8 = 12;
    // const CLKO_DIV_OFFSET: u8 = 0;

    // const DIV4_ODDSET_MASK: u32 = 0b11;
    const PLL3_DIV4_MASK: u32 = 0b1111;
    // const USRC_ODDSET_MASK: u32 = 0b11;
    // const USRC_DIV_MASK: u32 = 0x3f;
    // const FORCE_CORE_EN_MASK: u32 = 0b1;
    // const CLKO_SEL_MASK: u32 = 0b1;
    // const CLKO_ODDSET_MASK: u32 = 0b11;
    // const CLKO_DIV_MASK: u32 = 0xff;

    /// ## Handle the PLL3_DIV4 field.
    ///
    /// This returns an `u8` with the PLL3_DIV4 value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfiguration;
    ///
    /// let mut fast_uart_cfg = FastUARTConfiguration(0x0600_000F); // deafault BM1397 value
    /// assert_eq!(fast_uart_cfg.pll3_div4(), 6);
    /// assert_eq!(fast_uart_cfg.set_pll3_div4(0).pll3_div4(), 0);
    /// assert_eq!(fast_uart_cfg.set_pll3_div4(15).pll3_div4(), 15);
    /// assert_eq!(fast_uart_cfg.set_pll3_div4(16).pll3_div4(), 0);
    /// ```
    pub const fn pll3_div4(&self) -> u8 {
        ((self.0 >> Self::PLL3_DIV4_OFFSET) & Self::PLL3_DIV4_MASK) as u8
    }
    pub fn set_pll3_div4(&mut self, pll3_div4: u8) -> &mut Self {
        self.0 &= !(Self::PLL3_DIV4_MASK << Self::PLL3_DIV4_OFFSET);
        self.0 |= ((pll3_div4 as u32) & Self::PLL3_DIV4_MASK) << Self::PLL3_DIV4_OFFSET;
        self
    }
}

impl core::fmt::Display for FastUARTConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FastUARTConfiguration")
            .field("pll3_div4", &self.pll3_div4())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for FastUARTConfiguration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "FastUARTConfiguration {{ pll3_div4: {} }}",
            self.pll3_div4(),
        );
    }
}

/// Baudrate CLocK SELect (second version)
///
/// This is used by [`FastUARTConfigurationV2::bclk_sel`] method.
///
/// [`FastUARTConfigurationV2::bclk_sel`]: crate::register::FastUARTConfigurationV2::bclk_sel
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(u8)]
pub enum BaudrateClockSelectV2 {
    /// Baudrate base clock is CLKI (external clock).
    #[default]
    Clki = 0,
    /// Baudrate base clock is PLL1.
    Pll1 = 1,
    // /// Baudrate base clock is PLL2.
    // Pll2 = 2,
    // /// Baudrate base clock is PLL3.
    // Pll3 = 3,
}

impl From<u8> for BaudrateClockSelectV2 {
    fn from(val: u8) -> BaudrateClockSelectV2 {
        match val {
            0 => BaudrateClockSelectV2::Clki,
            1 => BaudrateClockSelectV2::Pll1,
            // 2 => BaudrateClockSelectV2::Pll2,
            // 3 => BaudrateClockSelectV2::Pll3,
            _ => unreachable!(),
        }
    }
}

impl From<BaudrateClockSelectV2> for u8 {
    fn from(val: BaudrateClockSelectV2) -> u8 {
        val as u8
    }
}

/// # Fast UART Configuration register (second version)
///
/// Used to configure UART settings.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FastUARTConfigurationV2(pub u32);
impl_boilerplate_for!(FastUARTConfigurationV2);

impl FastUARTConfigurationV2 {
    pub const ADDR: u8 = 0x28;

    // const B31_OFFSET: u8 = 31;
    // const B29_OFFSET: u8 = 30;
    // const B28_OFFSET: u8 = 29;
    const B28_OFFSET: u8 = 28;
    const BCK_SEL_OFFSET: u8 = 26;
    // const B24_OFFSET: u8 = 24;
    const PLL1_DIV4_OFFSET: u8 = 20;
    // const B16_19_OFFSET: u8 = 16;
    const BT8D_OFFSET: u8 = 8;
    // const CLKO_DIV_OFFSET: u8 = 0;

    // const B31_MASK: u32 = 0b1;
    // const B30_MASK: u32 = 0b1;
    // const B29_MASK: u32 = 0b1;
    const B28_MASK: u32 = 0b1;
    const BCK_SEL_MASK: u32 = 0b1; /* should be 0b11 but only 1 values are known for now */
    // const B24_MASK: u32 = 0b1;
    const PLL1_DIV4_MASK: u32 = 0b1111;
    // const B16_19_MASK: u32 = 0b1111;
    const BT8D_MASK: u32 = 0xff;
    // const CLKO_DIV_MASK: u32 = 0xff;

    /// ## Handle the B28 field.
    ///
    /// Get and set the B28 state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationV2;
    ///
    /// let mut fast_uart_cfg = FastUARTConfigurationV2(0x0007_0000); // BM1366 default value
    /// assert!(!fast_uart_cfg.is_b28());
    /// assert!(fast_uart_cfg.set_b28().is_b28());
    /// assert!(!fast_uart_cfg.clr_b28().is_b28());
    /// ```
    pub const fn is_b28(&self) -> bool {
        (self.0 >> Self::B28_OFFSET) & Self::B28_MASK == Self::B28_MASK
    }
    pub fn set_b28(&mut self) -> &mut Self {
        self.0 |= Self::B28_MASK << Self::B28_OFFSET;
        self
    }
    pub fn clr_b28(&mut self) -> &mut Self {
        self.0 &= !(Self::B28_MASK << Self::B28_OFFSET);
        self
    }

    /// ## Handle the Baudrate Clock Select field.
    ///
    /// This returns an `BaudrateClockSelectV2` with the current Baudrate Clock Select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{BaudrateClockSelectV2, FastUARTConfigurationV2};
    ///
    /// let mut fast_uart_cfg = FastUARTConfigurationV2(0x0130_1A00); // BM1366 default value
    /// assert_eq!(fast_uart_cfg.bclk_sel(), BaudrateClockSelectV2::Clki);
    /// assert_eq!(fast_uart_cfg.set_bclk_sel(BaudrateClockSelectV2::Pll1).bclk_sel(), BaudrateClockSelectV2::Pll1);
    /// ```
    pub fn bclk_sel(&self) -> BaudrateClockSelectV2 {
        (((self.0 >> Self::BCK_SEL_OFFSET) & Self::BCK_SEL_MASK) as u8).into()
    }
    pub fn set_bclk_sel(&mut self, bclk_sel: BaudrateClockSelectV2) -> &mut Self {
        self.0 &= !(Self::BCK_SEL_MASK << Self::BCK_SEL_OFFSET);
        self.0 |= ((bclk_sel as u32) & Self::BCK_SEL_MASK) << Self::BCK_SEL_OFFSET;
        self
    }

    /// ## Handle the PLL1_DIV4 field.
    ///
    /// This returns an `u8` with the PLL1_DIV4 value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationV2;
    ///
    /// let mut fast_uart_cfg = FastUARTConfigurationV2(0x0130_1A00); // BM1366 default value
    /// assert_eq!(fast_uart_cfg.pll1_div4(), 3);
    /// assert_eq!(fast_uart_cfg.set_pll1_div4(0).pll1_div4(), 0);
    /// assert_eq!(fast_uart_cfg.set_pll1_div4(15).pll1_div4(), 15);
    /// assert_eq!(fast_uart_cfg.set_pll1_div4(16).pll1_div4(), 0);
    /// ```
    pub const fn pll1_div4(&self) -> u8 {
        ((self.0 >> Self::PLL1_DIV4_OFFSET) & Self::PLL1_DIV4_MASK) as u8
    }
    pub fn set_pll1_div4(&mut self, pll3_div4: u8) -> &mut Self {
        self.0 &= !(Self::PLL1_DIV4_MASK << Self::PLL1_DIV4_OFFSET);
        self.0 |= ((pll3_div4 as u32) & Self::PLL1_DIV4_MASK) << Self::PLL1_DIV4_OFFSET;
        self
    }
    /// ## Handle the BT8D field.
    ///
    /// This returns an `u8` with the BT8D value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationV2;
    ///
    /// let mut fast_uart_cfg = FastUARTConfigurationV2(0x0130_1A00); // BM1366 default value
    /// assert_eq!(fast_uart_cfg.bt8d(), 26);
    /// assert_eq!(fast_uart_cfg.set_bt8d(0).bt8d(), 0); // min value
    /// assert_eq!(fast_uart_cfg.set_bt8d(0xff).bt8d(), 255); // max value
    /// ```
    pub const fn bt8d(&self) -> u8 {
        ((self.0 >> Self::BT8D_OFFSET) & Self::BT8D_MASK) as u8
    }
    pub fn set_bt8d(&mut self, bt8d: u8) -> &mut Self {
        self.0 &= !(Self::BT8D_MASK << Self::BT8D_OFFSET);
        self.0 |= (bt8d as u32 & Self::BT8D_MASK) << Self::BT8D_OFFSET;
        self
    }
}

impl core::fmt::Display for FastUARTConfigurationV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FastUARTConfigurationV2")
            .field("pll1_div4", &self.pll1_div4())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for FastUARTConfigurationV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "FastUARTConfigurationV2 {{ pll1_div4: {} }}",
            self.pll1_div4(),
        );
    }
}
