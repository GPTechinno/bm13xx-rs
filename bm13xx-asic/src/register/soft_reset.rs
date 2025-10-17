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
