use crate::register::Register;

/// # Fast UART Configuration Add register (BM1372)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FastUARTConfigurationAdd(pub u32);
impl_boilerplate_for!(FastUARTConfigurationAdd);

impl FastUARTConfigurationAdd {
    pub const ADDR: u8 = 0x29;

    const DSCLK_SEL_OFFSET: u8 = 25;
    const DS_ODDSET_OFFSET: u8 = 23;
    const DS_DIV_OFFSET: u8 = 16;
    const TOPCLK_SEL_OFFSET: u8 = 9;
    const TOP_ODDSET_OFFSET: u8 = 7;
    const TOP_DIV_OFFSET: u8 = 0;

    const DSCLK_SEL_MASK: u32 = 0b11111;
    const DS_ODDSET_MASK: u32 = 0b11;
    const DS_DIV_MASK: u32 = 0b111111;
    const TOPCLK_SEL_MASK: u32 = 0b1111;
    const TOP_ODDSET_MASK: u32 = 0b11;
    const TOP_DIV_MASK: u32 = 0b111111;

    /// ## Handle the `DSCLK_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.dsclk_sel(), 0x0);
    /// assert_eq!(reg.set_dsclk_sel(0x1f).dsclk_sel(), 0x1f); // max value
    /// assert_eq!(reg.set_dsclk_sel(0x20).dsclk_sel(), 0x0); // out of bound value
    /// ```
    pub const fn dsclk_sel(&self) -> u8 {
        ((self.0 >> Self::DSCLK_SEL_OFFSET) & Self::DSCLK_SEL_MASK) as u8
    }
    pub fn set_dsclk_sel(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::DSCLK_SEL_MASK << Self::DSCLK_SEL_OFFSET);
        self.0 |= ((value as u32) & Self::DSCLK_SEL_MASK) << Self::DSCLK_SEL_OFFSET;
        self
    }

    /// ## Handle the `DS_ODDSET` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.ds_oddset(), 0x0);
    /// assert_eq!(reg.set_ds_oddset(0x3).ds_oddset(), 0x3); // max value
    /// assert_eq!(reg.set_ds_oddset(0x4).ds_oddset(), 0x0); // out of bound value
    /// ```
    pub const fn ds_oddset(&self) -> u8 {
        ((self.0 >> Self::DS_ODDSET_OFFSET) & Self::DS_ODDSET_MASK) as u8
    }
    pub fn set_ds_oddset(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::DS_ODDSET_MASK << Self::DS_ODDSET_OFFSET);
        self.0 |= ((value as u32) & Self::DS_ODDSET_MASK) << Self::DS_ODDSET_OFFSET;
        self
    }

    /// ## Handle the `DS_DIV` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.ds_div(), 0x0);
    /// assert_eq!(reg.set_ds_div(0x3f).ds_div(), 0x3f); // max value
    /// assert_eq!(reg.set_ds_div(0x40).ds_div(), 0x0); // out of bound value
    /// ```
    pub const fn ds_div(&self) -> u8 {
        ((self.0 >> Self::DS_DIV_OFFSET) & Self::DS_DIV_MASK) as u8
    }
    pub fn set_ds_div(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::DS_DIV_MASK << Self::DS_DIV_OFFSET);
        self.0 |= ((value as u32) & Self::DS_DIV_MASK) << Self::DS_DIV_OFFSET;
        self
    }

    /// ## Handle the `TOPCLK_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.topclk_sel(), 0x0);
    /// assert_eq!(reg.set_topclk_sel(0xf).topclk_sel(), 0xf); // max value
    /// assert_eq!(reg.set_topclk_sel(0x10).topclk_sel(), 0x0); // out of bound value
    /// ```
    pub const fn topclk_sel(&self) -> u8 {
        ((self.0 >> Self::TOPCLK_SEL_OFFSET) & Self::TOPCLK_SEL_MASK) as u8
    }
    pub fn set_topclk_sel(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::TOPCLK_SEL_MASK << Self::TOPCLK_SEL_OFFSET);
        self.0 |= ((value as u32) & Self::TOPCLK_SEL_MASK) << Self::TOPCLK_SEL_OFFSET;
        self
    }

    /// ## Handle the `TOP_ODDSET` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.top_oddset(), 0x0);
    /// assert_eq!(reg.set_top_oddset(0x3).top_oddset(), 0x3); // max value
    /// assert_eq!(reg.set_top_oddset(0x4).top_oddset(), 0x0); // out of bound value
    /// ```
    pub const fn top_oddset(&self) -> u8 {
        ((self.0 >> Self::TOP_ODDSET_OFFSET) & Self::TOP_ODDSET_MASK) as u8
    }
    pub fn set_top_oddset(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::TOP_ODDSET_MASK << Self::TOP_ODDSET_OFFSET);
        self.0 |= ((value as u32) & Self::TOP_ODDSET_MASK) << Self::TOP_ODDSET_OFFSET;
        self
    }

    /// ## Handle the `TOP_DIV` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfigurationAdd;
    ///
    /// let mut reg = FastUARTConfigurationAdd(0x00000000); // reset value
    /// assert_eq!(reg.top_div(), 0x0);
    /// assert_eq!(reg.set_top_div(0x3f).top_div(), 0x3f); // max value
    /// assert_eq!(reg.set_top_div(0x40).top_div(), 0x0); // out of bound value
    /// ```
    pub const fn top_div(&self) -> u8 {
        ((self.0 >> Self::TOP_DIV_OFFSET) & Self::TOP_DIV_MASK) as u8
    }
    pub fn set_top_div(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::TOP_DIV_MASK << Self::TOP_DIV_OFFSET);
        self.0 |= ((value as u32) & Self::TOP_DIV_MASK) << Self::TOP_DIV_OFFSET;
        self
    }
}

impl core::fmt::Display for FastUARTConfigurationAdd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FastUARTConfigurationAdd")
            .field("dsclk_sel", &self.dsclk_sel())
            .field("ds_oddset", &self.ds_oddset())
            .field("ds_div", &self.ds_div())
            .field("topclk_sel", &self.topclk_sel())
            .field("top_oddset", &self.top_oddset())
            .field("top_div", &self.top_div())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for FastUARTConfigurationAdd {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "FastUARTConfigurationAdd {{ dsclk_sel: {}, ds_oddset: {}, ds_div: {}, topclk_sel: {}, top_oddset: {}, top_div: {} }}",
            self.dsclk_sel(),
            self.ds_oddset(),
            self.ds_div(),
            self.topclk_sel(),
            self.top_oddset(),
            self.top_div(),
        );
    }
}
