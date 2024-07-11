#![no_std]
//! BM1366 ASIC implementation.

use core::time::Duration;
use fugit::HertzU32;

pub const BM1366_CORE_CNT: usize = 112;
pub const BM1366_SMALL_CORE_CNT: usize = 894;
pub const BM1366_CORE_SMALL_CORE_CNT: usize = 8;
pub const BM1366_DOMAIN_CNT: usize = 1;
pub const BM1366_PLL_CNT: usize = 2;
pub const BM1366_NONCE_CORES_BITS: usize = 7; // Core ID is hardcoded on Nonce[31:25] -> 7 bits
pub const BM1366_NONCE_SMALL_CORES_BITS: usize = 3; // Small Core ID is hardcoded on Nonce[24:22] -> 3 bits

const NONCE_BITS: usize = 32;
const CHIP_ADDR_BITS: usize = 8;

/// # BM1366
#[derive(Debug)]
pub struct BM1366 {
    pub sha: bm13xx_asic::sha::Asic<
        BM1366_CORE_CNT,
        BM1366_SMALL_CORE_CNT,
        BM1366_CORE_SMALL_CORE_CNT,
        BM1366_DOMAIN_CNT,
    >,
    pub plls: [bm13xx_asic::pll::Pll; BM1366_PLL_CNT],
    pub chip_addr: u8,
    pub chip_interval: usize,
    pub version_rolling_enabled: bool,
    pub version_mask: u32,
}

impl BM1366 {
    pub fn new_with_clk(clk: HertzU32) -> Self {
        let mut bm1366 = Self::default();
        bm1366
            .plls
            .iter_mut()
            .for_each(|pll| pll.set_input_clk_freq(clk));
        bm1366
    }

    /// ## Set the Chip Address
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let mut bm1366 = BM1366::default();
    /// bm1366.set_chip_addr(2, 2);
    /// assert_eq!(bm1366.chip_addr, 2);
    /// assert_eq!(bm1366.chip_interval, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8, chip_interval: usize) {
        self.chip_addr = chip_addr;
        self.chip_interval = chip_interval;
    }

    /// ## Enable the Hardware Version Rolling
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let mut bm1366 = BM1366::default();
    /// bm1366.enable_version_rolling(0x1fffe000);
    /// assert!(bm1366.version_rolling_enabled);
    /// assert_eq!(bm1366.version_mask, 0x1fffe000);
    /// ```
    pub fn enable_version_rolling(&mut self, version_mask: u32) {
        self.version_rolling_enabled = true;
        self.version_mask = version_mask;
    }

    fn version_mask_bits(&self) -> usize {
        self.version_mask.count_ones() as usize
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use fugit::HertzU32;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.hash_freq(), HertzU32::MHz(70u32));
    // bm1366.plls[0].set_parameter(0x40A0_0241); // from Bitaxe default freq
    // assert_eq!(bm1366.hash_freq(), HertzU32::MHz(200u32));
    /// ```
    pub fn hash_freq(&self) -> HertzU32 {
        self.plls[0].frequency(0)
    }

    /// ## Get the theoretical Hashrate in GH/s
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use fugit::HertzU32;
    ///
    /// let bm1366 = BM1366::default();
    /// assert_eq!(bm1366.theoretical_hashrate_ghs(), 62.579998);
    /// ```
    pub fn theoretical_hashrate_ghs(&self) -> f32 {
        self.hash_freq().raw() as f32 * self.sha.small_core_count() as f32 / 1_000_000_000.0
    }

    /// ## Get the rolling duration
    ///
    /// BM1366 can do Version Rolling in Hardware.
    ///
    /// If Hardware Version Rolling is not enabled, BM1366 only roll the Nonce Space (32 bits), but:
    /// - Nonce\[31:25\] is used to hardcode the Core ID.
    /// - Nonce\[24:22\] is used to hardcode the Small Core ID.
    /// - Nonce\[21:14\] is used to hardcode the Chip Address.
    /// So only the Nonce\[13:0\] are rolled for each Chip Address.
    ///
    /// If Hardware Version Rolling is enabled, BM1366 roll the Nonce Space (32 bits) and
    /// up to 16 bits in Version Space, but:
    /// - Nonce\[31:25\] is used to hardcode the Core ID.
    /// - Nonce\[24:17\] is used to hardcode the Chip Address.
    /// - Version\[15:13\] is used to hardcode the Small Core ID (assuming the Version Mask is 0x1fffe000).
    /// So only the Nonce\[16:0\] and Version\[28:16\] are rolled for each Chip Address.
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use core::time::Duration;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.rolling_duration(), Duration::from_secs_f32(0.059918629));
    /// bm1366.enable_version_rolling(0x1fffe000);
    /// bm1366.set_chip_addr(0, 1); // example of S19k-Pro HB56601 with 204 ASIC in the chain
    /// assert_eq!(bm1366.rolling_duration(), Duration::from_secs_f32(15.339168549));
    /// ```
    pub fn rolling_duration(&self) -> Duration {
        let space = if self.version_rolling_enabled {
            (1 << (NONCE_BITS - BM1366_NONCE_CORES_BITS - CHIP_ADDR_BITS
                + self.version_mask_bits()
                - BM1366_NONCE_SMALL_CORES_BITS)) as f32
        } else {
            (1 << (NONCE_BITS
                - BM1366_NONCE_CORES_BITS
                - BM1366_NONCE_SMALL_CORES_BITS
                - CHIP_ADDR_BITS)) as f32
        };
        Duration::from_secs_f32(space * self.chip_interval as f32 / (self.hash_freq().raw() as f32))
    }
}

impl Default for BM1366 {
    fn default() -> Self {
        let mut bm1366 = Self {
            sha: bm13xx_asic::sha::Asic::default(),
            plls: [bm13xx_asic::pll::Pll::default(); BM1366_PLL_CNT],
            chip_addr: 0,
            chip_interval: 256,
            version_rolling_enabled: false,
            version_mask: 0x1fffe000,
        };
        bm1366.plls[0].set_parameter(0xC054_0165);
        bm1366.plls[1].set_parameter(0x2050_0174); // TODO: understand what is the 2 in MSB

        // bm1366.plls[0].set_divider(0x0000_0000); // already default value
        // bm1366.plls[1].set_divider(0x0000_0000);
        bm1366
    }
}
