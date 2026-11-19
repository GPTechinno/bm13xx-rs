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

/// # Misc Control register, BM1372 layout
///
/// Same register as [`MiscControlV2`](crate::register::MiscControlV2), with `RNO_EN` on 8 bits and `RTST_F_EN`/`RTST_M_EN` moved to [`MiscControlAdd`](crate::register::MiscControlAdd).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MiscControlV3(pub u32);
impl_boilerplate_for!(MiscControlV3);

impl MiscControlV3 {
    pub const ADDR: u8 = 0x18;

    const RNO_EN_OFFSET: u8 = 24;
    const CORE_SWPS_OFFSET: u8 = 23;
    const VERIFY_FIFO_FULL_OFFSET: u8 = 22;
    const SPAT_NOD_OFFSET: u8 = 21;
    const RVS_K0_OFFSET: u8 = 20;
    const TVER_EN_OFFSET: u8 = 15;
    const RNTMO_EN_OFFSET: u8 = 14;
    const SOFT_INV_CLKO_OFFSET: u8 = 13;
    const FORCE_CORE_EN_OFFSET: u8 = 9;
    const PCE_EN_OFFSET: u8 = 8;
    const RET_WORK_ERR_FLAG_OFFSET: u8 = 7;
    const CHIPID_MV_OFFSET: u8 = 6;
    const RI_DISA_OFFSET: u8 = 3;
    const FSW_CKEN_OFFSET: u8 = 2;
    const TOP_CTRL_SEL_OFFSET: u8 = 1;
    const DOMAIN_VDD_SEL_OFFSET: u8 = 0;

    const RNO_EN_MASK: u32 = 0b11111111;
    const CORE_SWPS_MASK: u32 = 0b1;
    const VERIFY_FIFO_FULL_MASK: u32 = 0b1;
    const SPAT_NOD_MASK: u32 = 0b1;
    const RVS_K0_MASK: u32 = 0b1;
    const TVER_EN_MASK: u32 = 0b1;
    const RNTMO_EN_MASK: u32 = 0b1;
    const SOFT_INV_CLKO_MASK: u32 = 0b1;
    const FORCE_CORE_EN_MASK: u32 = 0b1;
    const PCE_EN_MASK: u32 = 0b1;
    const RET_WORK_ERR_FLAG_MASK: u32 = 0b1;
    const CHIPID_MV_MASK: u32 = 0b1;
    const RI_DISA_MASK: u32 = 0b1;
    const FSW_CKEN_MASK: u32 = 0b1;
    const TOP_CTRL_SEL_MASK: u32 = 0b1;
    const DOMAIN_VDD_SEL_MASK: u32 = 0b1;

    /// ## Handle the `RNO_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.rno_en(), 0x0);
    /// assert_eq!(reg.set_rno_en(0xff).rno_en(), 0xff); // max value
    /// ```
    pub const fn rno_en(&self) -> u8 {
        ((self.0 >> Self::RNO_EN_OFFSET) & Self::RNO_EN_MASK) as u8
    }
    pub fn set_rno_en(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::RNO_EN_MASK << Self::RNO_EN_OFFSET);
        self.0 |= ((value as u32) & Self::RNO_EN_MASK) << Self::RNO_EN_OFFSET;
        self
    }

    /// ## Handle the `CORE_SWPS` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.core_swps(), false);
    /// assert!(reg.set_core_swps(true).core_swps());
    /// assert!(!reg.set_core_swps(false).core_swps());
    /// ```
    pub const fn core_swps(&self) -> bool {
        (self.0 >> Self::CORE_SWPS_OFFSET) & Self::CORE_SWPS_MASK != 0
    }
    pub fn set_core_swps(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::CORE_SWPS_MASK << Self::CORE_SWPS_OFFSET);
        self.0 |= (value as u32) << Self::CORE_SWPS_OFFSET;
        self
    }

    /// ## Handle the `VERIFY_FIFO_FULL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.verify_fifo_full(), false);
    /// assert!(reg.set_verify_fifo_full(true).verify_fifo_full());
    /// assert!(!reg.set_verify_fifo_full(false).verify_fifo_full());
    /// ```
    pub const fn verify_fifo_full(&self) -> bool {
        (self.0 >> Self::VERIFY_FIFO_FULL_OFFSET) & Self::VERIFY_FIFO_FULL_MASK != 0
    }
    pub fn set_verify_fifo_full(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::VERIFY_FIFO_FULL_MASK << Self::VERIFY_FIFO_FULL_OFFSET);
        self.0 |= (value as u32) << Self::VERIFY_FIFO_FULL_OFFSET;
        self
    }

    /// ## Handle the `SPAT_NOD` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.spat_nod(), false);
    /// assert!(reg.set_spat_nod(true).spat_nod());
    /// assert!(!reg.set_spat_nod(false).spat_nod());
    /// ```
    pub const fn spat_nod(&self) -> bool {
        (self.0 >> Self::SPAT_NOD_OFFSET) & Self::SPAT_NOD_MASK != 0
    }
    pub fn set_spat_nod(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::SPAT_NOD_MASK << Self::SPAT_NOD_OFFSET);
        self.0 |= (value as u32) << Self::SPAT_NOD_OFFSET;
        self
    }

    /// ## Handle the `RVS_K0` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.rvs_k0(), false);
    /// assert!(reg.set_rvs_k0(true).rvs_k0());
    /// assert!(!reg.set_rvs_k0(false).rvs_k0());
    /// ```
    pub const fn rvs_k0(&self) -> bool {
        (self.0 >> Self::RVS_K0_OFFSET) & Self::RVS_K0_MASK != 0
    }
    pub fn set_rvs_k0(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RVS_K0_MASK << Self::RVS_K0_OFFSET);
        self.0 |= (value as u32) << Self::RVS_K0_OFFSET;
        self
    }

    /// ## Handle the `TVER_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.tver_en(), true);
    /// assert!(reg.set_tver_en(true).tver_en());
    /// assert!(!reg.set_tver_en(false).tver_en());
    /// ```
    pub const fn tver_en(&self) -> bool {
        (self.0 >> Self::TVER_EN_OFFSET) & Self::TVER_EN_MASK != 0
    }
    pub fn set_tver_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TVER_EN_MASK << Self::TVER_EN_OFFSET);
        self.0 |= (value as u32) << Self::TVER_EN_OFFSET;
        self
    }

    /// ## Handle the `RNTMO_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.rntmo_en(), true);
    /// assert!(reg.set_rntmo_en(true).rntmo_en());
    /// assert!(!reg.set_rntmo_en(false).rntmo_en());
    /// ```
    pub const fn rntmo_en(&self) -> bool {
        (self.0 >> Self::RNTMO_EN_OFFSET) & Self::RNTMO_EN_MASK != 0
    }
    pub fn set_rntmo_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RNTMO_EN_MASK << Self::RNTMO_EN_OFFSET);
        self.0 |= (value as u32) << Self::RNTMO_EN_OFFSET;
        self
    }

    /// ## Handle the `SOFT_INV_CLKO` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.soft_inv_clko(), false);
    /// assert!(reg.set_soft_inv_clko(true).soft_inv_clko());
    /// assert!(!reg.set_soft_inv_clko(false).soft_inv_clko());
    /// ```
    pub const fn soft_inv_clko(&self) -> bool {
        (self.0 >> Self::SOFT_INV_CLKO_OFFSET) & Self::SOFT_INV_CLKO_MASK != 0
    }
    pub fn set_soft_inv_clko(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::SOFT_INV_CLKO_MASK << Self::SOFT_INV_CLKO_OFFSET);
        self.0 |= (value as u32) << Self::SOFT_INV_CLKO_OFFSET;
        self
    }

    /// ## Handle the `FORCE_CORE_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.force_core_en(), false);
    /// assert!(reg.set_force_core_en(true).force_core_en());
    /// assert!(!reg.set_force_core_en(false).force_core_en());
    /// ```
    pub const fn force_core_en(&self) -> bool {
        (self.0 >> Self::FORCE_CORE_EN_OFFSET) & Self::FORCE_CORE_EN_MASK != 0
    }
    pub fn set_force_core_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::FORCE_CORE_EN_MASK << Self::FORCE_CORE_EN_OFFSET);
        self.0 |= (value as u32) << Self::FORCE_CORE_EN_OFFSET;
        self
    }

    /// ## Handle the `PCE_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.pce_en(), true);
    /// assert!(reg.set_pce_en(true).pce_en());
    /// assert!(!reg.set_pce_en(false).pce_en());
    /// ```
    pub const fn pce_en(&self) -> bool {
        (self.0 >> Self::PCE_EN_OFFSET) & Self::PCE_EN_MASK != 0
    }
    pub fn set_pce_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::PCE_EN_MASK << Self::PCE_EN_OFFSET);
        self.0 |= (value as u32) << Self::PCE_EN_OFFSET;
        self
    }

    /// ## Handle the `RET_WORK_ERR_FLAG` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.ret_work_err_flag(), false);
    /// assert!(reg.set_ret_work_err_flag(true).ret_work_err_flag());
    /// assert!(!reg.set_ret_work_err_flag(false).ret_work_err_flag());
    /// ```
    pub const fn ret_work_err_flag(&self) -> bool {
        (self.0 >> Self::RET_WORK_ERR_FLAG_OFFSET) & Self::RET_WORK_ERR_FLAG_MASK != 0
    }
    pub fn set_ret_work_err_flag(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RET_WORK_ERR_FLAG_MASK << Self::RET_WORK_ERR_FLAG_OFFSET);
        self.0 |= (value as u32) << Self::RET_WORK_ERR_FLAG_OFFSET;
        self
    }

    /// ## Handle the `CHIPID_MV` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.chipid_mv(), false);
    /// assert!(reg.set_chipid_mv(true).chipid_mv());
    /// assert!(!reg.set_chipid_mv(false).chipid_mv());
    /// ```
    pub const fn chipid_mv(&self) -> bool {
        (self.0 >> Self::CHIPID_MV_OFFSET) & Self::CHIPID_MV_MASK != 0
    }
    pub fn set_chipid_mv(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::CHIPID_MV_MASK << Self::CHIPID_MV_OFFSET);
        self.0 |= (value as u32) << Self::CHIPID_MV_OFFSET;
        self
    }

    /// ## Handle the `RI_DISA` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.ri_disa(), false);
    /// assert!(reg.set_ri_disa(true).ri_disa());
    /// assert!(!reg.set_ri_disa(false).ri_disa());
    /// ```
    pub const fn ri_disa(&self) -> bool {
        (self.0 >> Self::RI_DISA_OFFSET) & Self::RI_DISA_MASK != 0
    }
    pub fn set_ri_disa(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RI_DISA_MASK << Self::RI_DISA_OFFSET);
        self.0 |= (value as u32) << Self::RI_DISA_OFFSET;
        self
    }

    /// ## Handle the `FSW_CKEN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.fsw_cken(), false);
    /// assert!(reg.set_fsw_cken(true).fsw_cken());
    /// assert!(!reg.set_fsw_cken(false).fsw_cken());
    /// ```
    pub const fn fsw_cken(&self) -> bool {
        (self.0 >> Self::FSW_CKEN_OFFSET) & Self::FSW_CKEN_MASK != 0
    }
    pub fn set_fsw_cken(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::FSW_CKEN_MASK << Self::FSW_CKEN_OFFSET);
        self.0 |= (value as u32) << Self::FSW_CKEN_OFFSET;
        self
    }

    /// ## Handle the `TOP_CTRL_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.top_ctrl_sel(), false);
    /// assert!(reg.set_top_ctrl_sel(true).top_ctrl_sel());
    /// assert!(!reg.set_top_ctrl_sel(false).top_ctrl_sel());
    /// ```
    pub const fn top_ctrl_sel(&self) -> bool {
        (self.0 >> Self::TOP_CTRL_SEL_OFFSET) & Self::TOP_CTRL_SEL_MASK != 0
    }
    pub fn set_top_ctrl_sel(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TOP_CTRL_SEL_MASK << Self::TOP_CTRL_SEL_OFFSET);
        self.0 |= (value as u32) << Self::TOP_CTRL_SEL_OFFSET;
        self
    }

    /// ## Handle the `DOMAIN_VDD_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlV3;
    ///
    /// let mut reg = MiscControlV3(0x0000C100); // reset value
    /// assert_eq!(reg.domain_vdd_sel(), false);
    /// assert!(reg.set_domain_vdd_sel(true).domain_vdd_sel());
    /// assert!(!reg.set_domain_vdd_sel(false).domain_vdd_sel());
    /// ```
    pub const fn domain_vdd_sel(&self) -> bool {
        (self.0 >> Self::DOMAIN_VDD_SEL_OFFSET) & Self::DOMAIN_VDD_SEL_MASK != 0
    }
    pub fn set_domain_vdd_sel(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::DOMAIN_VDD_SEL_MASK << Self::DOMAIN_VDD_SEL_OFFSET);
        self.0 |= (value as u32) << Self::DOMAIN_VDD_SEL_OFFSET;
        self
    }
}

impl core::fmt::Display for MiscControlV3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MiscControlV3")
            .field("rno_en", &self.rno_en())
            .field("core_swps", &self.core_swps())
            .field("verify_fifo_full", &self.verify_fifo_full())
            .field("spat_nod", &self.spat_nod())
            .field("rvs_k0", &self.rvs_k0())
            .field("tver_en", &self.tver_en())
            .field("rntmo_en", &self.rntmo_en())
            .field("soft_inv_clko", &self.soft_inv_clko())
            .field("force_core_en", &self.force_core_en())
            .field("pce_en", &self.pce_en())
            .field("ret_work_err_flag", &self.ret_work_err_flag())
            .field("chipid_mv", &self.chipid_mv())
            .field("ri_disa", &self.ri_disa())
            .field("fsw_cken", &self.fsw_cken())
            .field("top_ctrl_sel", &self.top_ctrl_sel())
            .field("domain_vdd_sel", &self.domain_vdd_sel())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MiscControlV3 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "MiscControlV3 {{ rno_en: {}, core_swps: {}, verify_fifo_full: {}, spat_nod: {}, rvs_k0: {}, tver_en: {}, rntmo_en: {}, soft_inv_clko: {}, force_core_en: {}, pce_en: {}, ret_work_err_flag: {}, chipid_mv: {}, ri_disa: {}, fsw_cken: {}, top_ctrl_sel: {}, domain_vdd_sel: {} }}",
            self.rno_en(),
            self.core_swps(),
            self.verify_fifo_full(),
            self.spat_nod(),
            self.rvs_k0(),
            self.tver_en(),
            self.rntmo_en(),
            self.soft_inv_clko(),
            self.force_core_en(),
            self.pce_en(),
            self.ret_work_err_flag(),
            self.chipid_mv(),
            self.ri_disa(),
            self.fsw_cken(),
            self.top_ctrl_sel(),
            self.domain_vdd_sel(),
        );
    }
}
