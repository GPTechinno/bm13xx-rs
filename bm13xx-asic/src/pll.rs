use fugit::HertzU32;

#[derive(Debug, Clone, Copy, Default)]
pub struct Pll {
    enabled: bool,
    locked: bool,
    fb_div: u16,
    ref_div: u8,
    post_div_1: u8,
    post_div_2: u8,
    out_div: [u8; 5],
}

impl Pll {
    /// ## Set the PLL Parameter.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert_eq!(pll.set_parameter(0xC060_0161).parameter(), 0xC060_0161); // BM1397 PLL0 default parameter
    /// assert_eq!(pll.set_parameter(0x0064_0111).parameter(), 0x0064_0111); // BM1397 PLL1 default parameter
    /// ```
    pub const fn parameter(&self) -> u32 {
        ((self.locked as u32) << 31)
            | ((self.enabled as u32) << 30)
            | ((self.fb_div as u32) << 16)
            | ((self.ref_div as u32) << 8)
            | ((self.post_div_1 as u32) << 4)
            | (self.post_div_2 as u32)
    }
    pub fn set_parameter(&mut self, parameter: u32) -> &mut Self {
        self.locked = parameter & 0x8000_0000 != 0;
        self.enabled = parameter & 0x4000_0000 != 0;
        self.fb_div = ((parameter >> 16) & 0xfff) as u16;
        self.ref_div = ((parameter >> 8) & 0x3f) as u8;
        self.post_div_1 = ((parameter >> 4) & 0x7) as u8;
        self.post_div_2 = (parameter & 0x7) as u8;
        self
    }

    /// ## Set the PLL Divider.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert_eq!(pll.set_divider(0x0304_0607).divider(), 0x0304_0607); // BM1397 PLL0 default divider
    /// assert_eq!(pll.set_divider(0x0304_0506).divider(), 0x0304_0506); // BM1397 PLL1 default divider
    /// ```
    pub const fn divider(&self) -> u32 {
        ((self.out_div[3] as u32) << 24)
            | ((self.out_div[2] as u32) << 16)
            | ((self.out_div[1] as u32) << 8)
            | (self.out_div[0] as u32)
    }
    pub fn set_divider(&mut self, divider: u32) -> &mut Self {
        self.out_div[3] = ((divider >> 24) & 0xf) as u8;
        self.out_div[2] = ((divider >> 16) & 0xf) as u8;
        self.out_div[1] = ((divider >> 8) & 0xf) as u8;
        self.out_div[0] = (divider & 0xf) as u8;
        self
    }

    /// ## Set the PLL Divider.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert_eq!(pll.set_out_div(4, 0).out_div(4), 0); // BM1397 PLL3_DIV4 default value
    /// assert_eq!(pll.set_out_div(3, 15).out_div(3), 15); // max value
    /// assert_eq!(pll.set_out_div(2, 16).out_div(2), 0); // value out of bound
    /// assert_eq!(pll.set_out_div(5, 10).out_div(5), 0); // index out of bound
    /// ```
    pub const fn out_div(&self, out: usize) -> u8 {
        if out < 5 {
            self.out_div[out]
        } else {
            0
        }
    }
    pub fn set_out_div(&mut self, out: usize, div: u8) -> &mut Self {
        if out < 5 {
            self.out_div[out] = div & 0xf;
        }
        self
    }

    /// ## Get the PLL Frequency for a given output.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default values
    /// assert_eq!(pll.frequency(HertzU32::MHz(25u32), 0), HertzU32::MHz(400u32));
    /// assert_eq!(pll.frequency(HertzU32::MHz(25u32), 5), HertzU32::MHz(0u32));
    /// pll.set_parameter(0x0064_0111); // BM1397 PLL1 default values
    /// assert_eq!(pll.frequency(HertzU32::MHz(25u32), 0), HertzU32::MHz(0u32));
    /// ```
    pub fn frequency(&self, in_clk_freq: HertzU32, out: usize) -> HertzU32 {
        if self.enabled && self.locked && out < 5 {
            in_clk_freq * (self.fb_div as u32)
                / ((self.ref_div as u32)
                    * (self.post_div_1 as u32)
                    * (self.post_div_2 as u32)
                    * (self.out_div[out] as u32 + 1))
        } else {
            HertzU32::MHz(0)
        }
    }

    /// ## Check if the PLL is enabled.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert!(!pll.locked());
    /// assert!(pll.lock().locked());
    /// assert!(!pll.unlock().locked());
    /// ```
    pub const fn locked(&self) -> bool {
        self.locked
    }
    pub fn lock(&mut self) -> &mut Self {
        self.locked = true;
        self
    }
    pub fn unlock(&mut self) -> &mut Self {
        self.locked = false;
        self
    }

    /// ## Check if the PLL is enabled.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert!(!pll.enabled());
    /// assert!(pll.enable().enabled());
    /// assert!(!pll.disable().enabled());
    /// ```
    pub const fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn enable(&mut self) -> &mut Self {
        self.enabled = true;
        self
    }
    pub fn disable(&mut self) -> &mut Self {
        self.enabled = false;
        self
    }
}
