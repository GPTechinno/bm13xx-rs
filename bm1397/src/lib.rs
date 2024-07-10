use fugit::HertzU32;

pub const BM1397_CORE_CNT: usize = 168;
pub const BM1397_SMALL_CORE_CNT: usize = 672;
pub const BM1397_CORE_SMALL_CORE_CNT: usize = 4;
pub const BM1397_DOMAIN_CNT: usize = 4;
pub const BM1397_PLL_CNT: usize = 4;

/// # BM1397
#[derive(Debug)]
pub struct BM1397 {
    pub sha: bm13xx_asic::sha::Asic<
        BM1397_CORE_CNT,
        BM1397_SMALL_CORE_CNT,
        BM1397_CORE_SMALL_CORE_CNT,
        BM1397_DOMAIN_CNT,
    >,
    pub plls: [bm13xx_asic::pll::Pll; BM1397_PLL_CNT],
}

impl BM1397 {
    pub fn new_with_clk(clk: HertzU32) -> Self {
        let mut bm1397 = Self::default();
        bm1397
            .plls
            .iter_mut()
            .for_each(|pll| pll.set_input_clk_freq(clk));
        bm1397
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use fugit::HertzU32;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.hash_freq(), HertzU32::MHz(400u32));
    /// ```
    pub fn hash_freq(&self) -> HertzU32 {
        self.plls[0].frequency()
    }

    /// ## Get the theoretical Hashrate in GH/s
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use fugit::HertzU32;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.theoretical_hashrate_ghs(), 268.8);
    /// ```
    pub fn theoretical_hashrate_ghs(&self) -> f32 {
        self.hash_freq().raw() as f32 * self.sha.small_core_count() as f32 / 1_000_000_000.0
    }
}

impl Default for BM1397 {
    fn default() -> Self {
        let mut bm1397 = Self {
            sha: bm13xx_asic::sha::Asic::default(),
            plls: [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT],
        };
        bm1397.plls[0].set_dividers(0x60, 1, 6, 1);
        bm1397.plls[0].enable();
        bm1397.plls[1].set_dividers(0x64, 1, 1, 1);
        bm1397.plls[2].set_dividers(0x68, 1, 1, 1);
        bm1397.plls[3].set_dividers(0x70, 1, 1, 1);
        bm1397
    }
}
