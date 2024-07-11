#![no_std]
//! BM1397 ASIC implementation.

use bm13xx_asic::register::*;

use core::time::Duration;
use fugit::HertzU32;

pub const BM1397_CORE_CNT: usize = 168;
pub const BM1397_SMALL_CORE_CNT: usize = 672;
pub const BM1397_CORE_SMALL_CORE_CNT: usize = 4;
pub const BM1397_DOMAIN_CNT: usize = 4;
pub const BM1397_PLL_CNT: usize = 4;
pub const BM1397_NONCE_CORES_BITS: usize = 8; // Core ID is hardcoded on Nonce[31:24] -> 8 bits
pub const BM1397_NONCE_CORES_MASK: u32 = 0b1111_1111;
pub const BM1397_NONCE_SMALL_CORES_BITS: usize = 2; // Small Core ID is hardcoded on Nonce[23:22] -> 2 bits
pub const BM1397_NONCE_SMALL_CORES_MASK: u32 = 0b11;

const NONCE_BITS: usize = 32;
const CHIP_ADDR_BITS: usize = 8;
const CHIP_ADDR_MASK: u32 = 0b1111_1111;

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
    pub registers: [Registers; 37],
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
    /// bm1397.set_chip_addr(2);
    /// assert_eq!(bm1397.chip_addr, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8) {
        self.chip_addr = chip_addr;
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use fugit::HertzU32;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.hash_freq(), HertzU32::MHz(50u32));
    /// ```
    pub fn hash_freq(&self) -> HertzU32 {
        self.plls[0].frequency(0)
    }

    /// ## Get the theoretical Hashrate in GH/s
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use fugit::HertzU32;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.theoretical_hashrate_ghs(), 33.6);
    /// ```
    pub fn theoretical_hashrate_ghs(&self) -> f32 {
        self.hash_freq().raw() as f32 * self.sha.small_core_count() as f32 / 1_000_000_000.0
    }

    /// ## Get the rolling duration
    ///
    /// BM1397 only roll the Nonce Space (32 bits), but:
    /// - Nonce\[31:24\] is used to hardcode the Core ID.
    /// - Nonce\[23:22\] is used to hardcode the Small Core ID.
    /// - Nonce\[21:14\] is used to hardcode the Chip Address.
    /// So only the Nonce\[13:0\] are rolled for each Chip Address.
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use core::time::Duration;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.rolling_duration(), Duration::from_secs_f32(0.00032768));
    /// ```
    pub fn rolling_duration(&self) -> Duration {
        let space = (1
            << (NONCE_BITS
                - BM1397_NONCE_CORES_BITS
                - BM1397_NONCE_SMALL_CORES_BITS
                - CHIP_ADDR_BITS)) as f32;
        Duration::from_secs_f32(space / (self.hash_freq().raw() as f32))
    }

    /// ## Get the Core ID that produced a given Nonce
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.nonce2core_id(0x12345678), 0x12);
    /// ```
    pub fn nonce2core_id(&self, nonce: u32) -> usize {
        ((nonce >> (NONCE_BITS - BM1397_NONCE_CORES_BITS)) & BM1397_NONCE_CORES_MASK) as usize
    }

    /// ## Get the Small Core ID that produced a given Nonce
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.nonce2small_core_id(0x12045678), 0);
    /// assert_eq!(bm1397.nonce2small_core_id(0x12445678), 1);
    /// assert_eq!(bm1397.nonce2small_core_id(0x12845678), 2);
    /// assert_eq!(bm1397.nonce2small_core_id(0x12c45678), 3);
    /// ```
    pub fn nonce2small_core_id(&self, nonce: u32) -> usize {
        ((nonce >> (NONCE_BITS - BM1397_NONCE_CORES_BITS - BM1397_NONCE_SMALL_CORES_BITS))
            & BM1397_NONCE_SMALL_CORES_MASK) as usize
    }

    /// ## Get the Chip Address that produced a given Nonce
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.nonce2chip_addr(0x12345678), 0xD1);
    /// ```
    pub fn nonce2chip_addr(&self, nonce: u32) -> usize {
        ((nonce
            >> (NONCE_BITS
                - BM1397_NONCE_CORES_BITS
                - BM1397_NONCE_SMALL_CORES_BITS
                - CHIP_ADDR_BITS))
            & CHIP_ADDR_MASK) as usize
    }
}

impl Default for BM1397 {
    fn default() -> Self {
        let mut bm1397 = Self {
            sha: bm13xx_asic::sha::Asic::default(),
            plls: [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT],
            chip_addr: 0,
            registers: [
                Registers::ChipIdentification(ChipIdentification(0x1397_1800)),
                Registers::HashRate(HashRate(0x8000_0000)),
                Registers::PLL0Parameter(PLL0Parameter(0xC060_0161)),
                Registers::ChipNonceOffset(ChipNonceOffset(0x0000_0000)),
                Registers::HashCountingNumber(HashCountingNumber(0x0000_0000)),
                Registers::TicketMask(TicketMask(0x0000_0000)),
                Registers::MiscControl(MiscControl(0x0000_3A01)),
                Registers::I2CControl(I2CControl(0x0100_0000)),
                Registers::OrderedClockEnable(OrderedClockEnable(0x0000_FFFF)),
                Registers::FastUARTConfiguration(FastUARTConfiguration(0x0600_000F)),
                Registers::UARTRelay(UARTRelay(0x000F_0000)),
                Registers::TicketMask2(TicketMask2(0x0000_0000)),
                Registers::CoreRegisterControl(CoreRegisterControl(0x0000_4000)),
                Registers::CoreRegisterValue(CoreRegisterValue(0x0000_0000)),
                Registers::ExternalTemperatureSensorRead(ExternalTemperatureSensorRead(
                    0x0000_0100,
                )),
                Registers::ErrorFlag(ErrorFlag(0xFF00_0000)),
                Registers::NonceErrorCounter(NonceErrorCounter(0x0000_0000)),
                Registers::NonceOverflowCounter(NonceOverflowCounter(0x0000_0000)),
                Registers::AnalogMuxControl(AnalogMuxControl(0x0000_0000)),
                Registers::IoDriverStrenghtConfiguration(IoDriverStrenghtConfiguration(
                    0x0211_2111,
                )),
                Registers::TimeOut(TimeOut(0x0000_FFFF)),
                Registers::PLL1Parameter(PLL1Parameter(0x0064_0111)),
                Registers::PLL2Parameter(PLL2Parameter(0x0068_0111)),
                Registers::PLL3Parameter(PLL3Parameter(0x0070_0111)),
                Registers::OrderedClockMonitor(OrderedClockMonitor(0x0000_0000)),
                Registers::PLL0Divider(PLL0Divider(0x0304_0607)),
                Registers::PLL1Divider(PLL1Divider(0x0304_0506)),
                Registers::PLL2Divider(PLL2Divider(0x0304_0506)),
                Registers::PLL3Divider(PLL3Divider(0x0304_0506)),
                Registers::ClockOrderControl0(ClockOrderControl0(0xD95C_8410)),
                Registers::ClockOrderControl1(ClockOrderControl1(0xFB73_EA62)),
                Registers::ClockOrderStatus(ClockOrderStatus(0x0000_0000)),
                Registers::FrequencySweepControl1(FrequencySweepControl1(0x0000_0070)),
                Registers::GoldenNonceForSweepReturn(GoldenNonceForSweepReturn(0x0037_6400)),
                Registers::ReturnedGroupPatternStatus(ReturnedGroupPatternStatus(0x3030_3030)),
                Registers::NonceReturnedTimeout(NonceReturnedTimeout(0x0000_FFFF)),
                Registers::ReturnedSinglePatternStatus(ReturnedSinglePatternStatus(0x0000_0000)),
            ],
        };
        bm1397.plls[0].set_parameter(0xC060_0161);
        bm1397.plls[1].set_parameter(0x0064_0111);
        bm1397.plls[2].set_parameter(0x0068_0111);
        bm1397.plls[3].set_parameter(0x0070_0111);
        bm1397.plls[0].set_divider(0x0304_0607);
        bm1397.plls[1].set_divider(0x0304_0506);
        bm1397.plls[2].set_divider(0x0304_0506);
        bm1397.plls[3].set_divider(0x0304_0506);
        bm1397
    }
}
