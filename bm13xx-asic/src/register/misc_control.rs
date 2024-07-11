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

    /// ## Get the BT8D.
    ///
    /// This returns an `u16` with the 9-bits BT8D value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControl;
    ///
    /// let misc = MiscControl(0x0000_3A01);
    /// assert_eq!(misc.bt8d(), 0x001A);
    /// ```
    pub const fn bt8d(&self) -> u16 {
        ((((self.0 >> Self::BT8D_8_5_OFFSET) & Self::BT8D_8_5_MASK) as u16) << 5)
            | (((self.0 >> Self::BT8D_4_0_OFFSET) & Self::BT8D_4_0_MASK) as u16)
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

    /// ## Get the Baudrate Clock Select.
    ///
    /// This returns an `BaudrateClockSelect` with the current Baudrate Clock Select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{BaudrateClockSelect, MiscControl};
    ///
    /// let misc = MiscControl(0x0000_3A01);
    /// assert_eq!(misc.bclk_sel(), BaudrateClockSelect::Clki);
    /// ```
    pub const fn bclk_sel(&self) -> BaudrateClockSelect {
        match (self.0 >> Self::BCK_SEL_OFFSET) & Self::BCK_SEL_MASK == Self::BCK_SEL_MASK {
            true => BaudrateClockSelect::Pll3,
            false => BaudrateClockSelect::Clki,
        }
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
