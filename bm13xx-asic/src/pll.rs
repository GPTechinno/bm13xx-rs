use fugit::HertzU32;

#[derive(Debug, Clone, Copy)]
pub struct Pll {
    in_clk_freq: HertzU32,
    enabled: bool,
    locked: bool,
    fb_div: u16,
    ref_div: u8,
    post_div_1: u8,
    post_div_2: u8,
    out_div: [u8; 5],
}

impl Pll {
    pub fn new(in_clk_freq: HertzU32) -> Self {
        Self {
            in_clk_freq,
            enabled: false,
            locked: false,
            fb_div: 0,
            ref_div: 0,
            post_div_1: 0,
            post_div_2: 0,
            out_div: [0; 5],
        }
    }

    /// ## Get the Input Clock Frequency.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let pll = Pll::new(HertzU32::MHz(25));
    /// assert_eq!(pll.input_clk_freq(), HertzU32::MHz(25u32));
    /// ```
    pub fn input_clk_freq(&self) -> HertzU32 {
        self.in_clk_freq
    }

    /// ## Set the Input Clock Frequency.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::new(HertzU32::MHz(25));
    /// pll.set_input_clk_freq(HertzU32::MHz(20));
    /// assert_eq!(pll.input_clk_freq(), HertzU32::MHz(20u32));
    /// ```
    pub fn set_input_clk_freq(&mut self, in_clk_freq: HertzU32) {
        self.in_clk_freq = in_clk_freq;
    }

    /// ## Set the PLL Parameter.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default parameter
    /// assert!(pll.enabled());
    /// assert!(pll.locked());
    /// pll.set_parameter(0x0064_0111); // BM1397 PLL1 default parameter
    /// assert!(!pll.enabled());
    /// assert!(!pll.locked());
    /// ```
    pub fn set_parameter(&mut self, parameter: u32) {
        self.locked = parameter & 0x4000_0000 != 0;
        self.enabled = parameter & 0x4000_0000 != 0;
        self.fb_div = ((parameter >> 16) & 0xfff) as u16;
        self.ref_div = ((parameter >> 8) & 0x3f) as u8;
        self.post_div_1 = ((parameter >> 4) & 0x7) as u8;
        self.post_div_2 = (parameter & 0x7) as u8;
    }

    /// ## Set the PLL Divider.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_divider(0x0304_0607); // BM1397 PLL0 default divider
    /// pll.set_divider(0x0304_0506); // BM1397 PLL1 default divider
    /// ```
    pub fn set_divider(&mut self, divider: u32) {
        self.out_div[3] = ((divider >> 24) & 0xf) as u8;
        self.out_div[2] = ((divider >> 16) & 0xf) as u8;
        self.out_div[1] = ((divider >> 8) & 0xf) as u8;
        self.out_div[0] = (divider & 0xf) as u8;
    }

    /// ## Set the PLL Divider.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_out_div_4(0); // BM1397 PLL3_DIV4 default value
    /// ```
    pub fn set_out_div_4(&mut self, div4: u8) {
        self.out_div[4] = div4 & 0xf;
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
    /// assert_eq!(pll.frequency(0), HertzU32::MHz(400u32));
    /// assert_eq!(pll.frequency(5), HertzU32::MHz(0u32));
    /// pll.set_parameter(0x0064_0111); // BM1397 PLL1 default values
    /// assert_eq!(pll.frequency(0), HertzU32::MHz(0u32));
    /// ```
    pub fn frequency(&self, out: usize) -> HertzU32 {
        if self.enabled && self.locked && out < 5 {
            self.in_clk_freq * (self.fb_div as u32)
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
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let pll = Pll::new(HertzU32::MHz(25));
    /// assert!(!pll.locked());
    /// ```
    pub fn locked(&self) -> bool {
        self.locked
    }

    /// ## Check if the PLL is enabled.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let pll = Pll::new(HertzU32::MHz(25));
    /// assert!(!pll.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// ## Enable the PLL.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::new(HertzU32::MHz(25));
    /// pll.enable();
    /// assert!(pll.enabled());
    /// ```
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// ## Disable the PLL.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::new(HertzU32::MHz(25));
    /// pll.disable();
    /// assert!(!pll.enabled());
    /// ```
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for Pll {
    fn default() -> Self {
        Self::new(HertzU32::MHz(25))
    }
}
