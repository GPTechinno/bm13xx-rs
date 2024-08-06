use core::f64;

use fugit::HertzU64;

pub const PLL_OUT_MAX: usize = 5;
const PLL_VCO_FREQ_MAX: HertzU64 = HertzU64::MHz(3200);
const PLL_VCO_FREQ_HIGH: HertzU64 = HertzU64::MHz(2400);
const PLL_VCO_FREQ_MIN: HertzU64 = HertzU64::MHz(2000);

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Pll {
    enabled: bool,
    locked: bool,
    vco_high_freq: bool,
    fb_div: u16,
    ref_div: u8,
    post1_div: u8,
    post2_div: u8,
    out_div: [u8; PLL_OUT_MAX],
}

impl Pll {
    const LOCKED_OFFSET: u8 = 31;
    const PLLEN_OFFSET: u8 = 30;
    const VCO_HIGH_FREQ_OFFSET: u8 = 28;
    const FBDIV_OFFSET: u8 = 16;
    const REFDIV_OFFSET: u8 = 8;
    const POSTDIV1_OFFSET: u8 = 4;
    const POSTDIV2_OFFSET: u8 = 0;

    const LOCKED_MASK: u32 = 0x1;
    const PLLEN_MASK: u32 = 0x1;
    const VCO_HIGH_FREQ_MASK: u32 = 0x1;
    const FBDIV_MASK: u32 = 0xfff;
    const REFDIV_MASK: u32 = 0x3f;
    const POSTDIV1_MASK: u32 = 0x7;
    const POSTDIV2_MASK: u32 = 0x7;

    /// ## Handle the PLL Parameter.
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
        ((self.locked as u32) << Self::LOCKED_OFFSET)
            | ((self.enabled as u32) << Self::PLLEN_OFFSET)
            | ((self.vco_high_freq as u32) << Self::VCO_HIGH_FREQ_OFFSET)
            | ((self.fb_div as u32) << Self::FBDIV_OFFSET)
            | ((self.ref_div as u32) << Self::REFDIV_OFFSET)
            | ((self.post1_div as u32) << Self::POSTDIV1_OFFSET)
            | ((self.post2_div as u32) << Self::POSTDIV2_OFFSET)
    }
    pub fn set_parameter(&mut self, parameter: u32) -> &mut Self {
        self.locked = (parameter >> Self::LOCKED_OFFSET) & Self::LOCKED_MASK != 0;
        self.enabled = (parameter >> Self::PLLEN_OFFSET) & Self::PLLEN_MASK != 0;
        self.vco_high_freq =
            (parameter >> Self::VCO_HIGH_FREQ_OFFSET) & Self::VCO_HIGH_FREQ_MASK != 0;
        self.fb_div = ((parameter >> Self::FBDIV_OFFSET) & Self::FBDIV_MASK) as u16;
        self.ref_div = ((parameter >> Self::REFDIV_OFFSET) & Self::REFDIV_MASK) as u8;
        self.post1_div = ((parameter >> Self::POSTDIV1_OFFSET) & Self::POSTDIV1_MASK) as u8;
        self.post2_div = ((parameter >> Self::POSTDIV2_OFFSET) & Self::POSTDIV2_MASK) as u8;
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
        if out < PLL_OUT_MAX {
            self.out_div[out]
        } else {
            0
        }
    }
    pub fn set_out_div(&mut self, out: usize, div: u8) -> &mut Self {
        if out < PLL_OUT_MAX {
            self.out_div[out] = div & 0xf;
        }
        self
    }

    /// ## Get the PLL VCO Frequency.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU64;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// assert_eq!(pll.vco_freq(HertzU64::MHz(25)), HertzU64::MHz(2400));
    /// pll.set_parameter(0x0064_0111); // BM1397 PLL1 default value
    /// assert_eq!(pll.vco_freq(HertzU64::MHz(25)), HertzU64::MHz(0));
    /// ```
    pub fn vco_freq(&self, in_clk_freq: HertzU64) -> HertzU64 {
        if self.enabled && self.locked {
            in_clk_freq * (self.fb_div as u32) / (self.ref_div as u32)
        } else {
            HertzU64::MHz(0)
        }
    }

    /// ## Get the PLL Frequency for a given output.
    ///
    /// ### Example
    /// ```
    /// use fugit::HertzU64;
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let clki = HertzU64::MHz(25);
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// pll.set_divider(0x0304_0607); // BM1397 PLL0 default divider
    /// assert_eq!(pll.frequency(clki, 0), HertzU64::Hz(21428571));
    /// assert_eq!(pll.frequency(clki, 5), HertzU64::MHz(0));
    /// assert_eq!(pll.set_frequency(clki, 0, HertzU64::MHz(425)).frequency(clki, 0), HertzU64::MHz(425));
    /// pll.set_parameter(0x0064_0111); // BM1397 PLL1 default value
    /// pll.set_divider(0x0304_0506); // BM1397 PLL1 default divider
    /// assert_eq!(pll.frequency(clki, 0), HertzU64::MHz(0));
    /// ```
    pub fn frequency(&self, in_clk_freq: HertzU64, out: usize) -> HertzU64 {
        if out < PLL_OUT_MAX {
            self.vco_freq(in_clk_freq)
                / ((self.post1_div as u32 + 1)
                    * (self.post2_div as u32 + 1)
                    * (self.out_div[out] as u32 + 1))
        } else {
            HertzU64::MHz(0)
        }
    }
    pub fn set_frequency(
        &mut self,
        in_clk_freq: HertzU64,
        out: usize,
        target_freq: HertzU64,
    ) -> &mut Self {
        if out < PLL_OUT_MAX {
            let mut pll = *self;
            pll.out_div[out] = 0;
            for ref_div in (1..=2).rev() {
                pll.ref_div = ref_div;
                for post2_div in 0..=7 {
                    pll.post2_div = post2_div;
                    for post1_div in post2_div..=7 {
                        pll.post1_div = post1_div;
                        let fb_div = (((post1_div + 1) as f64
                            * (post2_div + 1) as f64
                            * target_freq.raw() as f64
                            * ref_div as f64
                            / in_clk_freq.raw() as f64)
                            + 0.5) as u16;
                        if fb_div < 251 {
                            pll.fb_div = fb_div;
                            pll.enable().lock();
                            let vco_freq = pll.vco_freq(in_clk_freq);
                            pll.vco_high_freq = vco_freq > PLL_VCO_FREQ_HIGH;
                            if (pll.ref_div > 1 || vco_freq <= HertzU64::MHz(3125))
                                && (vco_freq <= PLL_VCO_FREQ_MAX)
                                && (vco_freq > PLL_VCO_FREQ_MIN)
                            {
                                let freq_diff = if target_freq > pll.frequency(in_clk_freq, out) {
                                    target_freq - pll.frequency(in_clk_freq, out)
                                } else {
                                    pll.frequency(in_clk_freq, out) - target_freq
                                };
                                if freq_diff < HertzU64::MHz(1) {
                                    *self = pll;
                                    return self;
                                }
                            }
                        }
                    }
                }
            }
        }
        self
    }

    /// ## Handle the PLL locked field.
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

    /// ## Handle the PLL enabled field.
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

    /// ## Handle the PLL vco_high_freq field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// assert!(!pll.vco_high_freq());
    /// assert!(pll.set_vco_high_freq().vco_high_freq());
    /// assert!(!pll.set_vco_low_freq().vco_high_freq());
    /// ```
    pub const fn vco_high_freq(&self) -> bool {
        self.vco_high_freq
    }
    pub fn set_vco_high_freq(&mut self) -> &mut Self {
        self.vco_high_freq = true;
        self
    }
    pub fn set_vco_low_freq(&mut self) -> &mut Self {
        self.vco_high_freq = false;
        self
    }

    /// ## Handle the PLL FB Divider field.
    ///
    /// Get and set the PLL FB Divider value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// assert_eq!(pll.fb_div(), 96);
    /// assert!(pll.set_fb_div(0).fb_div() == 0); // min value
    /// assert!(pll.set_fb_div(0xfff).fb_div() == 0xfff); // max value
    /// assert!(pll.set_fb_div(0x1000).fb_div() == 0); // out of bound value
    /// ```
    pub const fn fb_div(&self) -> u16 {
        self.fb_div
    }
    pub fn set_fb_div(&mut self, fb_div: u16) -> &mut Self {
        self.fb_div = fb_div & Self::FBDIV_MASK as u16;
        self
    }

    /// ## Handle the PLL REF Divider field.
    ///
    /// Get and set the PLL REF Divider value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// assert_eq!(pll.ref_div(), 1);
    /// assert!(pll.set_ref_div(0).ref_div() == 0); // min value
    /// assert!(pll.set_ref_div(0x3f).ref_div() == 0x3f); // max value
    /// assert!(pll.set_ref_div(0x40).ref_div() == 0); // out of bound value
    /// ```
    pub const fn ref_div(&self) -> u8 {
        self.ref_div
    }
    pub fn set_ref_div(&mut self, ref_div: u8) -> &mut Self {
        self.ref_div = ref_div & Self::REFDIV_MASK as u8;
        self
    }

    /// ## Handle the PLL POST Divider 1 field.
    ///
    /// Get and set the PLL POST Divider 1 value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// assert_eq!(pll.post1_div(), 6);
    /// assert!(pll.set_post1_div(0).post1_div() == 0); // min value
    /// assert!(pll.set_post1_div(0x7).post1_div() == 0x7); // max value
    /// assert!(pll.set_post1_div(0x8).post1_div() == 0); // out of bound value
    /// ```
    pub const fn post1_div(&self) -> u8 {
        self.post1_div
    }
    pub fn set_post1_div(&mut self, post1_div: u8) -> &mut Self {
        self.post1_div = post1_div & Self::POSTDIV1_MASK as u8;
        self
    }

    /// ## Handle the PLL POST Divider 2 field.
    ///
    /// Get and set the PLL POST Divider 2 value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::pll::Pll;
    ///
    /// let mut pll = Pll::default();
    /// pll.set_parameter(0xC060_0161); // BM1397 PLL0 default value
    /// assert_eq!(pll.post2_div(), 1);
    /// assert!(pll.set_post2_div(0).post2_div() == 0); // min value
    /// assert!(pll.set_post2_div(0x7).post2_div() == 0x7); // max value
    /// assert!(pll.set_post2_div(0x8).post2_div() == 0); // out of bound value
    /// ```
    pub const fn post2_div(&self) -> u8 {
        self.post2_div
    }
    pub fn set_post2_div(&mut self, post2_div: u8) -> &mut Self {
        self.post2_div = post2_div & Self::POSTDIV2_MASK as u8;
        self
    }
}
