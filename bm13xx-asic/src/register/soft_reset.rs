use crate::register::Register;

/// # SoftResetControl register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SoftResetControl(pub u32);
impl_boilerplate_for!(SoftResetControl);

impl SoftResetControl {
    pub const ADDR: u8 = 0xA8;

    const B10_OFFSET: u8 = 10;
    const B8_OFFSET: u8 = 8;
    const B7_4_OFFSET: u8 = 4;
    const B3_0_OFFSET: u8 = 0;

    const B10_MASK: u32 = 0x1;
    const B8_MASK: u32 = 0x1;
    const B7_4_MASK: u32 = 0xf;
    const B3_0_MASK: u32 = 0xf;

    /// ## Handle the B10 field.
    ///
    /// Get and set the B10 state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControl;
    ///
    /// let mut sft_rst_ctrl = SoftResetControl(0x0007_0000); // BM1366 default value
    /// assert!(!sft_rst_ctrl.is_b10());
    /// assert!(sft_rst_ctrl.set_b10().is_b10());
    /// assert!(!sft_rst_ctrl.clr_b10().is_b10());
    /// ```
    pub const fn is_b10(&self) -> bool {
        (self.0 >> Self::B10_OFFSET) & Self::B10_MASK == Self::B10_MASK
    }
    pub fn set_b10(&mut self) -> &mut Self {
        self.0 |= Self::B10_MASK << Self::B10_OFFSET;
        self
    }
    pub fn clr_b10(&mut self) -> &mut Self {
        self.0 &= !(Self::B10_MASK << Self::B10_OFFSET);
        self
    }

    /// ## Handle the B8 field.
    ///
    /// Get and set the B8 state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControl;
    ///
    /// let mut sft_rst_ctrl = SoftResetControl(0x0007_0000); // BM1366 default value
    /// assert!(!sft_rst_ctrl.is_b8());
    /// assert!(sft_rst_ctrl.set_b8().is_b8());
    /// assert!(!sft_rst_ctrl.clr_b8().is_b8());
    /// ```
    pub const fn is_b8(&self) -> bool {
        (self.0 >> Self::B8_OFFSET) & Self::B8_MASK == Self::B8_MASK
    }
    pub fn set_b8(&mut self) -> &mut Self {
        self.0 |= Self::B8_MASK << Self::B8_OFFSET;
        self
    }
    pub fn clr_b8(&mut self) -> &mut Self {
        self.0 &= !(Self::B8_MASK << Self::B8_OFFSET);
        self
    }

    /// ## Handle the B\[7:4\] field.
    ///
    /// Get and set the B\[7:4\] value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControl;
    ///
    /// let mut sft_rst_ctrl = SoftResetControl(0x0007_0000); // BM1366 default value
    /// assert_eq!(sft_rst_ctrl.b7_4(), 0);
    /// assert_eq!(sft_rst_ctrl.set_b7_4(0xf).b7_4(), 0xf); // max value
    /// assert_eq!(sft_rst_ctrl.set_b7_4(0x10).b7_4(), 0); // out of bound value
    /// ```
    pub const fn b7_4(&self) -> u8 {
        ((self.0 >> Self::B7_4_OFFSET) & Self::B7_4_MASK) as u8
    }
    pub fn set_b7_4(&mut self, b7_4: u8) -> &mut Self {
        self.0 &= !(Self::B7_4_MASK << Self::B7_4_OFFSET);
        self.0 |= ((b7_4 as u32) & Self::B7_4_MASK) << Self::B7_4_OFFSET;
        self
    }

    /// ## Handle the B\[3:0\] field.
    ///
    /// Get and set the B\[3:0\] value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControl;
    ///
    /// let mut sft_rst_ctrl = SoftResetControl(0x0007_0000); // BM1366 default value
    /// assert_eq!(sft_rst_ctrl.b3_0(), 0);
    /// assert_eq!(sft_rst_ctrl.set_b3_0(0xf).b3_0(), 0xf); // max value
    /// assert_eq!(sft_rst_ctrl.set_b3_0(0x10).b3_0(), 0); // out of bound value
    /// ```
    pub const fn b3_0(&self) -> u8 {
        ((self.0 >> Self::B3_0_OFFSET) & Self::B3_0_MASK) as u8
    }
    pub fn set_b3_0(&mut self, b3_0: u8) -> &mut Self {
        self.0 &= !(Self::B3_0_MASK << Self::B3_0_OFFSET);
        self.0 |= ((b3_0 as u32) & Self::B3_0_MASK) << Self::B3_0_OFFSET;
        self
    }
}

impl core::fmt::Display for SoftResetControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RegA8").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SoftResetControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "RegA8 {{  }}",);
    }
}

/// # Soft Reset Control register, BM1372 layout
///
/// Same register as [`SoftResetControl`](crate::register::SoftResetControl), with a second group of core resets (`CORE_SRST1`, `CORE_SRST_FAST1`).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SoftResetControlV2(pub u32);
impl_boilerplate_for!(SoftResetControlV2);

impl SoftResetControlV2 {
    pub const ADDR: u8 = 0xA8;

    const CORE_SRST_FAST1_OFFSET: u8 = 28;
    const CORE_SRST1_OFFSET: u8 = 24;
    const SRST_TIME_OFFSET: u8 = 16;
    const CHIP_RESET_OFFSET: u8 = 10;
    const TOPCTRL_RESET_OFFSET: u8 = 9;
    const TVER_RESET_OFFSET: u8 = 8;
    const CORE_SRST_FAST_OFFSET: u8 = 4;
    const CORE_SRST_OFFSET: u8 = 0;

    const CORE_SRST_FAST1_MASK: u32 = 0b1111;
    const CORE_SRST1_MASK: u32 = 0b1111;
    const SRST_TIME_MASK: u32 = 0b11111111;
    const CHIP_RESET_MASK: u32 = 0b1;
    const TOPCTRL_RESET_MASK: u32 = 0b1;
    const TVER_RESET_MASK: u32 = 0b1;
    const CORE_SRST_FAST_MASK: u32 = 0b1111;
    const CORE_SRST_MASK: u32 = 0b1111;

    /// ## Handle the `CORE_SRST_FAST1` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.core_srst_fast1(), 0x0);
    /// assert_eq!(reg.set_core_srst_fast1(0xf).core_srst_fast1(), 0xf); // max value
    /// assert_eq!(reg.set_core_srst_fast1(0x10).core_srst_fast1(), 0x0); // out of bound value
    /// ```
    pub const fn core_srst_fast1(&self) -> u8 {
        ((self.0 >> Self::CORE_SRST_FAST1_OFFSET) & Self::CORE_SRST_FAST1_MASK) as u8
    }
    pub fn set_core_srst_fast1(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CORE_SRST_FAST1_MASK << Self::CORE_SRST_FAST1_OFFSET);
        self.0 |= ((value as u32) & Self::CORE_SRST_FAST1_MASK) << Self::CORE_SRST_FAST1_OFFSET;
        self
    }

    /// ## Handle the `CORE_SRST1` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.core_srst1(), 0x0);
    /// assert_eq!(reg.set_core_srst1(0xf).core_srst1(), 0xf); // max value
    /// assert_eq!(reg.set_core_srst1(0x10).core_srst1(), 0x0); // out of bound value
    /// ```
    pub const fn core_srst1(&self) -> u8 {
        ((self.0 >> Self::CORE_SRST1_OFFSET) & Self::CORE_SRST1_MASK) as u8
    }
    pub fn set_core_srst1(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CORE_SRST1_MASK << Self::CORE_SRST1_OFFSET);
        self.0 |= ((value as u32) & Self::CORE_SRST1_MASK) << Self::CORE_SRST1_OFFSET;
        self
    }

    /// ## Handle the `SRST_TIME` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.srst_time(), 0x7);
    /// assert_eq!(reg.set_srst_time(0xff).srst_time(), 0xff); // max value
    /// ```
    pub const fn srst_time(&self) -> u8 {
        ((self.0 >> Self::SRST_TIME_OFFSET) & Self::SRST_TIME_MASK) as u8
    }
    pub fn set_srst_time(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::SRST_TIME_MASK << Self::SRST_TIME_OFFSET);
        self.0 |= ((value as u32) & Self::SRST_TIME_MASK) << Self::SRST_TIME_OFFSET;
        self
    }

    /// ## Handle the `CHIP_RESET` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.chip_reset(), false);
    /// assert!(reg.set_chip_reset(true).chip_reset());
    /// assert!(!reg.set_chip_reset(false).chip_reset());
    /// ```
    pub const fn chip_reset(&self) -> bool {
        (self.0 >> Self::CHIP_RESET_OFFSET) & Self::CHIP_RESET_MASK != 0
    }
    pub fn set_chip_reset(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::CHIP_RESET_MASK << Self::CHIP_RESET_OFFSET);
        self.0 |= (value as u32) << Self::CHIP_RESET_OFFSET;
        self
    }

    /// ## Handle the `TOPCTRL_RESET` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.topctrl_reset(), false);
    /// assert!(reg.set_topctrl_reset(true).topctrl_reset());
    /// assert!(!reg.set_topctrl_reset(false).topctrl_reset());
    /// ```
    pub const fn topctrl_reset(&self) -> bool {
        (self.0 >> Self::TOPCTRL_RESET_OFFSET) & Self::TOPCTRL_RESET_MASK != 0
    }
    pub fn set_topctrl_reset(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TOPCTRL_RESET_MASK << Self::TOPCTRL_RESET_OFFSET);
        self.0 |= (value as u32) << Self::TOPCTRL_RESET_OFFSET;
        self
    }

    /// ## Handle the `TVER_RESET` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.tver_reset(), false);
    /// assert!(reg.set_tver_reset(true).tver_reset());
    /// assert!(!reg.set_tver_reset(false).tver_reset());
    /// ```
    pub const fn tver_reset(&self) -> bool {
        (self.0 >> Self::TVER_RESET_OFFSET) & Self::TVER_RESET_MASK != 0
    }
    pub fn set_tver_reset(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TVER_RESET_MASK << Self::TVER_RESET_OFFSET);
        self.0 |= (value as u32) << Self::TVER_RESET_OFFSET;
        self
    }

    /// ## Handle the `CORE_SRST_FAST` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.core_srst_fast(), 0x0);
    /// assert_eq!(reg.set_core_srst_fast(0xf).core_srst_fast(), 0xf); // max value
    /// assert_eq!(reg.set_core_srst_fast(0x10).core_srst_fast(), 0x0); // out of bound value
    /// ```
    pub const fn core_srst_fast(&self) -> u8 {
        ((self.0 >> Self::CORE_SRST_FAST_OFFSET) & Self::CORE_SRST_FAST_MASK) as u8
    }
    pub fn set_core_srst_fast(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CORE_SRST_FAST_MASK << Self::CORE_SRST_FAST_OFFSET);
        self.0 |= ((value as u32) & Self::CORE_SRST_FAST_MASK) << Self::CORE_SRST_FAST_OFFSET;
        self
    }

    /// ## Handle the `CORE_SRST` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::SoftResetControlV2;
    ///
    /// let mut reg = SoftResetControlV2(0x00070000); // reset value
    /// assert_eq!(reg.core_srst(), 0x0);
    /// assert_eq!(reg.set_core_srst(0xf).core_srst(), 0xf); // max value
    /// assert_eq!(reg.set_core_srst(0x10).core_srst(), 0x0); // out of bound value
    /// ```
    pub const fn core_srst(&self) -> u8 {
        ((self.0 >> Self::CORE_SRST_OFFSET) & Self::CORE_SRST_MASK) as u8
    }
    pub fn set_core_srst(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CORE_SRST_MASK << Self::CORE_SRST_OFFSET);
        self.0 |= ((value as u32) & Self::CORE_SRST_MASK) << Self::CORE_SRST_OFFSET;
        self
    }
}

impl core::fmt::Display for SoftResetControlV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SoftResetControlV2")
            .field("core_srst_fast1", &self.core_srst_fast1())
            .field("core_srst1", &self.core_srst1())
            .field("srst_time", &self.srst_time())
            .field("chip_reset", &self.chip_reset())
            .field("topctrl_reset", &self.topctrl_reset())
            .field("tver_reset", &self.tver_reset())
            .field("core_srst_fast", &self.core_srst_fast())
            .field("core_srst", &self.core_srst())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SoftResetControlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "SoftResetControlV2 {{ core_srst_fast1: {}, core_srst1: {}, srst_time: {}, chip_reset: {}, topctrl_reset: {}, tver_reset: {}, core_srst_fast: {}, core_srst: {} }}",
            self.core_srst_fast1(),
            self.core_srst1(),
            self.srst_time(),
            self.chip_reset(),
            self.topctrl_reset(),
            self.tver_reset(),
            self.core_srst_fast(),
            self.core_srst(),
        );
    }
}
