//! BM1366 ASIC implementation.

#![no_std]
#![macro_use]
pub(crate) mod fmt;

use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
use bm13xx_protocol::command::{Command, Destination};

use core::time::Duration;
use fugit::HertzU64;
use heapless::{FnvIndexMap, Vec};

pub const BM1366_CHIP_ID: u16 = 0x1366;
pub const BM1366_CORE_CNT: usize = 112;
pub const BM1366_SMALL_CORE_CNT: usize = 894;
pub const BM1366_CORE_SMALL_CORE_CNT: usize = 8;
pub const BM1366_DOMAIN_CNT: usize = 1;
pub const BM1366_PLL_CNT: usize = 2;
pub const BM1366_PLL_ID_HASH: usize = 0; // PLL0 isused for Hashing
pub const BM1366_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 can be used for Hashing
pub const BM1366_PLL_ID_UART: usize = 1; // PLL1 can be used for UART Baudrate
pub const BM1366_PLL_OUT_UART: usize = 4; // specifically PLL1_OUT4 can be used for UART Baudrate
pub const BM1366_NONCE_CORES_BITS: usize = 7; // Core ID is hardcoded on Nonce[31:25] -> 7 bits
pub const BM1366_NONCE_CORES_MASK: u32 = 0b111_1111;
pub const BM1366_NONCE_SMALL_CORES_BITS: usize = 3; // Small Core ID is hardcoded on Nonce[24:22] -> 3 bits
pub const BM1366_NONCE_SMALL_CORES_MASK: u32 = 0b111;

const NONCE_BITS: usize = 32;
const CHIP_ADDR_BITS: usize = 8;
const CHIP_ADDR_MASK: u32 = 0b1111_1111;

/// # BM1366
#[derive(Debug)]
// #[cfg_attr(feature = "defmt-03", derive(defmt::Format))] // FnvIndexMap doesn't implement defmt
pub struct BM1366 {
    pub sha: bm13xx_asic::sha::Sha<
        BM1366_CORE_CNT,
        BM1366_SMALL_CORE_CNT,
        BM1366_CORE_SMALL_CORE_CNT,
        BM1366_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU64,
    pub plls: [bm13xx_asic::pll::Pll; BM1366_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub core_registers: FnvIndexMap<u8, u8, 16>,
    pub version_rolling_enabled: bool,
    pub version_mask: u32,
}

impl BM1366 {
    pub fn new_with_clk(clk: HertzU64) -> Self {
        BM1366 {
            input_clock_freq: clk,
            ..Default::default()
        }
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
    /// use bm1366::{BM1366, BM1366_PLL_ID_HASH};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.hash_freq(), HertzU64::MHz(50));
    /// assert_eq!(bm1366.set_hash_freq(HertzU64::MHz(200)).hash_freq(), HertzU64::MHz(200));
    /// ```
    pub fn hash_freq(&self) -> HertzU64 {
        self.plls[BM1366_PLL_ID_HASH].frequency(self.input_clock_freq, BM1366_PLL_OUT_HASH)
    }
    pub fn set_hash_freq(&mut self, freq: HertzU64) -> &mut Self {
        self.plls[BM1366_PLL_ID_HASH].set_frequency(
            self.input_clock_freq,
            BM1366_PLL_OUT_HASH,
            freq,
        );
        self
    }

    /// ## Get the theoretical Hashrate in GH/s
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use fugit::HertzU64;
    ///
    /// let bm1366 = BM1366::default();
    /// assert_eq!(bm1366.theoretical_hashrate_ghs(), 44.7);
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
    ///
    /// So only the Nonce\[13:0\] are rolled for each Chip Address.
    ///
    /// If Hardware Version Rolling is enabled, BM1366 roll the Nonce Space (32 bits) and
    /// up to 16 bits in Version Space, but:
    /// - Nonce\[31:25\] is used to hardcode the Core ID.
    /// - Nonce\[24:17\] is used to hardcode the Chip Address.
    /// - Version\[15:13\] is used to hardcode the Small Core ID (assuming the Version Mask is 0x1fffe000).
    ///
    /// So only the Nonce\[16:0\] and Version\[28:16\] are rolled for each Chip Address.
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use core::time::Duration;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.rolling_duration(), Duration::from_secs_f32(0.00032768));
    /// bm1366.enable_version_rolling(0x1fffe000);
    /// assert_eq!(bm1366.rolling_duration(), Duration::from_secs_f32(21.474836349));
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
    /// assert_eq!(bm1366.nonce2core_id(0x906732c8), 72); // first Bitaxe Block 853742
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
    /// assert_eq!(bm1366.version2small_core_id(0x00f94000), 2); // first Bitaxe Block 853742
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
            sha: bm13xx_asic::sha::Sha::default(),
            input_clock_freq: HertzU64::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1366_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            core_registers: FnvIndexMap::<_, _, 16>::new(),
            version_rolling_enabled: false,
            version_mask: 0x1fffe000,
        };
        // Default PLLs Parameter
        bm1366.plls[0].set_parameter(0xC054_0165);
        bm1366.plls[1].set_parameter(0x2050_0174);
        // Default PLLs Divider
        bm1366.plls[0].set_divider(0x0000_0000);
        bm1366.plls[1].set_divider(0x0000_0000);
        // Default Registers Value
        bm1366
            .registers
            .insert(ChipIdentification::ADDR, 0x1366_0000)
            .unwrap();
        bm1366
            .registers
            .insert(HashRate::ADDR, 0x0001_2a89)
            .unwrap();
        bm1366
            .registers
            .insert(PLL0Parameter::ADDR, 0xc054_0165)
            .unwrap();
        bm1366
            .registers
            .insert(ChipNonceOffsetV2::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(HashCountingNumber::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(TicketMask::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(MiscControl::ADDR, 0x0000_c100)
            .unwrap();
        bm1366
            .registers
            .insert(I2CControl::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(OrderedClockEnable::ADDR, 0x0000_0003)
            .unwrap();
        bm1366.registers.insert(Reg24::ADDR, 0x0010_0000).unwrap();
        bm1366
            .registers
            .insert(FastUARTConfigurationV2::ADDR, 0x0130_1a00)
            .unwrap();
        bm1366
            .registers
            .insert(UARTRelay::ADDR, 0x000f_0000)
            .unwrap();
        bm1366.registers.insert(Reg30::ADDR, 0x0000_0070).unwrap();
        bm1366.registers.insert(Reg34::ADDR, 0x0000_0000).unwrap();
        bm1366
            .registers
            .insert(TicketMask2::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(CoreRegisterControl::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(CoreRegisterValue::ADDR, 0x1eaf_5fbe)
            .unwrap();
        bm1366
            .registers
            .insert(ExternalTemperatureSensorRead::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(ErrorFlag::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(NonceErrorCounter::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(NonceOverflowCounter::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(AnalogMuxControlV2::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(IoDriverStrenghtConfiguration::ADDR, 0x0001_2111)
            .unwrap();
        bm1366.registers.insert(TimeOut::ADDR, 0x0000_FFFF).unwrap();
        bm1366
            .registers
            .insert(PLL1Parameter::ADDR, 0x2050_0174)
            .unwrap();
        bm1366
            .registers
            .insert(OrderedClockMonitor::ADDR, 0x0001_0200)
            .unwrap();
        bm1366
            .registers
            .insert(PLL0Divider::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(PLL1Divider::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(ClockOrderControl0::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(ClockOrderControl1::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(ClockOrderStatus::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(FrequencySweepControl1::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(GoldenNonceForSweepReturn::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(ReturnedGroupPatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(NonceReturnedTimeout::ADDR, 0x00fd_0077)
            .unwrap();
        bm1366
            .registers
            .insert(ReturnedSinglePatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        bm1366
            .registers
            .insert(VersionRolling::ADDR, 0x0000_ffff)
            .unwrap();
        bm1366.registers.insert(RegA8::ADDR, 0x0007_0000).unwrap();
        bm1366.registers.insert(RegAC::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegB0::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegB4::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegB8::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegBC::ADDR, 0x0000_3313).unwrap();
        bm1366.registers.insert(RegC0::ADDR, 0x0000_2000).unwrap();
        bm1366.registers.insert(RegC4::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegC8::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegCC::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegD0::ADDR, 0x0000_0070).unwrap();
        bm1366.registers.insert(RegD4::ADDR, 0x0037_6400).unwrap();
        bm1366.registers.insert(RegD8::ADDR, 0x3030_3030).unwrap();
        bm1366.registers.insert(RegDC::ADDR, 0x0000_ffff).unwrap();
        bm1366.registers.insert(RegE0::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegE4::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegE8::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegEC::ADDR, 0x0000_0008).unwrap();
        bm1366.registers.insert(RegF0::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegF4::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegF8::ADDR, 0x0000_0000).unwrap();
        bm1366.registers.insert(RegFC::ADDR, 0x0000_0000).unwrap();
        // Default Core Registers Value
        bm1366
            .core_registers
            .insert(ClockDelayCtrlV2::ID, 0x98)
            .unwrap();
        // bm1366.core_registers.insert(1, 0x00).unwrap(); // not used anywhere in official FW
        bm1366.core_registers.insert(2, 0x55).unwrap();
        bm1366.core_registers.insert(3, 0x00).unwrap();
        bm1366.core_registers.insert(4, 0x00).unwrap();
        bm1366
            .core_registers
            .insert(HashClockCtrl::ID, 0x40)
            .unwrap();
        bm1366
            .core_registers
            .insert(HashClockCounter::ID, 0x08)
            .unwrap();
        bm1366.core_registers.insert(7, 0x11).unwrap();
        bm1366.core_registers.insert(CoreReg8::ID, 0x00).unwrap();
        bm1366.core_registers.insert(15, 0x00).unwrap();
        bm1366.core_registers.insert(16, 0x00).unwrap();
        bm1366.core_registers.insert(CoreReg22::ID, 0x00).unwrap();
        bm1366
    }
}

impl Asic for BM1366 {
    /// ## Get the Chip ID
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1366 = BM1366::default();
    /// assert_eq!(bm1366.chip_id(), 0x1366);
    /// ```
    fn chip_id(&self) -> u16 {
        BM1366_CHIP_ID
    }

    /// ## Has Version Rolling in chip
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1366 = BM1366::default();
    /// assert!(bm1366.has_version_rolling());
    /// ```
    fn has_version_rolling(&self) -> bool {
        true
    }

    /// ## Init the Chip command list
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::{core_register::*, register::*, Asic};
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut init_seq = bm1366.send_init(256, 1, 10, 2);
    /// assert_eq!(init_seq.len(), 8);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x12, 0x2c, 0x00, 0x18, 0x00, 0x03, 0x0c]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x2c, 0x00, 0x18, 0x00, 0x03, 0x10]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x12, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x1b]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x02, 0x11, 0x11, 0x11, 0x06]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x54, 0x00, 0x00, 0x00, 0x03, 0x1d]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xff, 0x08]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x20, 0x19]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x0c]);
    /// assert_eq!(bm1366.core_registers.get(&HashClockCtrl::ID).unwrap(), &0x40);
    /// assert_eq!(bm1366.core_registers.get(&ClockDelayCtrlV2::ID).unwrap(), &0x20);
    /// assert_eq!(bm1366.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00ff);
    /// assert_eq!(bm1366.registers.get(&AnalogMuxControlV2::ADDR).unwrap(), &0x0000_0003);
    /// assert_eq!(bm1366.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(), &0x0211_1111);
    /// ```
    ///
    fn send_init(
        &mut self,
        initial_diffculty: u32,
        chain_domain_cnt: u8,
        domain_asic_cnt: u8,
        asic_addr_interval: u16,
    ) -> Vec<CmdDelay, 2048> {
        let mut init_seq = Vec::new();
        let hash_clk_ctrl = HashClockCtrl(*self.core_registers.get(&HashClockCtrl::ID).unwrap())
            .enable()
            .set_pll_source(0)
            .val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, HashClockCtrl(hash_clk_ctrl)), // CLOCK_CTRL=64
                    Destination::All,
                ),
                delay_ms: 10,
            })
            .unwrap();
        self.core_registers
            .insert(HashClockCtrl::ID, hash_clk_ctrl)
            .unwrap();
        let clk_dly_ctrl =
            ClockDelayCtrlV2(*self.core_registers.get(&ClockDelayCtrlV2::ID).unwrap())
                .set_ccdly(0)
                .set_pwth(4)
                .disable_sweep_frequency_mode()
                .val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, ClockDelayCtrlV2(clk_dly_ctrl)),
                    Destination::All,
                ),
                delay_ms: 10,
            })
            .unwrap();
        self.core_registers
            .insert(ClockDelayCtrlV2::ID, clk_dly_ctrl)
            .unwrap();
        let tck_mask = TicketMask::from_difficulty(initial_diffculty).val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(TicketMask::ADDR, tck_mask, Destination::All),
                delay_ms: 10,
            })
            .unwrap();
        self.registers.insert(TicketMask::ADDR, tck_mask).unwrap();
        let ana_mux_ctrl =
            AnalogMuxControlV2(*self.registers.get(&AnalogMuxControlV2::ADDR).unwrap())
                .set_diode_vdd_mux_sel(3)
                .val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(AnalogMuxControlV2::ADDR, ana_mux_ctrl, Destination::All),
                delay_ms: 0,
            })
            .unwrap();
        self.registers
            .insert(AnalogMuxControlV2::ADDR, ana_mux_ctrl)
            .unwrap();
        let io_drv_st_cfg = IoDriverStrenghtConfiguration(
            *self
                .registers
                .get(&IoDriverStrenghtConfiguration::ADDR)
                .unwrap(),
        )
        .set_strenght(DriverSelect::RF, 2)
        .set_strenght(DriverSelect::R0, 1)
        .set_strenght(DriverSelect::CLKO, 1)
        .set_strenght(DriverSelect::NRSTO, 1)
        .set_strenght(DriverSelect::BO, 1)
        .set_strenght(DriverSelect::CO, 1)
        .enable(DriverRSelect::D0R)
        .disable(DriverRSelect::D1R)
        .disable(DriverRSelect::D2R)
        .disable(DriverRSelect::D3R)
        .val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    IoDriverStrenghtConfiguration::ADDR,
                    io_drv_st_cfg,
                    Destination::All,
                ),
                delay_ms: 0,
            })
            .unwrap();
        self.registers
            .insert(IoDriverStrenghtConfiguration::ADDR, io_drv_st_cfg)
            .unwrap();
        // last chip of each voltage domain should have IoDriverStrenghtConfiguration set to 0x0211_f111
        // (iterating voltage domain in decreasing chip address order)
        for dom in (0..chain_domain_cnt).rev() {
            init_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        IoDriverStrenghtConfiguration::ADDR,
                        0x0211_f111,
                        Destination::Chip((dom + domain_asic_cnt - 1) * asic_addr_interval as u8),
                    ),
                    delay_ms: 0,
                })
                .unwrap();
        }
        // first and last chip of each voltage domain should have UARTRelay with GAP_CNT=domain_asic_num*(chain_domain_num-domain_i)+14 and RO_REL_EN=CO_REL_EN=1
        // (iterating voltage domain in decreasing chip address order)
        for dom in (0..chain_domain_cnt).rev() {
            let uart_delay = UARTRelay(*self.registers.get(&UARTRelay::ADDR).unwrap())
                .set_gap_cnt(
                    (domain_asic_cnt as u16) * ((chain_domain_cnt as u16) - (dom as u16)) + 14,
                )
                .enable_ro_relay()
                .enable_co_relay()
                .val();
            init_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        UARTRelay::ADDR,
                        uart_delay,
                        Destination::Chip(dom * asic_addr_interval as u8),
                    ),
                    delay_ms: 0,
                })
                .unwrap();
            if domain_asic_cnt > 1 {
                init_seq
                    .push(CmdDelay {
                        cmd: Command::write_reg(
                            UARTRelay::ADDR,
                            uart_delay,
                            Destination::Chip(
                                (dom + domain_asic_cnt - 1) * asic_addr_interval as u8,
                            ),
                        ),
                        delay_ms: 0,
                    })
                    .unwrap();
            }
        }
        init_seq
    }

    /// ## Send Baudrate command list
    ///
    /// ### Example
    /// ```
    /// use bm1366::{BM1366, BM1366_PLL_ID_UART};
    /// use bm13xx_asic::{register::*, Asic};
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut baud_seq = bm1366.send_baudrate(6_250_000);
    /// assert_eq!(baud_seq.len(), 2);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x05, 0x60, 0x07, 0x00, 23]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x60, 0xc0, 0x70, 0x01, 0x11, 26]);
    /// assert!(bm1366.plls[BM1366_PLL_ID_UART].enabled());
    /// assert_eq!(bm1366.registers.get(&PLL1Parameter::ADDR).unwrap(), &0xC070_0111);
    /// assert_eq!(bm1366.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x0560_0700);
    /// let mut baud_seq = bm1366.send_baudrate(1_000_000);
    /// assert_eq!(baud_seq.len(), 2);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x60, 0x00, 0x70, 0x01, 0x11, 15]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x01, 0x60, 0x02, 0x00, 21]);
    /// assert!(!bm1366.plls[BM1366_PLL_ID_UART].enabled());
    /// assert_eq!(bm1366.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x0160_0200);
    /// assert_eq!(bm1366.registers.get(&PLL1Parameter::ADDR).unwrap(), &0x0070_0111);
    /// ```
    fn send_baudrate(&mut self, baudrate: u32) -> Vec<CmdDelay, 3> {
        let mut baud_seq = Vec::new();
        if baudrate <= self.input_clock_freq.raw() as u32 / 8 {
            let fbase = self.input_clock_freq.raw() as u32;
            let bt8d = (fbase / (8 * baudrate)) - 1;
            let fast_uart_cfg = FastUARTConfigurationV2(
                *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
            )
            .set_bclk_sel(BaudrateClockSelectV2::Clki)
            .set_bt8d(bt8d as u8)
            .val();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        FastUARTConfigurationV2::ADDR,
                        fast_uart_cfg,
                        Destination::All,
                    ),
                    delay_ms: 0,
                })
                .unwrap();
            self.registers
                .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                .unwrap();
            let pll1_param = self.plls[BM1366_PLL_ID_UART].disable().unlock().parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL1Parameter::ADDR, pll1_param, Destination::All),
                    delay_ms: 0,
                })
                .unwrap();
            self.registers
                .insert(PLL1Parameter::ADDR, pll1_param)
                .unwrap();
        } else {
            let pll1_div4 = 6;
            self.plls[BM1366_PLL_ID_UART]
                .lock()
                .enable()
                .set_fb_div(112)
                .set_ref_div(1)
                .set_post1_div(1)
                .set_post2_div(1)
                .set_out_div(BM1366_PLL_OUT_UART, pll1_div4);
            // self.plls[BM1366_PLL_ID_UART]
            //     .set_parameter(0xC070_0111)
            //     .set_out_div(BM1366_PLL_OUT_UART, pll1_div4);
            let fbase = self.plls[BM1366_PLL_ID_UART]
                .frequency(self.input_clock_freq, BM1366_PLL_OUT_UART)
                .raw();
            let pll1_param = self.plls[BM1366_PLL_ID_UART].parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL1Parameter::ADDR, pll1_param, Destination::All),
                    delay_ms: 0,
                })
                .unwrap();
            self.registers
                .insert(PLL1Parameter::ADDR, pll1_param)
                .unwrap();
            let bt8d = (fbase as u32 / (2 * baudrate)) - 1;
            let fast_uart_cfg = FastUARTConfigurationV2(
                *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
            )
            .set_pll1_div4(pll1_div4)
            .set_bclk_sel(BaudrateClockSelectV2::Pll1)
            .set_bt8d(bt8d as u8)
            .val();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        FastUARTConfigurationV2::ADDR,
                        fast_uart_cfg,
                        Destination::All,
                    ),
                    delay_ms: 0,
                })
                .unwrap();
            self.registers
                .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                .unwrap();
        }
        baud_seq
    }

    /// ## Reset the Chip Cores command list
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::{core_register::*, register::*, Asic};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut reset_seq = bm1366.send_reset_core(Destination::All);
    /// assert_eq!(reset_seq.len(), 6);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x20, 0x19]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x0c]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0xff, 0x0f, 0xc1, 0x00, 0]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa8, 0x00, 0x07, 0x01, 0x00, 31]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x00, 0xc1, 0x00, 29]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa8, 0x00, 0x07, 0x00, 0x0f, 21]);
    /// let mut bm1366 = BM1366::default();
    /// let mut reset_seq = bm1366.send_reset_core(Destination::Chip(0));
    /// assert_eq!(reset_seq.len(), 5);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x82, 0xaa, 0x05]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x20, 0x11]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x04]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x18, 0xf0, 0x00, 0xc1, 0x00, 0x0c]);
    /// assert_eq!(reset_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0xa8, 0x00, 0x07, 0x01, 0xf0, 0x15]);
    /// ```
    fn send_reset_core(&mut self, dest: Destination) -> Vec<CmdDelay, 800> {
        let mut reset_seq = Vec::new();
        if dest == Destination::All {
            let reg_a8 = RegA8(*self.registers.get(&RegA8::ADDR).unwrap())
                .set_b3_0(0xf)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(RegA8::ADDR, reg_a8, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(RegA8::ADDR, reg_a8).unwrap();
            let misc = MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
                .set_core_return_nonce(0)
                .set_b27_26(0)
                .set_b25_24(0)
                .set_b19_16(0)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
            let reg_a8 = RegA8(*self.registers.get(&RegA8::ADDR).unwrap())
                .set_b8()
                .set_b3_0(0)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(RegA8::ADDR, reg_a8, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(RegA8::ADDR, reg_a8).unwrap();
            let misc = MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
                .set_core_return_nonce(0xf)
                .set_b27_26(0x3)
                .set_b25_24(0x3)
                .set_b19_16(0xf)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
        } else {
            let reg_a8 = RegA8(*self.registers.get(&RegA8::ADDR).unwrap())
                .set_b8()
                .set_b7_4(0xf)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(RegA8::ADDR, reg_a8, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(RegA8::ADDR, reg_a8).unwrap();
            let misc = MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
                .set_core_return_nonce(0xf)
                .set_b27_26(0)
                .set_b25_24(0)
                .set_b19_16(0)
                .val();
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                    delay_ms: 10,
                })
                .unwrap();
            self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
        }
        let hash_clk_ctrl = HashClockCtrl(*self.core_registers.get(&HashClockCtrl::ID).unwrap())
            .enable()
            .set_pll_source(0)
            .val();
        reset_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, HashClockCtrl(hash_clk_ctrl)), // CLOCK_CTRL=64
                    dest,
                ),
                delay_ms: 10,
            })
            .unwrap();
        self.core_registers
            .insert(HashClockCtrl::ID, hash_clk_ctrl)
            .unwrap();
        let clk_dly_ctrl =
            ClockDelayCtrlV2(*self.core_registers.get(&ClockDelayCtrlV2::ID).unwrap())
                .set_ccdly(0)
                .set_pwth(4)
                .disable_sweep_frequency_mode()
                .val();
        reset_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, ClockDelayCtrlV2(clk_dly_ctrl)),
                    dest,
                ),
                delay_ms: 10,
            })
            .unwrap();
        self.core_registers
            .insert(ClockDelayCtrlV2::ID, clk_dly_ctrl)
            .unwrap();
        if let Destination::Chip(_) = dest {
            let core_reg2 = 0xAA;
            reset_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        CoreRegisterControl::ADDR,
                        CoreRegisterControl::write_core_reg(0, CoreReg2(core_reg2)),
                        dest,
                    ),
                    delay_ms: 10,
                })
                .unwrap();
            self.core_registers.insert(CoreReg2::ID, core_reg2).unwrap();
        }
        reset_seq
    }

    /// ## Send Hash Frequency command list
    ///
    /// ### Example
    /// ```
    /// use bm1366::{BM1366, BM1366_PLL_ID_HASH};
    /// use bm13xx_asic::{register::*, Asic};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut hash_freq_seq = bm1366.send_hash_freq(HertzU64::MHz(75));
    /// assert_eq!(hash_freq_seq.len(), 4);
    /// assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xa8, 0x02, 0x63, 0x14]);
    // assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xa5, 0x02, 0x54, 0x09]); // seen on S19XP, but equivalent
    /// assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb0, 0x02, 0x73, 9]);
    /// assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xaf, 0x02, 0x64, 0x0d]);
    // assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xa2, 0x02, 0x55, 0x30]); // seen on S19XP, but equivalent
    /// assert_eq!(hash_freq_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb4, 0x02, 0x74, 29]);
    /// assert_eq!(bm1366.plls[BM1366_PLL_ID_HASH].parameter(), 0xc0a8_0263);
    /// ```
    fn send_hash_freq(&mut self, target_freq: HertzU64) -> Vec<CmdDelay, 800> {
        let mut hash_freq_seq = Vec::new();
        if self.plls[BM1366_PLL_ID_HASH].out_div(BM1366_PLL_OUT_HASH) != 0 {
            self.plls[BM1366_PLL_ID_HASH].set_out_div(BM1366_PLL_OUT_HASH, 0);
            hash_freq_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        PLL0Divider::ADDR,
                        self.plls[BM1366_PLL_ID_HASH].divider(),
                        Destination::All,
                    ),
                    delay_ms: 2,
                })
                .unwrap();
            self.registers
                .insert(PLL0Divider::ADDR, self.plls[BM1366_PLL_ID_HASH].divider())
                .unwrap();
        }
        let mut freq = self.hash_freq();
        let mut long_delay = false;
        loop {
            freq += HertzU64::kHz(6250);
            if freq > target_freq {
                freq = target_freq;
            }
            self.set_hash_freq(freq);
            if freq > HertzU64::MHz(380) {
                long_delay = !long_delay;
            }
            hash_freq_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        PLL0Parameter::ADDR,
                        self.plls[BM1366_PLL_ID_HASH].parameter(),
                        Destination::All,
                    ),
                    delay_ms: if long_delay { 2300 } else { 400 },
                })
                .unwrap();
            self.registers
                .insert(
                    PLL0Parameter::ADDR,
                    self.plls[BM1366_PLL_ID_HASH].parameter(),
                )
                .unwrap();
            if freq == target_freq {
                break;
            }
        }
        hash_freq_seq
    }

    /// ## Send Enable Version Rolling command list
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::Asic;
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut vers_roll_seq = bm1366.send_version_rolling(0x1fff_e000);
    /// assert_eq!(vers_roll_seq.len(), 2);
    /// assert_eq!(vers_roll_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa4, 0x90, 0x00, 0xff, 0xff, 0x1c]);
    /// assert_eq!(vers_roll_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x10, 0x00, 0x00, 0x15, 0x1c, 0x02]);
    /// ```
    fn send_version_rolling(&mut self, mask: u32) -> Vec<CmdDelay, 2> {
        let mut vers_roll_seq = Vec::new();
        let hcn = 0x0000_151c;
        vers_roll_seq
            .push(CmdDelay {
                cmd: Command::write_reg(HashCountingNumber::ADDR, hcn, Destination::All),
                delay_ms: 1,
            })
            .unwrap();
        self.registers
            .insert(HashCountingNumber::ADDR, hcn)
            .unwrap();
        let vers_roll = VersionRolling(*self.registers.get(&VersionRolling::ADDR).unwrap())
            .enable()
            .set_mask(mask)
            .val();
        vers_roll_seq
            .push(CmdDelay {
                cmd: Command::write_reg(VersionRolling::ADDR, vers_roll, Destination::All),
                delay_ms: 1,
            })
            .unwrap();
        self.registers
            .insert(VersionRolling::ADDR, vers_roll)
            .unwrap();
        self.enable_version_rolling(mask);
        vers_roll_seq
    }
    fn between_reset_and_set_freq(&mut self) -> Vec<CmdDelay, 40> {
        unimplemented!() // FIXME: A temporary hack this function should be removed
    }
}
