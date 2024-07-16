#![no_std]
//! BM1366 ASIC implementation.

use bm13xx_asic::{
    core_register::{ClockDelayCtrl, HashClockCtrl},
    register::*,
};
use bm13xx_protocol::{
    command::{Command, Destination},
    Bm13xxProtocol, CmdDelay,
};

use core::time::Duration;
use fugit::HertzU32;
use heapless::{FnvIndexMap, Vec};

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
pub struct BM1366 {
    pub sha: bm13xx_asic::sha::Asic<
        BM1366_CORE_CNT,
        BM1366_SMALL_CORE_CNT,
        BM1366_CORE_SMALL_CORE_CNT,
        BM1366_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU32,
    pub plls: [bm13xx_asic::pll::Pll; BM1366_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub version_rolling_enabled: bool,
    pub version_mask: u32,
}

impl BM1366 {
    pub fn new_with_clk(clk: HertzU32) -> Self {
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
    /// use bm1366::BM1366;
    /// use fugit::HertzU32;
    ///
    /// let mut bm1366 = BM1366::default();
    /// assert_eq!(bm1366.hash_freq(), HertzU32::MHz(70u32));
    // bm1366.plls[0].set_parameter(0x40A0_0241); // from Bitaxe default freq
    // assert_eq!(bm1366.hash_freq(), HertzU32::MHz(200u32));
    /// ```
    pub fn hash_freq(&self) -> HertzU32 {
        self.plls[BM1366_PLL_ID_HASH].frequency(self.input_clock_freq, BM1366_PLL_OUT_HASH)
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
            input_clock_freq: HertzU32::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1366_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            version_rolling_enabled: false,
            version_mask: 0x1fffe000,
        };
        // Default PLLs Parameter
        bm1366.plls[0].set_parameter(0xC054_0165);
        bm1366.plls[1].set_parameter(0x2050_0174); /* TODO: understand what is the 2 in MSB */
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
            .insert(ChipNonceOffset::ADDR, 0x0000_0000)
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
            .insert(AnalogMuxControl::ADDR, 0x0000_0000)
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
        // bm1366.registers.insert(PLL2Parameter::ADDR, 0x0000_0000).unwrap();
        // bm1366.registers.insert(PLL3Parameter::ADDR, 0x0000_0000).unwrap();
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
        // bm1366.registers.insert(PLL2Divider::ADDR, 0x0000_0000).unwrap();
        // bm1366.registers.insert(PLL3Divider::ADDR, 0x0000_0000).unwrap();
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
        bm1366
    }
}

impl Bm13xxProtocol for BM1366 {
    /// ## Init the Chip
    ///
    /// ### Example
    /// ```
    /// use bm1366::BM1366;
    /// use bm13xx_asic::register::*;
    /// use bm13xx_protocol::Bm13xxProtocol;
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut init_seq = bm1366.init(256);
    /// assert_eq!(init_seq.len(), 5);
    // assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x41, 0x09, 0x00, 0x2c, 0x00, 0x7c, 0x00, 0x03, 0x03]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x02, 0x11, 0x11, 0x11, 0x06]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x54, 0x00, 0x00, 0x00, 0x03, 0x1d]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xff, 0x08]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x20, 0x19]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x0c]);
    // assert_eq!(bm1366.core_registers.get(&HashClockCtrl::ID).unwrap(), &0x40);
    // assert_eq!(bm1366.core_registers.get(&ClockDelayCtrl::ID).unwrap(), &0x20);
    /// assert_eq!(bm1366.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00ff);
    /// assert_eq!(bm1366.registers.get(&AnalogMuxControl::ADDR).unwrap(), &0x0000_0003);
    /// assert_eq!(bm1366.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(), &0x0211_1111);
    /// ```
    ///
    fn init(&mut self, initial_diffculty: u32) -> Vec<CmdDelay, 20> {
        let mut init_seq = Vec::new();
        // let hash_clk_ctrl = HashClockCtrl(
        //     *self.core_registers.get(&HashClockCtrl::ID).unwrap(),
        // )
        // .set_clock_ctrl(64)
        // .val();
        let hash_clk_ctrl = 64;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, HashClockCtrl(hash_clk_ctrl)), // CLOCK_CTRL=64
                    Destination::All,
                ),
                delay: Duration::from_millis(0),
            })
            .unwrap();
        // self.core_registers.insert(HashClockCtrl::ID, hash_clk_ctrl).unwrap();
        // let clk_dly_ctrl = ClockDelayCtrl(
        //     *self.core_registers.get(&ClockDelayCtrl::ID).unwrap(),
        // )
        // .set_ccdly_sel(0)
        // .set_pwth_sel(2)
        // .val();
        let clk_dly_ctrl = 0x20;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, ClockDelayCtrl(clk_dly_ctrl)), // CCDLY_SEL=0 PWTH_SEL=2 HASH_CLKEN=0 MMEN=0 SWPF_MODE=0
                    Destination::All,
                ),
                delay: Duration::from_millis(0),
            })
            .unwrap();
        // self.core_registers.insert(ClockDelayCtrl::ID, clk_dly_ctrl).unwrap();
        let tck_mask = TicketMask::from_difficulty(initial_diffculty).val();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(TicketMask::ADDR, tck_mask, Destination::All),
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers.insert(TicketMask::ADDR, tck_mask).unwrap();
        // let ana_mux_ctrl = AnalogMuxControl(
        //     *self.registers.get(&AnalogMuxControl::ADDR).unwrap(),
        // )
        // .set_diode_vdd_mux_sel(3)
        // .val();
        let ana_mux_ctrl = 0x0000_0003;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(AnalogMuxControl::ADDR, ana_mux_ctrl, Destination::All), // DIODE_VDD_MUX_SEL=3
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers
            .insert(AnalogMuxControl::ADDR, ana_mux_ctrl)
            .unwrap();
        // let io_drv_st_cfg = IoDriverStrenghtConfiguration(
        //     *self.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(),
        // )
        // .set_io_drv_strength(BO_DS, 0x11)
        // .val();
        let io_drv_st_cfg = 0x0211_1111;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    IoDriverStrenghtConfiguration::ADDR,
                    io_drv_st_cfg,
                    Destination::All,
                ),
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers
            .insert(IoDriverStrenghtConfiguration::ADDR, io_drv_st_cfg)
            .unwrap();
        init_seq
    }

    /// ## Set Baudrate
    ///
    /// ### Example
    /// ```
    /// use bm1366::{BM1366, BM1366_PLL_ID_UART};
    /// use bm13xx_asic::register::*;
    /// use bm13xx_protocol::Bm13xxProtocol;
    ///
    /// let mut bm1366 = BM1366::default();
    /// let mut baud_seq = bm1366.set_baudrate(6_250_000);
    /// assert_eq!(baud_seq.len(), 2);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x05, 0x60, 0x07, 0x00, 23]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x60, 0xc0, 0x70, 0x01, 0x11, 26]);
    /// assert!(bm1366.plls[BM1366_PLL_ID_UART].enabled());
    /// assert_eq!(bm1366.registers.get(&PLL1Parameter::ADDR).unwrap(), &0xC070_0111);
    /// assert_eq!(bm1366.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x0560_0700);
    /// let mut baud_seq = bm1366.set_baudrate(1_000_000);
    /// assert_eq!(baud_seq.len(), 2);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x60, 0x00, 0x70, 0x01, 0x11, 15]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x01, 0x60, 0x02, 0x00, 21]);
    /// assert!(!bm1366.plls[BM1366_PLL_ID_UART].enabled());
    /// assert_eq!(bm1366.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x0160_0200);
    /// assert_eq!(bm1366.registers.get(&PLL1Parameter::ADDR).unwrap(), &0x0070_0111);
    /// ```
    fn set_baudrate(&mut self, baudrate: u32) -> Vec<CmdDelay, 3> {
        let mut baud_seq = Vec::new();
        if baudrate <= self.input_clock_freq.raw() / 8 {
            // TODO: calculate the threshold based on the input clock
            let fbase = self.input_clock_freq.raw();
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
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                .unwrap();
            let pll1_param = self.plls[BM1366_PLL_ID_UART].disable().unlock().parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL1Parameter::ADDR, pll1_param, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(PLL1Parameter::ADDR, pll1_param)
                .unwrap();
        } else {
            let pll1_div4 = 6;
            // self.plls[BM1366_PLL_ID_UART].enable().set_fbdiv(112).set_refdiv(1).set_postdiv1(1).set_postdiv2(1).set_out_div(BM1366_PLL_OUT_UART, pll1_div4);
            self.plls[BM1366_PLL_ID_UART]
                .set_parameter(0xC070_0111)
                .set_out_div(BM1366_PLL_OUT_UART, pll1_div4);
            let fbase = self.plls[BM1366_PLL_ID_UART]
                .frequency(self.input_clock_freq, BM1366_PLL_OUT_UART)
                .raw();
            let pll1_param = self.plls[BM1366_PLL_ID_UART].parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL1Parameter::ADDR, pll1_param, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(PLL1Parameter::ADDR, pll1_param)
                .unwrap();
            let bt8d = (fbase / (8 * baudrate)) - 1;
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
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                .unwrap();
        }
        baud_seq
    }
}
