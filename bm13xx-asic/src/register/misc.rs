use crate::register::Register;

/// Baudrate CLocK SELect.
///
/// This is used by [`MiscControl::bclk_sel`] method.
///
/// [`MiscControl::bclk_sel`]: crate::register::MiscControl::bclk_sel
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BaudrateClockSelect {
    /// Baudrate base clock is CLKI (external clock).
    #[default]
    Clki = 0,
    /// Baudrate base clock is PLL3.
    Pll3 = 1,
}

impl From<bool> for BaudrateClockSelect {
    fn from(val: bool) -> BaudrateClockSelect {
        if val {
            BaudrateClockSelect::Clki
        } else {
            BaudrateClockSelect::Pll3
        }
    }
}

impl From<BaudrateClockSelect> for u8 {
    fn from(val: BaudrateClockSelect) -> u8 {
        val as u8
    }
}

/// # Misc Control register
///
/// Used to control various settings.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MiscControl(pub u32);
impl_boilerplate_for!(MiscControl);

impl MiscControl {
    pub const ADDR: u8 = 0x18;

    const BT8D_8_5_OFFSET: u8 = 24;
    const CORE_SRST_OFFSET: u8 = 22;
    // const SPAT_NOD_OFFSET: u8 = 21;
    // const RVS_K0_OFFSET: u8 = 20;
    // const DSCLK_SEL_OFFSET: u8 = 18;
    // const TOP_CLK_SEL_OFFSET: u8 = 17;
    const BCK_SEL_OFFSET: u8 = 16;
    // const RET_ERR_NONCE_OFFSET: u8 = 15;
    // const RFS_OFFSET: u8 = 14;
    // const INV_CLKO_OFFSET: u8 = 13;
    const BT8D_4_0_OFFSET: u8 = 8;
    // const RET_WORK_ERR_FLAG_OFFSET: u8 = 7;
    // const TFS_OFFSET: u8 = 4;
    // const HASHRATE_TWS_OFFSET: u8 = 0;

    const BT8D_8_5_MASK: u32 = 0b1111;
    const CORE_SRST_MASK: u32 = 0b1;
    // const SPAT_NOD_MASK: u32 = 0b1;
    // const RVS_K0_MASK: u32 = 0b1;
    // const DSCLK_SEL_MASK: u32 = 0b11;
    // const TOP_CLK_SEL_MASK: u32 = 0b1;
    const BCK_SEL_MASK: u32 = 0b1;
    // const RET_ERR_NONCE_MASK: u32 = 0b1;
    // const RFS_MASK: u32 = 0b1;
    // const INV_CLKO_MASK: u32 = 0b1;
    const BT8D_4_0_MASK: u32 = 0b11111;
    // const RET_WORK_ERR_FLAG_MASK: u32 = 0b1;
    // const TFS_MASK: u32 = 0xb111;
    // const HASHRATE_TWS_MASK: u32 = 0xb11;

    /// ## Handle the BT8D field.
    ///
    /// This returns an `u16` with the 9-bits BT8D value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControl;
    ///
    /// let mut misc = MiscControl(0x0000_3A01);
    /// assert_eq!(misc.bt8d(), 26);
    /// assert_eq!(misc.set_bt8d(0).bt8d(), 0); // min value
    /// assert_eq!(misc.set_bt8d(0x1ff).bt8d(), 511); // max value
    /// assert_eq!(misc.set_bt8d(0x200).bt8d(), 0); // out of bound value
    /// ```
    pub const fn bt8d(&self) -> u16 {
        ((((self.0 >> Self::BT8D_8_5_OFFSET) & Self::BT8D_8_5_MASK) as u16) << 5)
            | (((self.0 >> Self::BT8D_4_0_OFFSET) & Self::BT8D_4_0_MASK) as u16)
    }
    pub fn set_bt8d(&mut self, bt8d: u16) -> &mut Self {
        self.0 &= !(Self::BT8D_8_5_MASK << Self::BT8D_8_5_OFFSET);
        self.0 |= ((bt8d >> 5) as u32 & Self::BT8D_8_5_MASK) << Self::BT8D_8_5_OFFSET;
        self.0 &= !(Self::BT8D_4_0_MASK << Self::BT8D_4_0_OFFSET);
        self.0 |= (bt8d as u32 & Self::BT8D_4_0_MASK) << Self::BT8D_4_0_OFFSET;
        self
    }

    /// ## Reset the Core.
    ///
    /// This returns an `bool` with the Core Reset state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControl;
    ///
    /// let misc = MiscControl(0x0000_3A01);
    /// assert!(!misc.core_srst());
    /// ```
    pub const fn core_srst(&self) -> bool {
        (self.0 >> Self::CORE_SRST_OFFSET) & Self::CORE_SRST_MASK == Self::CORE_SRST_MASK
    }

    /// ## Handle the Baudrate Clock Select field.
    ///
    /// This returns an `BaudrateClockSelect` with the current Baudrate Clock Select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{BaudrateClockSelect, MiscControl};
    ///
    /// let mut misc = MiscControl(0x0000_3A01);
    /// assert_eq!(misc.bclk_sel(), BaudrateClockSelect::Clki);
    /// assert_eq!(misc.set_bclk_sel(BaudrateClockSelect::Pll3).bclk_sel(), BaudrateClockSelect::Pll3);
    /// ```
    pub const fn bclk_sel(&self) -> BaudrateClockSelect {
        match (self.0 >> Self::BCK_SEL_OFFSET) & Self::BCK_SEL_MASK == Self::BCK_SEL_MASK {
            true => BaudrateClockSelect::Pll3,
            false => BaudrateClockSelect::Clki,
        }
    }
    pub fn set_bclk_sel(&mut self, bclk_sel: BaudrateClockSelect) -> &mut Self {
        self.0 &= !(Self::BCK_SEL_MASK << Self::BCK_SEL_OFFSET);
        self.0 |= (bclk_sel as u32) << Self::BCK_SEL_OFFSET;
        self
    }
}

impl core::fmt::Display for MiscControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MiscControl")
            .field("bt8d", &self.bt8d())
            .field("core_srst", &self.core_srst())
            .field("bclk_sel", &self.bclk_sel())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MiscControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "MiscControl {{ bt8d: {}, core_srst: {}, bclk_sel: {} }}",
            self.bt8d(),
            self.core_srst(),
            self.bclk_sel(),
        );
    }
}

/// # Misc Control register V2
///
/// Used to control various settings.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MiscControlV2(pub u32);
impl_boilerplate_for!(MiscControlV2);

impl MiscControlV2 {
    pub const ADDR: u8 = 0x18;

    const CORE_RETURN_NONCE_OFFSET: u8 = 28;
    const B27_26_OFFSET: u8 = 26;
    const B25_24_OFFSET: u8 = 24;
    const B19_16_OFFSET: u8 = 16;

    const CORE_RETURN_NONCE_MASK: u32 = 0b1111;
    const B27_26_MASK: u32 = 0b11;
    const B25_24_MASK: u32 = 0b11;
    const B19_16_MASK: u32 = 0b1111;

    /// ## Handle the Core Return Nonce field.
    ///
    /// Get and set the Core Return Nonce value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV2;
    ///
    /// let mut misc = MiscControlV2(0x0000_C100); // BM1366 default value
    /// assert_eq!(misc.core_return_nonce(), 0);
    /// assert_eq!(misc.set_core_return_nonce(0xf).core_return_nonce(), 0xf); // max value
    /// assert_eq!(misc.set_core_return_nonce(0x10).core_return_nonce(), 0); // out of bound value
    /// ```
    pub const fn core_return_nonce(&self) -> u8 {
        ((self.0 >> Self::CORE_RETURN_NONCE_OFFSET) & Self::CORE_RETURN_NONCE_MASK) as u8
    }
    pub fn set_core_return_nonce(&mut self, core_return_nonce: u8) -> &mut Self {
        self.0 &= !(Self::CORE_RETURN_NONCE_MASK << Self::CORE_RETURN_NONCE_OFFSET);
        self.0 |= ((core_return_nonce as u32) & Self::CORE_RETURN_NONCE_MASK)
            << Self::CORE_RETURN_NONCE_OFFSET;
        self
    }

    /// ## Handle the B\[27:26\] field.
    ///
    /// Get and set the B\[27:26\] value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV2;
    ///
    /// let mut misc = MiscControlV2(0x0000_C100); // BM1366 default value
    /// assert_eq!(misc.b27_26(), 0);
    /// assert_eq!(misc.set_b27_26(0b11).b27_26(), 0b11); // max value
    /// assert_eq!(misc.set_b27_26(0b100).b27_26(), 0b00); // out of bound value
    /// ```
    pub const fn b27_26(&self) -> u8 {
        ((self.0 >> Self::B27_26_OFFSET) & Self::B27_26_MASK) as u8
    }
    pub fn set_b27_26(&mut self, b27_26: u8) -> &mut Self {
        self.0 &= !(Self::B27_26_MASK << Self::B27_26_OFFSET);
        self.0 |= ((b27_26 as u32) & Self::B27_26_MASK) << Self::B27_26_OFFSET;
        self
    }

    /// ## Handle the B\[25:24\] field.
    ///
    /// Get and set the B\[25:24\] value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV2;
    ///
    /// let mut misc = MiscControlV2(0x0000_C100); // BM1366 default value
    /// assert_eq!(misc.b25_24(), 0);
    /// assert_eq!(misc.set_b25_24(0b11).b25_24(), 0b11); // max value
    /// assert_eq!(misc.set_b25_24(0b100).b25_24(), 0b0); // out of bound value
    /// ```
    pub const fn b25_24(&self) -> u8 {
        ((self.0 >> Self::B25_24_OFFSET) & Self::B25_24_MASK) as u8
    }
    pub fn set_b25_24(&mut self, b25_24: u8) -> &mut Self {
        self.0 &= !(Self::B25_24_MASK << Self::B25_24_OFFSET);
        self.0 |= ((b25_24 as u32) & Self::B25_24_MASK) << Self::B25_24_OFFSET;
        self
    }

    /// ## Handle the B\[19:16\] field.
    ///
    /// Get and set the B\[19:16\] value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV2;
    ///
    /// let mut misc = MiscControlV2(0x0000_C100); // BM1366 default value
    /// assert_eq!(misc.b19_16(), 0);
    /// assert_eq!(misc.set_b19_16(0b1111).b19_16(), 0b1111); // max value
    /// assert_eq!(misc.set_b19_16(0b10000).b19_16(), 0b0000); // out of bound value
    /// ```
    pub const fn b19_16(&self) -> u8 {
        ((self.0 >> Self::B19_16_OFFSET) & Self::B19_16_MASK) as u8
    }
    pub fn set_b19_16(&mut self, b19_16: u8) -> &mut Self {
        self.0 &= !(Self::B19_16_MASK << Self::B19_16_OFFSET);
        self.0 |= ((b19_16 as u32) & Self::B19_16_MASK) << Self::B19_16_OFFSET;
        self
    }
}

impl core::fmt::Display for MiscControlV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MiscControlV2").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MiscControlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "MiscControlV2 {{ }}",);
    }
}
