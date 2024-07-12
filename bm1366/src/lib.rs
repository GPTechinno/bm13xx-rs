#![no_std]
//! BM1366 ASIC implementation.

use bm13xx_asic::register::*;

use core::time::Duration;
use fugit::HertzU32;

pub const BM1366_CORE_CNT: usize = 112;
pub const BM1366_SMALL_CORE_CNT: usize = 894;
pub const BM1366_CORE_SMALL_CORE_CNT: usize = 8;
pub const BM1366_DOMAIN_CNT: usize = 1;
pub const BM1366_PLL_CNT: usize = 2;
pub const BM1366_NONCE_CORES_BITS: usize = 7; // Core ID is hardcoded on Nonce[31:25] -> 7 bits
pub const BM1366_NONCE_CORES_MASK: u32 = 0b111_1111;
pub const BM1366_NONCE_SMALL_CORES_BITS: usize = 3; // Small Core ID is hardcoded on Nonce[24:22] -> 3 bits
pub const BM1366_NONCE_SMALL_CORES_MASK: u32 = 0b111;

const NONCE_BITS: usize = 32;
const CHIP_ADDR_BITS: usize = 8;
const CHIP_ADDR_MASK: u32 = 0b1111_1111;

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
    pub registers: [Registers; 57],
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
    /// bm1366.set_chip_addr(2);
    /// assert_eq!(bm1366.chip_addr, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8) {
        self.chip_addr = chip_addr;
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
    /// assert_eq!(bm1366.rolling_duration(), Duration::from_secs_f32(0.000234057));
    /// bm1366.enable_version_rolling(0x1fffe000);
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
        Duration::from_secs_f32(space / (self.hash_freq().raw() as f32))
    }

    /// ## Get the Core ID that produced a given Nonce
    ///
    /// Core ID is always hardcoded in Nonce\[31:25\].
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let bm1366 = BM1366::default();
    /// assert_eq!(bm1366.nonce2core_id(0x12345678), 0x09);
    /// ```
    pub fn nonce2core_id(&self, nonce: u32) -> usize {
        ((nonce >> (NONCE_BITS - BM1366_NONCE_CORES_BITS)) & BM1366_NONCE_CORES_MASK) as usize
    }

    /// ## Get the Small Core ID that produced a given Nonce
    ///
    /// If the Hardware Version Rolling is disabled, the Small Core ID is hardcoded in Nonce\[24:22\].
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let bm1366 = BM1366::default();
    /// assert_eq!(bm1366.nonce2small_core_id(0x12045678), 0);
    /// assert_eq!(bm1366.nonce2small_core_id(0x12445678), 1);
    /// assert_eq!(bm1366.nonce2small_core_id(0x12845678), 2);
    /// assert_eq!(bm1366.nonce2small_core_id(0x12c45678), 3);
    /// assert_eq!(bm1366.nonce2small_core_id(0x13045678), 4);
    /// assert_eq!(bm1366.nonce2small_core_id(0x13445678), 5);
    /// assert_eq!(bm1366.nonce2small_core_id(0x13845678), 6);
    /// assert_eq!(bm1366.nonce2small_core_id(0x13c45678), 7);
    /// ```
    pub fn nonce2small_core_id(&self, nonce: u32) -> usize {
        ((nonce >> (NONCE_BITS - BM1366_NONCE_CORES_BITS - BM1366_NONCE_SMALL_CORES_BITS))
            & BM1366_NONCE_SMALL_CORES_MASK) as usize
    }

    /// ## Get the Small Core ID that produced a given Version
    ///
    /// If the Hardware Version Rolling is enabled, the Small Core ID is hardcoded in Version\[15:13\]
    /// (assuming the Version Mask is 0x1fffe000).
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let mut bm1366 = BM1366::default();
    /// bm1366.enable_version_rolling(0x1fffe000);
    /// assert_eq!(bm1366.version2small_core_id(0x1fff0000), 0);
    /// assert_eq!(bm1366.version2small_core_id(0x1fff2000), 1);
    /// assert_eq!(bm1366.version2small_core_id(0x1fff4000), 2);
    /// assert_eq!(bm1366.version2small_core_id(0x1fff6000), 3);
    /// assert_eq!(bm1366.version2small_core_id(0x1fff8000), 4);
    /// assert_eq!(bm1366.version2small_core_id(0x1fffa000), 5);
    /// assert_eq!(bm1366.version2small_core_id(0x1fffd000), 6);
    /// assert_eq!(bm1366.version2small_core_id(0x1fffe000), 7);
    /// ```
    pub fn version2small_core_id(&self, version: u32) -> usize {
        ((version >> self.version_mask.trailing_zeros()) & BM1366_NONCE_SMALL_CORES_MASK) as usize
    }

    /// ## Get the Chip Address that produced a given Nonce
    ///
    /// If the Hardware Version Rolling is enabled, the Chip Address is hardcoded in Nonce\[24:17\],
    /// else it is hardcoded in Nonce\[21:14\].
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.nonce2chip_addr(0x12345678), 0xD1);
    /// bm1366.enable_version_rolling(0x1fffe000);
    /// assert_eq!(bm1366.nonce2chip_addr(0x12345679), 0x1A);
    /// ```
    pub fn nonce2chip_addr(&self, nonce: u32) -> usize {
        if self.version_rolling_enabled {
            ((nonce >> (NONCE_BITS - BM1366_NONCE_CORES_BITS - CHIP_ADDR_BITS)) & CHIP_ADDR_MASK)
                as usize
        } else {
            ((nonce
                >> (NONCE_BITS
                    - BM1366_NONCE_CORES_BITS
                    - BM1366_NONCE_SMALL_CORES_BITS
                    - CHIP_ADDR_BITS))
                & CHIP_ADDR_MASK) as usize
        }
    }
}

impl Default for BM1366 {
    fn default() -> Self {
        let mut bm1366 = Self {
            sha: bm13xx_asic::sha::Asic::default(),
            plls: [bm13xx_asic::pll::Pll::default(); BM1366_PLL_CNT],
            chip_addr: 0,
            registers: [
                Registers::ChipIdentification(ChipIdentification(0x1366_0000)),
                Registers::HashRate(HashRate(0x0001_2a89)),
                Registers::PLL0Parameter(PLL0Parameter(0xc054_0165)),
                Registers::ChipNonceOffset(ChipNonceOffset(0x0000_0000)),
                Registers::HashCountingNumber(HashCountingNumber(0x0000_0000)),
                Registers::TicketMask(TicketMask(0x0000_0000)),
                Registers::MiscControl(MiscControl(0x0000_c100)),
                // Registers::I2CControl(I2CControl(0x0000_0000)),
                Registers::OrderedClockEnable(OrderedClockEnable(0x0000_0003)),
                Registers::Reg24(Reg24(0x0010_0000)),
                Registers::FastUARTConfiguration(FastUARTConfiguration(0x0130_1a00)),
                Registers::UARTRelay(UARTRelay(0x000f_0000)),
                Registers::Reg30(Reg30(0x0000_0070)),
                Registers::Reg34(Reg34(0x0000_0000)),
                Registers::TicketMask2(TicketMask2(0x0000_0000)),
                Registers::CoreRegisterControl(CoreRegisterControl(0x0000_0000)),
                Registers::CoreRegisterValue(CoreRegisterValue(0x1eaf_5fbe)),
                // Registers::ExternalTemperatureSensorRead(ExternalTemperatureSensorRead(
                //     0x0000_0000,
                // )),
                Registers::ErrorFlag(ErrorFlag(0x0000_0000)),
                Registers::NonceErrorCounter(NonceErrorCounter(0x0000_0000)),
                Registers::NonceOverflowCounter(NonceOverflowCounter(0x0000_0000)),
                Registers::AnalogMuxControl(AnalogMuxControl(0x0000_0000)),
                Registers::IoDriverStrenghtConfiguration(IoDriverStrenghtConfiguration(
                    0x0001_2111,
                )),
                Registers::TimeOut(TimeOut(0x0000_FFFF)),
                Registers::PLL1Parameter(PLL1Parameter(0x2050_0174)),
                // Registers::PLL2Parameter(PLL2Parameter(0x0000_0000)),
                // Registers::PLL3Parameter(PLL3Parameter(0x0000_0000)),
                Registers::OrderedClockMonitor(OrderedClockMonitor(0x0001_0200)),
                Registers::PLL0Divider(PLL0Divider(0x0000_0000)),
                Registers::PLL1Divider(PLL1Divider(0x0000_0000)),
                // Registers::PLL2Divider(PLL2Divider(0x0000_0000)),
                // Registers::PLL3Divider(PLL3Divider(0x0000_0000)),
                Registers::ClockOrderControl0(ClockOrderControl0(0x0000_0000)),
                Registers::ClockOrderControl1(ClockOrderControl1(0x0000_0000)),
                Registers::ClockOrderStatus(ClockOrderStatus(0x0000_0000)),
                Registers::FrequencySweepControl1(FrequencySweepControl1(0x0000_0000)),
                Registers::GoldenNonceForSweepReturn(GoldenNonceForSweepReturn(0x0000_0000)),
                Registers::ReturnedGroupPatternStatus(ReturnedGroupPatternStatus(0x0000_0000)),
                Registers::NonceReturnedTimeout(NonceReturnedTimeout(0x00fd_0077)),
                Registers::ReturnedSinglePatternStatus(ReturnedSinglePatternStatus(0x0000_0000)),
                Registers::VersionRolling(VersionRolling(0x0000_ffff)),
                Registers::RegA8(RegA8(0x0007_0000)),
                Registers::RegAC(RegAC(0x0000_0000)),
                Registers::RegB0(RegB0(0x0000_0000)),
                Registers::RegB4(RegB4(0x0000_0000)),
                Registers::RegB8(RegB8(0x0000_0000)),
                Registers::RegBC(RegBC(0x0000_3313)),
                Registers::RegC0(RegC0(0x0000_2000)),
                Registers::RegC4(RegC4(0x0000_0000)),
                Registers::RegC8(RegC8(0x0000_0000)),
                Registers::RegCC(RegCC(0x0000_0000)),
                Registers::RegD0(RegD0(0x0000_0070)),
                Registers::RegD4(RegD4(0x0037_6400)),
                Registers::RegD8(RegD8(0x3030_3030)),
                Registers::RegDC(RegDC(0x0000_ffff)),
                Registers::RegE0(RegE0(0x0000_0000)),
                Registers::RegE4(RegE4(0x0000_0000)),
                Registers::RegE8(RegE8(0x0000_0000)),
                Registers::RegEC(RegEC(0x0000_0008)),
                Registers::RegF0(RegF0(0x0000_0000)),
                Registers::RegF4(RegF4(0x0000_0000)),
                Registers::RegF8(RegF8(0x0000_0000)),
                Registers::RegFC(RegFC(0x0000_0000)),
            ],
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
