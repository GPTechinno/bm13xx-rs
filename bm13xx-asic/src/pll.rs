use fugit::HertzU32;

#[derive(Debug, Clone, Copy)]
pub struct Pll {
    in_clk_freq: HertzU32,
    fb_div: u16,
    ref_div: u8,
    post_div_1: u8,
    post_div_2: u8,
    enabled: bool,
}

impl Pll {
    pub fn new(in_clk_freq: HertzU32) -> Self {
        Self {
            in_clk_freq,
            fb_div: 0,
            ref_div: 0,
            post_div_1: 0,
            post_div_2: 0,
            enabled: false,
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

    /// ## Set the PLL Dividers.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_dividers(0x60, 1, 6, 1); // BM1397 PLL0 default values
    /// ```
    pub fn set_dividers(&mut self, fb_div: u16, ref_div: u8, post_div_1: u8, post_div_2: u8) {
        self.fb_div = fb_div;
        self.ref_div = ref_div;
        self.post_div_1 = post_div_1;
        self.post_div_2 = post_div_2;
    }

    /// ## Get the PLL Frequency.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU32;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_dividers(0x60, 1, 6, 1); // BM1397 PLL0 default values
    /// assert_eq!(pll.frequency(), HertzU32::MHz(0u32));
    /// pll.enable();
    /// assert_eq!(pll.frequency(), HertzU32::MHz(400u32));
    /// ```
    pub fn frequency(&self) -> HertzU32 {
        if self.enabled {
            self.in_clk_freq * (self.fb_div as u32)
                / ((self.ref_div as u32) * (self.post_div_1 as u32) * (self.post_div_2 as u32))
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
