use crate::register::Register;

/// # Temperature Sensor Control register (BM1368)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TempSensorCtrl(pub u32);
impl_boilerplate_for!(TempSensorCtrl);

impl TempSensorCtrl {
    pub const ADDR: u8 = 0xB0;

    const TS_PD_OFFSET: u8 = 31;
    const RUN_CTRL_OFFSET: u8 = 29;
    const RUN_EN_OFFSET: u8 = 28;
    const TS_CLOAD_OFFSET: u8 = 24;
    const TS_SOFT_RSTN_OFFSET: u8 = 17;
    const TS_CLK_SEL_OFFSET: u8 = 16;
    const CFGA_OFFSET: u8 = 8;
    const CFG0_OFFSET: u8 = 0;

    const TS_PD_MASK: u32 = 0b1;
    const RUN_CTRL_MASK: u32 = 0b1;
    const RUN_EN_MASK: u32 = 0b1;
    const TS_CLOAD_MASK: u32 = 0b1;
    const TS_SOFT_RSTN_MASK: u32 = 0b1;
    const TS_CLK_SEL_MASK: u32 = 0b1;
    const CFGA_MASK: u32 = 0b11111111;
    const CFG0_MASK: u32 = 0b11111111;

    /// ## Handle the `TS_PD` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.ts_pd(), true);
    /// assert!(reg.set_ts_pd(true).ts_pd());
    /// assert!(!reg.set_ts_pd(false).ts_pd());
    /// ```
    pub const fn ts_pd(&self) -> bool {
        (self.0 >> Self::TS_PD_OFFSET) & Self::TS_PD_MASK != 0
    }
    pub fn set_ts_pd(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_PD_MASK << Self::TS_PD_OFFSET);
        self.0 |= (value as u32) << Self::TS_PD_OFFSET;
        self
    }

    /// ## Handle the `RUN_CTRL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.run_ctrl(), false);
    /// assert!(reg.set_run_ctrl(true).run_ctrl());
    /// assert!(!reg.set_run_ctrl(false).run_ctrl());
    /// ```
    pub const fn run_ctrl(&self) -> bool {
        (self.0 >> Self::RUN_CTRL_OFFSET) & Self::RUN_CTRL_MASK != 0
    }
    pub fn set_run_ctrl(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RUN_CTRL_MASK << Self::RUN_CTRL_OFFSET);
        self.0 |= (value as u32) << Self::RUN_CTRL_OFFSET;
        self
    }

    /// ## Handle the `RUN_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.run_en(), false);
    /// assert!(reg.set_run_en(true).run_en());
    /// assert!(!reg.set_run_en(false).run_en());
    /// ```
    pub const fn run_en(&self) -> bool {
        (self.0 >> Self::RUN_EN_OFFSET) & Self::RUN_EN_MASK != 0
    }
    pub fn set_run_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RUN_EN_MASK << Self::RUN_EN_OFFSET);
        self.0 |= (value as u32) << Self::RUN_EN_OFFSET;
        self
    }

    /// ## Handle the `TS_CLOAD` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.ts_cload(), false);
    /// assert!(reg.set_ts_cload(true).ts_cload());
    /// assert!(!reg.set_ts_cload(false).ts_cload());
    /// ```
    pub const fn ts_cload(&self) -> bool {
        (self.0 >> Self::TS_CLOAD_OFFSET) & Self::TS_CLOAD_MASK != 0
    }
    pub fn set_ts_cload(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_CLOAD_MASK << Self::TS_CLOAD_OFFSET);
        self.0 |= (value as u32) << Self::TS_CLOAD_OFFSET;
        self
    }

    /// ## Handle the `TS_SOFT_RSTN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.ts_soft_rstn(), false);
    /// assert!(reg.set_ts_soft_rstn(true).ts_soft_rstn());
    /// assert!(!reg.set_ts_soft_rstn(false).ts_soft_rstn());
    /// ```
    pub const fn ts_soft_rstn(&self) -> bool {
        (self.0 >> Self::TS_SOFT_RSTN_OFFSET) & Self::TS_SOFT_RSTN_MASK != 0
    }
    pub fn set_ts_soft_rstn(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_SOFT_RSTN_MASK << Self::TS_SOFT_RSTN_OFFSET);
        self.0 |= (value as u32) << Self::TS_SOFT_RSTN_OFFSET;
        self
    }

    /// ## Handle the `TS_CLK_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.ts_clk_sel(), false);
    /// assert!(reg.set_ts_clk_sel(true).ts_clk_sel());
    /// assert!(!reg.set_ts_clk_sel(false).ts_clk_sel());
    /// ```
    pub const fn ts_clk_sel(&self) -> bool {
        (self.0 >> Self::TS_CLK_SEL_OFFSET) & Self::TS_CLK_SEL_MASK != 0
    }
    pub fn set_ts_clk_sel(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_CLK_SEL_MASK << Self::TS_CLK_SEL_OFFSET);
        self.0 |= (value as u32) << Self::TS_CLK_SEL_OFFSET;
        self
    }

    /// ## Handle the `CFGA` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.cfga(), 0x0);
    /// assert_eq!(reg.set_cfga(0xff).cfga(), 0xff); // max value
    /// ```
    pub const fn cfga(&self) -> u8 {
        ((self.0 >> Self::CFGA_OFFSET) & Self::CFGA_MASK) as u8
    }
    pub fn set_cfga(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CFGA_MASK << Self::CFGA_OFFSET);
        self.0 |= ((value as u32) & Self::CFGA_MASK) << Self::CFGA_OFFSET;
        self
    }

    /// ## Handle the `CFG0` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl;
    ///
    /// let mut reg = TempSensorCtrl(0x80000000); // reset value
    /// assert_eq!(reg.cfg0(), 0x0);
    /// assert_eq!(reg.set_cfg0(0xff).cfg0(), 0xff); // max value
    /// ```
    pub const fn cfg0(&self) -> u8 {
        ((self.0 >> Self::CFG0_OFFSET) & Self::CFG0_MASK) as u8
    }
    pub fn set_cfg0(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CFG0_MASK << Self::CFG0_OFFSET);
        self.0 |= ((value as u32) & Self::CFG0_MASK) << Self::CFG0_OFFSET;
        self
    }
}

impl core::fmt::Display for TempSensorCtrl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempSensorCtrl")
            .field("ts_pd", &self.ts_pd())
            .field("run_ctrl", &self.run_ctrl())
            .field("run_en", &self.run_en())
            .field("ts_cload", &self.ts_cload())
            .field("ts_soft_rstn", &self.ts_soft_rstn())
            .field("ts_clk_sel", &self.ts_clk_sel())
            .field("cfga", &self.cfga())
            .field("cfg0", &self.cfg0())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TempSensorCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TempSensorCtrl {{ ts_pd: {}, run_ctrl: {}, run_en: {}, ts_cload: {}, ts_soft_rstn: {}, ts_clk_sel: {}, cfga: {}, cfg0: {} }}",
            self.ts_pd(),
            self.run_ctrl(),
            self.run_en(),
            self.ts_cload(),
            self.ts_soft_rstn(),
            self.ts_clk_sel(),
            self.cfga(),
            self.cfg0(),
        );
    }
}

/// # Temperature Sensor Data register (BM1368)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TempSensorData(pub u32);
impl_boilerplate_for!(TempSensorData);

impl TempSensorData {
    pub const ADDR: u8 = 0xB4;

    const TS_DONE_OFFSET: u8 = 31;
    const TS_ERROR_OFFSET: u8 = 30;
    const DOUT_TYPE_OFFSET: u8 = 29;
    const DOUT_OFFSET: u8 = 0;

    const TS_DONE_MASK: u32 = 0b1;
    const TS_ERROR_MASK: u32 = 0b1;
    const DOUT_TYPE_MASK: u32 = 0b1;
    const DOUT_MASK: u32 = 0b1111111111111111;

    /// ## Handle the `TS_DONE` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData;
    ///
    /// let mut reg = TempSensorData(0x00000000); // reset value
    /// assert_eq!(reg.ts_done(), false);
    /// assert!(reg.set_ts_done(true).ts_done());
    /// assert!(!reg.set_ts_done(false).ts_done());
    /// ```
    pub const fn ts_done(&self) -> bool {
        (self.0 >> Self::TS_DONE_OFFSET) & Self::TS_DONE_MASK != 0
    }
    pub fn set_ts_done(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_DONE_MASK << Self::TS_DONE_OFFSET);
        self.0 |= (value as u32) << Self::TS_DONE_OFFSET;
        self
    }

    /// ## Handle the `TS_ERROR` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData;
    ///
    /// let mut reg = TempSensorData(0x00000000); // reset value
    /// assert_eq!(reg.ts_error(), false);
    /// assert!(reg.set_ts_error(true).ts_error());
    /// assert!(!reg.set_ts_error(false).ts_error());
    /// ```
    pub const fn ts_error(&self) -> bool {
        (self.0 >> Self::TS_ERROR_OFFSET) & Self::TS_ERROR_MASK != 0
    }
    pub fn set_ts_error(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_ERROR_MASK << Self::TS_ERROR_OFFSET);
        self.0 |= (value as u32) << Self::TS_ERROR_OFFSET;
        self
    }

    /// ## Handle the `DOUT_TYPE` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData;
    ///
    /// let mut reg = TempSensorData(0x00000000); // reset value
    /// assert_eq!(reg.dout_type(), false);
    /// assert!(reg.set_dout_type(true).dout_type());
    /// assert!(!reg.set_dout_type(false).dout_type());
    /// ```
    pub const fn dout_type(&self) -> bool {
        (self.0 >> Self::DOUT_TYPE_OFFSET) & Self::DOUT_TYPE_MASK != 0
    }
    pub fn set_dout_type(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::DOUT_TYPE_MASK << Self::DOUT_TYPE_OFFSET);
        self.0 |= (value as u32) << Self::DOUT_TYPE_OFFSET;
        self
    }

    /// ## Handle the `DOUT` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData;
    ///
    /// let mut reg = TempSensorData(0x00000000); // reset value
    /// assert_eq!(reg.dout(), 0x0);
    /// assert_eq!(reg.set_dout(0xffff).dout(), 0xffff); // max value
    /// ```
    pub const fn dout(&self) -> u16 {
        ((self.0 >> Self::DOUT_OFFSET) & Self::DOUT_MASK) as u16
    }
    pub fn set_dout(&mut self, value: u16) -> &mut Self {
        self.0 &= !(Self::DOUT_MASK << Self::DOUT_OFFSET);
        self.0 |= ((value as u32) & Self::DOUT_MASK) << Self::DOUT_OFFSET;
        self
    }

    /// ## Temperature in °C from `DOUT`
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData;
    ///
    /// assert_eq!(TempSensorData(0x8000_0000 | 1884).celsius() as i32, 25);
    /// assert_eq!(TempSensorData(0x8000_0000 | 2333).celsius() as i32, 99);
    /// ```
    pub fn celsius(&self) -> f32 {
        crate::temperature::ts_dout_to_celsius(self.dout())
    }
}

impl core::fmt::Display for TempSensorData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempSensorData")
            .field("ts_done", &self.ts_done())
            .field("ts_error", &self.ts_error())
            .field("dout_type", &self.dout_type())
            .field("dout", &self.dout())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TempSensorData {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TempSensorData {{ ts_done: {}, ts_error: {}, dout_type: {}, dout: {} }}",
            self.ts_done(),
            self.ts_error(),
            self.dout_type(),
            self.dout(),
        );
    }
}

/// # Temperature Sensor Control register (BM1372)
///
/// Same fields as [`TempSensorCtrl`], plus `TS_CLK_EN`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TempSensorCtrl0(pub u32);
impl_boilerplate_for!(TempSensorCtrl0);

impl TempSensorCtrl0 {
    pub const ADDR: u8 = 0xB2;

    const TS_PD_OFFSET: u8 = 31;
    const TS_CLK_EN_OFFSET: u8 = 30;
    const RUN_CTRL_OFFSET: u8 = 29;
    const RUN_EN_OFFSET: u8 = 28;
    const TS_CLOAD_OFFSET: u8 = 24;
    const TS_SOFT_RSTN_OFFSET: u8 = 17;
    const TS_CLK_SEL_OFFSET: u8 = 16;
    const CFGA_OFFSET: u8 = 8;
    const CFG0_OFFSET: u8 = 0;

    const TS_PD_MASK: u32 = 0b1;
    const TS_CLK_EN_MASK: u32 = 0b1;
    const RUN_CTRL_MASK: u32 = 0b1;
    const RUN_EN_MASK: u32 = 0b1;
    const TS_CLOAD_MASK: u32 = 0b1;
    const TS_SOFT_RSTN_MASK: u32 = 0b1;
    const TS_CLK_SEL_MASK: u32 = 0b1;
    const CFGA_MASK: u32 = 0b11111111;
    const CFG0_MASK: u32 = 0b11111111;

    /// ## Handle the `TS_PD` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.ts_pd(), true);
    /// assert!(reg.set_ts_pd(true).ts_pd());
    /// assert!(!reg.set_ts_pd(false).ts_pd());
    /// ```
    pub const fn ts_pd(&self) -> bool {
        (self.0 >> Self::TS_PD_OFFSET) & Self::TS_PD_MASK != 0
    }
    pub fn set_ts_pd(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_PD_MASK << Self::TS_PD_OFFSET);
        self.0 |= (value as u32) << Self::TS_PD_OFFSET;
        self
    }

    /// ## Handle the `TS_CLK_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.ts_clk_en(), false);
    /// assert!(reg.set_ts_clk_en(true).ts_clk_en());
    /// assert!(!reg.set_ts_clk_en(false).ts_clk_en());
    /// ```
    pub const fn ts_clk_en(&self) -> bool {
        (self.0 >> Self::TS_CLK_EN_OFFSET) & Self::TS_CLK_EN_MASK != 0
    }
    pub fn set_ts_clk_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_CLK_EN_MASK << Self::TS_CLK_EN_OFFSET);
        self.0 |= (value as u32) << Self::TS_CLK_EN_OFFSET;
        self
    }

    /// ## Handle the `RUN_CTRL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.run_ctrl(), false);
    /// assert!(reg.set_run_ctrl(true).run_ctrl());
    /// assert!(!reg.set_run_ctrl(false).run_ctrl());
    /// ```
    pub const fn run_ctrl(&self) -> bool {
        (self.0 >> Self::RUN_CTRL_OFFSET) & Self::RUN_CTRL_MASK != 0
    }
    pub fn set_run_ctrl(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RUN_CTRL_MASK << Self::RUN_CTRL_OFFSET);
        self.0 |= (value as u32) << Self::RUN_CTRL_OFFSET;
        self
    }

    /// ## Handle the `RUN_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.run_en(), false);
    /// assert!(reg.set_run_en(true).run_en());
    /// assert!(!reg.set_run_en(false).run_en());
    /// ```
    pub const fn run_en(&self) -> bool {
        (self.0 >> Self::RUN_EN_OFFSET) & Self::RUN_EN_MASK != 0
    }
    pub fn set_run_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::RUN_EN_MASK << Self::RUN_EN_OFFSET);
        self.0 |= (value as u32) << Self::RUN_EN_OFFSET;
        self
    }

    /// ## Handle the `TS_CLOAD` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.ts_cload(), false);
    /// assert!(reg.set_ts_cload(true).ts_cload());
    /// assert!(!reg.set_ts_cload(false).ts_cload());
    /// ```
    pub const fn ts_cload(&self) -> bool {
        (self.0 >> Self::TS_CLOAD_OFFSET) & Self::TS_CLOAD_MASK != 0
    }
    pub fn set_ts_cload(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_CLOAD_MASK << Self::TS_CLOAD_OFFSET);
        self.0 |= (value as u32) << Self::TS_CLOAD_OFFSET;
        self
    }

    /// ## Handle the `TS_SOFT_RSTN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.ts_soft_rstn(), false);
    /// assert!(reg.set_ts_soft_rstn(true).ts_soft_rstn());
    /// assert!(!reg.set_ts_soft_rstn(false).ts_soft_rstn());
    /// ```
    pub const fn ts_soft_rstn(&self) -> bool {
        (self.0 >> Self::TS_SOFT_RSTN_OFFSET) & Self::TS_SOFT_RSTN_MASK != 0
    }
    pub fn set_ts_soft_rstn(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_SOFT_RSTN_MASK << Self::TS_SOFT_RSTN_OFFSET);
        self.0 |= (value as u32) << Self::TS_SOFT_RSTN_OFFSET;
        self
    }

    /// ## Handle the `TS_CLK_SEL` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.ts_clk_sel(), false);
    /// assert!(reg.set_ts_clk_sel(true).ts_clk_sel());
    /// assert!(!reg.set_ts_clk_sel(false).ts_clk_sel());
    /// ```
    pub const fn ts_clk_sel(&self) -> bool {
        (self.0 >> Self::TS_CLK_SEL_OFFSET) & Self::TS_CLK_SEL_MASK != 0
    }
    pub fn set_ts_clk_sel(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_CLK_SEL_MASK << Self::TS_CLK_SEL_OFFSET);
        self.0 |= (value as u32) << Self::TS_CLK_SEL_OFFSET;
        self
    }

    /// ## Handle the `CFGA` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.cfga(), 0x0);
    /// assert_eq!(reg.set_cfga(0xff).cfga(), 0xff); // max value
    /// ```
    pub const fn cfga(&self) -> u8 {
        ((self.0 >> Self::CFGA_OFFSET) & Self::CFGA_MASK) as u8
    }
    pub fn set_cfga(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CFGA_MASK << Self::CFGA_OFFSET);
        self.0 |= ((value as u32) & Self::CFGA_MASK) << Self::CFGA_OFFSET;
        self
    }

    /// ## Handle the `CFG0` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorCtrl0;
    ///
    /// let mut reg = TempSensorCtrl0(0x80000000); // reset value
    /// assert_eq!(reg.cfg0(), 0x0);
    /// assert_eq!(reg.set_cfg0(0xff).cfg0(), 0xff); // max value
    /// ```
    pub const fn cfg0(&self) -> u8 {
        ((self.0 >> Self::CFG0_OFFSET) & Self::CFG0_MASK) as u8
    }
    pub fn set_cfg0(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CFG0_MASK << Self::CFG0_OFFSET);
        self.0 |= ((value as u32) & Self::CFG0_MASK) << Self::CFG0_OFFSET;
        self
    }
}

impl core::fmt::Display for TempSensorCtrl0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempSensorCtrl0")
            .field("ts_pd", &self.ts_pd())
            .field("ts_clk_en", &self.ts_clk_en())
            .field("run_ctrl", &self.run_ctrl())
            .field("run_en", &self.run_en())
            .field("ts_cload", &self.ts_cload())
            .field("ts_soft_rstn", &self.ts_soft_rstn())
            .field("ts_clk_sel", &self.ts_clk_sel())
            .field("cfga", &self.cfga())
            .field("cfg0", &self.cfg0())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TempSensorCtrl0 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TempSensorCtrl0 {{ ts_pd: {}, ts_clk_en: {}, run_ctrl: {}, run_en: {}, ts_cload: {}, ts_soft_rstn: {}, ts_clk_sel: {}, cfga: {}, cfg0: {} }}",
            self.ts_pd(),
            self.ts_clk_en(),
            self.run_ctrl(),
            self.run_en(),
            self.ts_cload(),
            self.ts_soft_rstn(),
            self.ts_clk_sel(),
            self.cfga(),
            self.cfg0(),
        );
    }
}

/// # Temperature Sensor Data register (BM1372)
///
/// Same fields as [`TempSensorData`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TempSensorData0(pub u32);
impl_boilerplate_for!(TempSensorData0);

impl TempSensorData0 {
    pub const ADDR: u8 = 0xB3;

    const TS_DONE_OFFSET: u8 = 31;
    const TS_ERROR_OFFSET: u8 = 30;
    const DOUT_TYPE_OFFSET: u8 = 29;
    const DOUT_OFFSET: u8 = 0;

    const TS_DONE_MASK: u32 = 0b1;
    const TS_ERROR_MASK: u32 = 0b1;
    const DOUT_TYPE_MASK: u32 = 0b1;
    const DOUT_MASK: u32 = 0b1111111111111111;

    /// ## Handle the `TS_DONE` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData0;
    ///
    /// let mut reg = TempSensorData0(0x00000000); // reset value
    /// assert_eq!(reg.ts_done(), false);
    /// assert!(reg.set_ts_done(true).ts_done());
    /// assert!(!reg.set_ts_done(false).ts_done());
    /// ```
    pub const fn ts_done(&self) -> bool {
        (self.0 >> Self::TS_DONE_OFFSET) & Self::TS_DONE_MASK != 0
    }
    pub fn set_ts_done(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_DONE_MASK << Self::TS_DONE_OFFSET);
        self.0 |= (value as u32) << Self::TS_DONE_OFFSET;
        self
    }

    /// ## Handle the `TS_ERROR` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData0;
    ///
    /// let mut reg = TempSensorData0(0x00000000); // reset value
    /// assert_eq!(reg.ts_error(), false);
    /// assert!(reg.set_ts_error(true).ts_error());
    /// assert!(!reg.set_ts_error(false).ts_error());
    /// ```
    pub const fn ts_error(&self) -> bool {
        (self.0 >> Self::TS_ERROR_OFFSET) & Self::TS_ERROR_MASK != 0
    }
    pub fn set_ts_error(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::TS_ERROR_MASK << Self::TS_ERROR_OFFSET);
        self.0 |= (value as u32) << Self::TS_ERROR_OFFSET;
        self
    }

    /// ## Handle the `DOUT_TYPE` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData0;
    ///
    /// let mut reg = TempSensorData0(0x00000000); // reset value
    /// assert_eq!(reg.dout_type(), false);
    /// assert!(reg.set_dout_type(true).dout_type());
    /// assert!(!reg.set_dout_type(false).dout_type());
    /// ```
    pub const fn dout_type(&self) -> bool {
        (self.0 >> Self::DOUT_TYPE_OFFSET) & Self::DOUT_TYPE_MASK != 0
    }
    pub fn set_dout_type(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::DOUT_TYPE_MASK << Self::DOUT_TYPE_OFFSET);
        self.0 |= (value as u32) << Self::DOUT_TYPE_OFFSET;
        self
    }

    /// ## Handle the `DOUT` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData0;
    ///
    /// let mut reg = TempSensorData0(0x00000000); // reset value
    /// assert_eq!(reg.dout(), 0x0);
    /// assert_eq!(reg.set_dout(0xffff).dout(), 0xffff); // max value
    /// ```
    pub const fn dout(&self) -> u16 {
        ((self.0 >> Self::DOUT_OFFSET) & Self::DOUT_MASK) as u16
    }
    pub fn set_dout(&mut self, value: u16) -> &mut Self {
        self.0 &= !(Self::DOUT_MASK << Self::DOUT_OFFSET);
        self.0 |= ((value as u32) & Self::DOUT_MASK) << Self::DOUT_OFFSET;
        self
    }

    /// ## Temperature in °C from `DOUT`
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::TempSensorData0;
    ///
    /// assert_eq!(TempSensorData0(0x8000_0000 | 1884).celsius() as i32, 25);
    /// assert_eq!(TempSensorData0(0x8000_0000 | 2333).celsius() as i32, 99);
    /// ```
    pub fn celsius(&self) -> f32 {
        crate::temperature::ts_dout_to_celsius(self.dout())
    }
}

impl core::fmt::Display for TempSensorData0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempSensorData0")
            .field("ts_done", &self.ts_done())
            .field("ts_error", &self.ts_error())
            .field("dout_type", &self.dout_type())
            .field("dout", &self.dout())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TempSensorData0 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TempSensorData0 {{ ts_done: {}, ts_error: {}, dout_type: {}, dout: {} }}",
            self.ts_done(),
            self.ts_error(),
            self.dout_type(),
            self.dout(),
        );
    }
}
