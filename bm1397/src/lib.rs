#![no_std]
//! BM1397 ASIC implementation.

use core::time::Duration;
use fugit::HertzU32;

pub const BM1397_CORE_CNT: usize = 168;
pub const BM1397_SMALL_CORE_CNT: usize = 672;
pub const BM1397_CORE_SMALL_CORE_CNT: usize = 4;
pub const BM1397_DOMAIN_CNT: usize = 4;
pub const BM1397_PLL_CNT: usize = 4;
pub const BM1397_NONCE_CORES_BITS: usize = 8; // Core ID is hardcoded on Nonce[31:24] -> 8 bits
pub const BM1397_NONCE_SMALL_CORES_BITS: usize = 2; // Small Core ID is hardcoded on Nonce[23:22] -> 2 bits

const NONCE_BITS: usize = 32;
const CHIP_ADDR_BITS: usize = 8;

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
    pub chip_addr: u8,
    pub chip_interval: usize,
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

    /// ## Set the Chip Address
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    ///
    /// let mut bm1397 = BM1397::default();
    /// bm1397.set_chip_addr(2, 2);
    /// assert_eq!(bm1397.chip_addr, 2);
    /// assert_eq!(bm1397.chip_interval, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8, chip_interval: usize) {
        self.chip_addr = chip_addr;
        self.chip_interval = chip_interval;
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

    /// ## Get the rolling duration
    ///
    /// BM1397 only roll the Nonce Space (32 bits), but:
    /// - Nonce[31:24] is used to hardcode the Core ID.
    /// - Nonce[23:22] is used to hardcode the Small Core ID.
    /// - Nonce[21:14] is used to hardcode the Chip Address.
    /// So only the Nonce[13:0] are rolled for each Chip Address.
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use core::time::Duration;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.rolling_duration(), Duration::from_secs_f32(0.01048576));
    /// ```
    pub fn rolling_duration(&self) -> Duration {
        let space = (1
            << (NONCE_BITS
                - BM1397_NONCE_CORES_BITS
                - BM1397_NONCE_SMALL_CORES_BITS
                - CHIP_ADDR_BITS)) as f32;
        Duration::from_secs_f32(space * self.chip_interval as f32 / (self.hash_freq().raw() as f32))
    }
}

impl Default for BM1397 {
    fn default() -> Self {
        let mut bm1397 = Self {
            sha: bm13xx_asic::sha::Asic::default(),
            plls: [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT],
            chip_addr: 0,
            chip_interval: 256,
        };
        bm1397.plls[0].set_dividers(0x60, 1, 6, 1);
        bm1397.plls[0].enable();
        bm1397.plls[1].set_dividers(0x64, 1, 1, 1);
        bm1397.plls[2].set_dividers(0x68, 1, 1, 1);
        bm1397.plls[3].set_dividers(0x70, 1, 1, 1);
        bm1397
    }
}
