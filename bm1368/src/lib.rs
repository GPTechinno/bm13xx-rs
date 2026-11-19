//! BM1368 ASIC implementation.

#![no_std]
pub(crate) mod fmt;

use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay, SensorStep, SequenceStep};
use bm13xx_protocol::command::{Command, Destination};

use fugit::HertzU64;
use heapless::index_map::FnvIndexMap;

pub const BM1368_CHIP_ID: u16 = 0x1368;
pub const BM1368_CORE_CNT: usize = 80;
pub const BM1368_SMALL_CORE_CNT: usize = 1276;
pub const BM1368_CORE_SMALL_CORE_CNT: usize = 16;
pub const BM1368_DOMAIN_CNT: usize = 4;
pub const BM1368_PLL_CNT: usize = 4;
pub const BM1368_PLL_ID_HASH: usize = 0; // PLL0 is used for Hashing
pub const BM1368_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 is used for Hashing
pub const BM1368_PLL_ID_UART: usize = 3; // PLL3 can be used for UART Baudrate
pub const BM1368_PLL_OUT_UART: usize = 4; // specifically PLL3_OUT4 can be used for UART Baudrate

/// # Hashboard settings
///
/// Settings that differ from one hashboard to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BoardConfig {
    /// `pwth` of core register `ClockDelayCtrlV3`, written by `init_next()` and `reset_core_next()`.
    pub clock_delay_pwth: u8,
    /// `diode_vdd_mux_sel` of `AnalogMuxControlV2`, written by `init_next()` when the board does.
    pub diode_vdd_mux_sel: Option<u8>,
    /// `CLKO` driver strength in `IoDriverStrenghtConfiguration` of the last chip of each voltage
    /// domain, written by `set_baudrate_next()`.
    pub domain_end_clko_strength: u8,
}

impl BoardConfig {
    /// Antminer S21: 108 chips in 12 voltage domains.
    pub const S21: Self = Self {
        clock_delay_pwth: 6,
        diode_vdd_mux_sel: Some(3),
        domain_end_clko_strength: 15,
    };
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self::S21
    }
}

/// # BM1368
#[derive(Debug)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))] // FnvIndexMap doesn't implement defmt
pub struct BM1368 {
    seq_step: SequenceStep,
    pub sha: bm13xx_asic::sha::Sha<
        BM1368_CORE_CNT,
        BM1368_SMALL_CORE_CNT,
        BM1368_CORE_SMALL_CORE_CNT,
        BM1368_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU64,
    pub plls: [bm13xx_asic::pll::Pll; BM1368_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub core_registers: FnvIndexMap<u8, u8, 16>,
    cno_interval: usize,
    /// Settings of the hashboard the chip is on.
    pub board: BoardConfig,
}

impl BM1368 {
    pub fn new_with_clk(clk: HertzU64) -> Self {
        BM1368 {
            input_clock_freq: clk,
            ..Default::default()
        }
    }

    /// ## Create a BM1368 on a given hashboard
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BoardConfig, BM1368};
    ///
    /// assert_eq!(BM1368::default().board, BoardConfig::S21);
    /// let bm1368 = BM1368::new_with_board(BoardConfig::S21);
    /// assert_eq!(bm1368.board, BoardConfig::S21);
    /// ```
    pub fn new_with_board(board: BoardConfig) -> Self {
        BM1368 {
            board,
            ..Default::default()
        }
    }

    /// ## Set the Chip Address
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    ///
    /// let mut bm1368 = BM1368::default();
    /// bm1368.set_chip_addr(2);
    /// assert_eq!(bm1368.chip_addr, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8) {
        self.chip_addr = chip_addr;
    }

    pub fn set_hash_freq(&mut self, freq: HertzU64) -> &mut Self {
        self.plls[BM1368_PLL_ID_HASH].set_frequency(
            self.input_clock_freq,
            BM1368_PLL_OUT_HASH,
            freq,
            false,
        );
        self
    }

    /// ## Hash counting number for a chain of `chain_asic_num` chips at the current hash frequency
    ///
    /// 80 % of a core's half nonce range time when the chips split the whole nonce range, minus
    /// 800, at least 2800: `⌊0.4 × ⌊⌊⌊2^32 / chain_asic_num⌋ / 128⌋ × 25 / f⌋ − 800⌋`, `f` in MHz.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm1368::BM1368;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1368 = BM1368::default();
    /// bm1368.set_hash_freq(HertzU64::MHz(490));
    /// assert_eq!(bm1368.hash_counting_number(108), 0x15A4);
    /// bm1368.set_hash_freq(HertzU64::MHz(500));
    /// assert_eq!(bm1368.hash_counting_number(1), 670_288); // single chip
    /// ```
    pub fn hash_counting_number(&self, chain_asic_num: usize) -> u32 {
        bm13xx_asic::hash_counting::hash_counting_number(
            (1u64 << 32) / chain_asic_num.max(1) as u64,
            BM1368_CORE_CNT,
            self.hash_freq(),
            80,
        )
        .saturating_sub(800)
        .max(2800)
    }

    /*
    const BM1368_NONCE_CORES_BITS: usize = 7; // Core ID is hardcoded on Nonce[31:25] -> 7 bits
    const BM1368_NONCE_CORES_MASK: u32 = 0b111_1111;
    const BM1368_NONCE_CHIP_BITS: usize = 16; // Chip ID is hardcoded left-aligned on Nonce[24:9] -> 16 bits
    const BM1368_NONCE_CHIP_MASK: u32 = 0b1111_1111_1111_1111;

    /// ## Get the Core ID that produced a given Nonce
    ///
    /// Core ID is always hardcoded in Nonce\[31:25\].
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.nonce2core_id(0x12345678), 0x09);
    /// assert_eq!(bm1368.nonce2core_id(0x906732c8), 72); // first Bitaxe Block 853742
    /// ```
    pub fn nonce2core_id(&self, nonce: u32) -> usize {
        ((nonce >> (NONCE_BITS - BM1368_NONCE_CORES_BITS)) & BM1368_NONCE_CORES_MASK) as usize
    }

    /// ## Get the Chip ID that produced a given Nonce
    ///
    /// If the Nonce space has been splitted between chips (usage of ChipNonceOffset),
    /// the Chip ID is hardcoded in Nonce\[24:9\] left aligned.
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.nonce2chip_id(0xfe00_01ff, 65), 0);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe07_e1ff, 65), 0);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe07_e3ff, 65), 1);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe0f_c1ff, 65), 1);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe0f_c3ff, 65), 2);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe17_a1ff, 65), 2);
    /// assert_eq!(bm1368.nonce2chip_id(0xfe17_a3ff, 65), 3);
    /// ```
    pub fn nonce2chip_id(&self, nonce: u32, chain_asic_num: usize) -> usize {
        (((nonce >> (NONCE_BITS - BM1368_NONCE_CORES_BITS - BM1368_NONCE_CHIP_BITS))
            & BM1368_NONCE_CHIP_MASK) as usize
            * chain_asic_num)
            >> 16
    }
    */
}

impl Default for BM1368 {
    fn default() -> Self {
        let mut bm1368 = Self {
            seq_step: SequenceStep::default(),
            sha: bm13xx_asic::sha::Sha::default(),
            input_clock_freq: HertzU64::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1368_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            core_registers: FnvIndexMap::<_, _, 16>::new(),
            cno_interval: 0,
            board: BoardConfig::default(),
        };
        bm1368.reset();
        bm1368
    }
}

impl Asic for BM1368 {
    /// ## Reset the Chip to default state
    fn reset(&mut self) {
        self.seq_step = SequenceStep::default();
        self.sha = bm13xx_asic::sha::Sha::default();
        self.input_clock_freq = HertzU64::MHz(25);
        self.plls = [bm13xx_asic::pll::Pll::default(); BM1368_PLL_CNT];
        self.chip_addr = 0;
        self.registers = FnvIndexMap::<_, _, 64>::new();
        self.core_registers = FnvIndexMap::<_, _, 16>::new();
        // Default PLLs Parameter
        self.plls[0].set_parameter(0xC054_0165);
        self.plls[1].set_parameter(0xa050_0174);
        self.plls[2].set_parameter(0xa050_0174);
        self.plls[3].set_parameter(0x0000_0000);
        // Default PLLs Divider
        self.plls[0].set_divider(0x0000_0000);
        self.plls[1].set_divider(0x0000_0000);
        self.plls[2].set_divider(0x0000_0000);
        self.plls[3].set_divider(0x0000_0000);
        // Default Registers Value
        self.registers
            .insert(ChipIdentification::ADDR, 0x1368_0000)
            .unwrap();
        self.registers
            .insert(Time1sCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL0Parameter::ADDR, 0xc054_0165)
            .unwrap();
        self.registers
            .insert(ChipNonceOffsetV2::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(HashCountingNumber::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(TicketMask::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(MiscControl::ADDR, 0x0000_c100)
            .unwrap();
        self.registers
            .insert(I2CControl::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(OrderedClockEnable::ADDR, 0x0000_0007)
            .unwrap();
        self.registers
            .insert(TopProcessMonitor::ADDR, 0x0010_0000)
            .unwrap();
        self.registers
            .insert(FastUARTConfigurationV2::ADDR, 0x0130_1a00)
            .unwrap();
        self.registers.insert(UARTRelay::ADDR, 0x000f_0000).unwrap();
        self.registers
            .insert(CoreNumber::ADDR, 0x0000_0050)
            .unwrap();
        self.registers.insert(Reg34::ADDR, 0x0000_0000).unwrap();
        self.registers
            .insert(TicketMask2::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(CoreRegisterControl::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(CoreRegisterValue::ADDR, 0x0802_0100)
            .unwrap();
        self.registers
            .insert(ExternalTemperatureSensorRead::ADDR, 0x0000_0000)
            .unwrap();
        self.registers.insert(ErrorFlag::ADDR, 0x0000_0000).unwrap();
        self.registers
            .insert(NonceErrorCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(NonceOverflowCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(AnalogMuxControlV2::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(IoDriverStrenghtConfiguration::ADDR, 0x0001_2111)
            .unwrap();
        self.registers.insert(TimeOut::ADDR, 0x0000_FFFF).unwrap();
        self.registers
            .insert(PLL1Parameter::ADDR, 0xa050_0174)
            .unwrap();
        self.registers
            .insert(PLL2Parameter::ADDR, 0xa050_0174)
            .unwrap();
        self.registers
            .insert(RoscPadDisable::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(OrderedClockMonitor::ADDR, 0x0001_0200)
            .unwrap();
        self.registers
            .insert(PLL0Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL1Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL2Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL3Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(ClockOrderControl0::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(ClockOrderControl1::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(GotNonceCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(NtickNonceCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(GoldenNonceForSweepReturn::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(ReturnedGroupPatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(NonceReturnedTimeout::ADDR, 0x00ff_0f07)
            .unwrap();
        self.registers
            .insert(ReturnedSinglePatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(MidstateCalc::ADDR, 0x0000_ffff)
            .unwrap();

        self.registers
            .insert(SoftResetControl::ADDR, 0x0007_0000)
            .unwrap();
        self.registers.insert(RegAC::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegB0::ADDR, 0x8000_0000).unwrap();
        self.registers.insert(RegB4::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(AdcCtrl0::ADDR, 0x2000_0000).unwrap();
        self.registers.insert(AdcCtrl1::ADDR, 0x0000_4480).unwrap();
        // self.registers.insert(RegBA::ADDR, 0x0000_0000).unwrap();
        // self.registers.insert(RegBB::ADDR, 0x0000_0000).unwrap();
        self.registers
            .insert(Pll0ChargePumpProgramming::ADDR, 0x0000_3313)
            .unwrap();
        // self.registers.insert(RegBD::ADDR, 0x0000_0000).unwrap();
        self.registers
            .insert(Pll0OffsetCalibration::ADDR, 0x0000_2000)
            .unwrap();
        self.registers
            .insert(AutoWorkCfg::ADDR, 0x0000_b898)
            .unwrap();
        self.registers.insert(RegC8::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegCC::ADDR, 0x0000_0000).unwrap();
        self.registers
            .insert(FrequencySweepControl::ADDR, 0x0000_0070)
            .unwrap();
        self.registers.insert(RegD4::ADDR, 0x0037_6400).unwrap();
        self.registers.insert(RegD8::ADDR, 0x3030_3030).unwrap();
        self.registers
            .insert(SweepNonceRetTimeout::ADDR, 0x0000_ffff)
            .unwrap();
        self.registers.insert(RegE0::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegE4::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegE8::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegEC::ADDR, 0x0000_0008).unwrap();
        self.registers.insert(RegF0::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegF4::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegF8::ADDR, 0x0000_0000).unwrap();
        self.registers.insert(RegFC::ADDR, 0x0000_0000).unwrap();
        // Default Core Registers Value
        self.core_registers
            .insert(ClockDelayCtrlV3::ID, 0x98)
            .unwrap();
        // self.core_registers.insert(1, 0x00).unwrap(); // not used anywhere in official FW
        self.core_registers.insert(CoreEnableV2::ID, 0x00).unwrap();
        // self.core_registers.insert(CoreError::ID, 0x00).unwrap();
        // self.core_registers.insert(CoreEnable::ID, 0x08).unwrap();
        self.core_registers
            .insert(HashClockCtrlV2::ID, 0x00)
            .unwrap();
        self.core_registers
            .insert(HashClockCounterV2::ID, 0x00)
            .unwrap();
        self.core_registers.insert(PgMont::ID, 0x08).unwrap();
        // self
        //     .core_registers
        //     .insert(SweepClockCtrl::ID, 0x00)
        //     .unwrap();
        self.core_registers.insert(AutoClkEn::ID, 0x00).unwrap();
        // self.core_registers.insert(9, 0x00).unwrap();
        // self.core_registers.insert(9, 0x00).unwrap();
        self.core_registers
            .insert(ClockSelectCtrl::ID, 0x00)
            .unwrap();
        // self.core_registers.insert(12, 0x00).unwrap();
        // self.core_registers.insert(13, 0x15).unwrap();
        // self.core_registers.insert(14, 0x00).unwrap();
        // self.core_registers.insert(15, 0x00).unwrap();
        // self.core_registers.insert(16, 0xbe).unwrap();
        // self.core_registers.insert(17, 0x00).unwrap();
        // self.core_registers.insert(18, 0x00).unwrap();
        // self.core_registers.insert(19, 0x00).unwrap();
        // self.core_registers.insert(20, 0x00).unwrap();
        // self.core_registers.insert(21, 0x00).unwrap();
        self.core_registers.insert(OcvPulseWidth::ID, 0x00).unwrap();
        // self.core_registers.insert(23, 0x80).unwrap();
    }

    /// ## Get the Chip ID
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.chip_id(), 0x1368);
    /// ```
    fn chip_id(&self) -> u16 {
        BM1368_CHIP_ID
    }

    /// ## Get the Chip Core count
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.core_count(), BM1368_CORE_CNT);
    /// ```
    fn core_count(&self) -> usize {
        self.sha.core_count()
    }

    /// ## Get the Chip Small Core count per Core
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_CORE_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.core_small_core_count(), BM1368_CORE_SMALL_CORE_CNT);
    /// ```
    fn core_small_core_count(&self) -> usize {
        self.sha.core_small_core_count()
    }

    /// ## Get the Chip Small Core count
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.small_core_count(), BM1368_SMALL_CORE_CNT);
    /// ```
    fn small_core_count(&self) -> usize {
        self.sha.small_core_count()
    }

    fn cno_interval(&self) -> usize {
        self.cno_interval
    }

    fn cno_bits(&self) -> u32 {
        ChipNonceOffsetV2::CNO_MASK.count_ones()
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_PLL_ID_HASH};
    /// use bm13xx_asic::Asic;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.hash_freq(), HertzU64::MHz(50));
    /// assert_eq!(bm1368.set_hash_freq(HertzU64::MHz(200)).hash_freq(), HertzU64::MHz(200));
    /// ```
    fn hash_freq(&self) -> HertzU64 {
        self.plls[BM1368_PLL_ID_HASH].frequency(self.input_clock_freq, BM1368_PLL_OUT_HASH)
    }

    /// ## Init the Chip command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    ///
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x8b, 0x00, 0x12], delay_ms: 10}));
    /// assert_eq!(bm1368.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x18, 0x1f], delay_ms: 10}));
    /// assert_eq!(bm1368.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xff, 0x08], delay_ms: 10}));
    /// assert_eq!(bm1368.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x54, 0x00, 0x00, 0x00, 0x03, 0x1d], delay_ms: 0}));
    /// assert_eq!(bm1368.init_next(256), None);
    /// assert_eq!(bm1368.core_registers.get(&ClockSelectCtrl::ID).unwrap(), &0x00);
    /// assert_eq!(bm1368.core_registers.get(&ClockDelayCtrlV3::ID).unwrap(), &0x18);
    /// assert_eq!(bm1368.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00ff);
    /// assert_eq!(bm1368.registers.get(&AnalogMuxControlV2::ADDR).unwrap(), &0x0000_0003);
    /// ```
    fn init_next(&mut self, difficulty: u32) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::Init(step) => {
                match step {
                    0 => {
                        self.seq_step = SequenceStep::Init(1);
                        // the whole register is written, other fields cleared
                        let clk_dly_ctrl = ClockDelayCtrlV3(0)
                            .set_ccdly(0)
                            .set_pwth(self.board.clock_delay_pwth)
                            .disable_sweep_frequency_mode()
                            .val();
                        self.core_registers
                            .insert(ClockDelayCtrlV3::ID, clk_dly_ctrl)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    ClockDelayCtrlV3(clk_dly_ctrl),
                                ),
                                Destination::All,
                            ),
                            delay_ms: 10,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::Init(2);
                        let tck_mask = TicketMask::from_difficulty(difficulty).val();
                        self.registers.insert(TicketMask::ADDR, tck_mask).unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(TicketMask::ADDR, tck_mask, Destination::All),
                            delay_ms: 10,
                        })
                    }
                    2 => {
                        self.seq_step = SequenceStep::Init(3);
                        // only on the boards that set it
                        let Some(diode_vdd_mux_sel) = self.board.diode_vdd_mux_sel else {
                            self.seq_step = SequenceStep::None;
                            return None;
                        };
                        let ana_mux_ctrl = AnalogMuxControlV2(
                            *self.registers.get(&AnalogMuxControlV2::ADDR).unwrap(),
                        )
                        .set_diode_vdd_mux_sel(diode_vdd_mux_sel)
                        .val();
                        self.registers
                            .insert(AnalogMuxControlV2::ADDR, ana_mux_ctrl)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                AnalogMuxControlV2::ADDR,
                                ana_mux_ctrl,
                                Destination::All,
                            ),
                            delay_ms: 0,
                        })
                    }
                    3 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                // authorize an Init sequence start whatever the current step was
                self.seq_step = SequenceStep::Init(0);
                let reg11 =
                    ClockSelectCtrl(*self.core_registers.get(&ClockSelectCtrl::ID).unwrap()).val();
                // reg11 = 0x00; // TODO: replace by some field specific method when they are known
                self.core_registers
                    .insert(ClockSelectCtrl::ID, reg11)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        CoreRegisterControl::ADDR,
                        CoreRegisterControl::write_core_reg(0, ClockSelectCtrl(reg11)),
                        Destination::All,
                    ),
                    delay_ms: 10,
                })
            }
        }
    }

    /// ## Send Baudrate command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    ///
    /// // 108 chips in 12 voltage domains of 9, address interval 2
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x02, 0x11, 0x11, 0x11, 0x06], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xd6, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x14], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xc4, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x08], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xb2, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x05], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xa0, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x19], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x8e, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x08], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x7c, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x06], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x6a, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x01], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x58, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x1b], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x46, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x0f], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x34, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x19], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x22, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x1e], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x58, 0x02, 0x11, 0xf1, 0x11, 0x04], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0x5a, 0xa5, 0x5a, 0xa5, 0x1c], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xc6, 0x2c, 0x00, 0x17, 0x00, 0x03, 0x1c], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xd6, 0x2c, 0x00, 0x17, 0x00, 0x03, 0x1f], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xb4, 0x2c, 0x00, 0x20, 0x00, 0x03, 0x06], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xc4, 0x2c, 0x00, 0x20, 0x00, 0x03, 0x0f], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xa2, 0x2c, 0x00, 0x29, 0x00, 0x03, 0x07], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xb2, 0x2c, 0x00, 0x29, 0x00, 0x03, 0x04], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x90, 0x2c, 0x00, 0x32, 0x00, 0x03, 0x17], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0xa0, 0x2c, 0x00, 0x32, 0x00, 0x03, 0x12], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x7e, 0x2c, 0x00, 0x3b, 0x00, 0x03, 0x14], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x8e, 0x2c, 0x00, 0x3b, 0x00, 0x03, 0x05], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x6c, 0x2c, 0x00, 0x44, 0x00, 0x03, 0x11], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x7c, 0x2c, 0x00, 0x44, 0x00, 0x03, 0x12], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x5a, 0x2c, 0x00, 0x4d, 0x00, 0x03, 0x16], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x6a, 0x2c, 0x00, 0x4d, 0x00, 0x03, 0x13], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x48, 0x2c, 0x00, 0x56, 0x00, 0x03, 0x00], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x58, 0x2c, 0x00, 0x56, 0x00, 0x03, 0x03], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x36, 0x2c, 0x00, 0x5f, 0x00, 0x03, 0x18], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x46, 0x2c, 0x00, 0x5f, 0x00, 0x03, 0x11], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x24, 0x2c, 0x00, 0x68, 0x00, 0x03, 0x08], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x34, 0x2c, 0x00, 0x68, 0x00, 0x03, 0x0b], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x12, 0x2c, 0x00, 0x71, 0x00, 0x03, 0x16], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x22, 0x2c, 0x00, 0x71, 0x00, 0x03, 0x13], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x2c, 0x00, 0x7a, 0x00, 0x03, 0x19], delay_ms: 0}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x2c, 0x00, 0x7a, 0x00, 0x03, 0x1a], delay_ms: 200}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x01, 0x30, 0x00, 0x00, 0x1a], delay_ms: 200}));
    /// assert_eq!(bm1368.set_baudrate_next(3_125_000, 12, 9, 2), None);
    /// assert_eq!(bm1368.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(), &0x0211_1111);
    /// assert_eq!(bm1368.registers.get(&RoscPadDisable::ADDR).unwrap(), &0x5aa5_5aa5);
    /// assert_eq!(bm1368.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x0130_0000);
    /// ```
    fn set_baudrate_next(
        &mut self,
        baudrate: u32,
        chain_domain_cnt: usize,
        domain_asic_cnt: usize,
        asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        let sub_seq1_start = 0;
        let sub_seq2_start = sub_seq1_start + chain_domain_cnt;
        let sub_seq3_start = sub_seq2_start + 1;
        let sub_seq4_start = sub_seq3_start + chain_domain_cnt;
        let sub_seq5_start = sub_seq4_start + chain_domain_cnt;
        let sub_seq6_start = sub_seq5_start + 1;
        let end = sub_seq6_start + 1;
        let pll3_div4 = 6;
        match self.seq_step {
            SequenceStep::Baudrate(step) => {
                if (sub_seq1_start..sub_seq2_start).contains(&step) {
                    self.seq_step = SequenceStep::Baudrate(step + 1);
                    // the last chip of each voltage domain gets a stronger CLKO driver
                    // (iterating voltage domain in decreasing chip address order)
                    let dom = sub_seq2_start - step - 1;
                    let io_drv_st_cfg = IoDriverStrenghtConfiguration(
                        *self
                            .registers
                            .get(&IoDriverStrenghtConfiguration::ADDR)
                            .unwrap(),
                    )
                    .set_strenght(DriverSelect::CLKO, self.board.domain_end_clko_strength)
                    .val();
                    // do not save any chip-specific value
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            IoDriverStrenghtConfiguration::ADDR,
                            io_drv_st_cfg,
                            Destination::Chip(
                                (((dom + 1) * domain_asic_cnt - 1) * asic_addr_interval) as u8,
                            ),
                        ),
                        delay_ms: 0,
                    })
                } else if step == sub_seq2_start {
                    self.seq_step = SequenceStep::Baudrate(sub_seq3_start);
                    let rosc = RoscPadDisable(*self.registers.get(&RoscPadDisable::ADDR).unwrap())
                        .disable()
                        .val();
                    self.registers.insert(RoscPadDisable::ADDR, rosc).unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(RoscPadDisable::ADDR, rosc, Destination::All),
                        delay_ms: 0,
                    })
                } else if (sub_seq3_start..sub_seq4_start).contains(&step) {
                    // first and last chip of each voltage domain should have UARTRelay with
                    // GAP_CNT=domain_asic_num*(chain_domain_num-domain_i)+14
                    // RO_REL_EN=CO_REL_EN=1
                    // (iterating voltage domain in decreasing chip address order)
                    self.seq_step = SequenceStep::Baudrate(step + chain_domain_cnt);
                    // jump to next sub-seq to alternate
                    let dom = sub_seq4_start - step - 1;
                    let uart_delay = UARTRelay(*self.registers.get(&UARTRelay::ADDR).unwrap())
                        .set_gap_cnt(
                            (domain_asic_cnt as u16) * ((chain_domain_cnt as u16) - (dom as u16))
                                + 14,
                        )
                        .enable_ro_relay()
                        .enable_co_relay()
                        .val();
                    // do not save any chip-specific value
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            UARTRelay::ADDR,
                            uart_delay,
                            Destination::Chip((dom * domain_asic_cnt * asic_addr_interval) as u8),
                        ),
                        delay_ms: 0,
                    })
                } else if (sub_seq4_start..sub_seq5_start).contains(&step) {
                    // same for last chip of each voltage domain
                    self.seq_step = SequenceStep::Baudrate(if step == sub_seq5_start - 1 {
                        sub_seq5_start
                    } else {
                        step - chain_domain_cnt + 1
                    });
                    // jump back to previous sub-seq to alternate
                    let dom = sub_seq5_start - step - 1;
                    let uart_delay = UARTRelay(*self.registers.get(&UARTRelay::ADDR).unwrap())
                        .set_gap_cnt(
                            (domain_asic_cnt as u16) * ((chain_domain_cnt as u16) - (dom as u16))
                                + 14,
                        )
                        .enable_ro_relay()
                        .enable_co_relay()
                        .val();
                    // do not save any chip-specific value
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            UARTRelay::ADDR,
                            uart_delay,
                            Destination::Chip(
                                (((dom + 1) * domain_asic_cnt - 1) * asic_addr_interval) as u8,
                            ),
                        ),
                        delay_ms: if step == sub_seq5_start - 1 { 200 } else { 0 },
                    })
                } else if step == sub_seq5_start {
                    if baudrate <= self.input_clock_freq.to_raw() as u32 / 8 {
                        self.seq_step = SequenceStep::Baudrate(end);
                        let fbase = self.input_clock_freq.to_raw() as u32;
                        let bt8d = (fbase / (8 * baudrate)) - 1;
                        let fast_uart_cfg = FastUARTConfigurationV2(
                            *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
                        )
                        .clr_b28()
                        // .set_b24()
                        .set_bclk_sel(BaudrateClockSelectV2::Clki)
                        .set_bt8d(bt8d as u8)
                        .val();
                        self.registers
                            .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                FastUARTConfigurationV2::ADDR,
                                fast_uart_cfg,
                                Destination::All,
                            ),
                            delay_ms: 200,
                        })
                    } else {
                        self.seq_step = SequenceStep::Baudrate(sub_seq6_start);
                        // TODO: 0x68 is RoscPadDisable on the BM1368; above 3.125 Mbaud the UART
                        // clock comes from PLL1 (0x60).
                        self.plls[BM1368_PLL_ID_UART]
                            // .set_parameter(0xC070_0111)
                            .lock()
                            .enable()
                            .set_fb_div(112)
                            .set_ref_div(1)
                            .set_post1_div(1)
                            .set_post2_div(1)
                            .set_out_div(BM1368_PLL_OUT_UART, pll3_div4);
                        let pll3_param = self.plls[BM1368_PLL_ID_UART].parameter();
                        self.registers
                            .insert(PLL3Parameter::ADDR, pll3_param)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                PLL3Parameter::ADDR,
                                pll3_param,
                                Destination::All,
                            ),
                            delay_ms: 0,
                        })
                    }
                } else if step == sub_seq6_start {
                    self.seq_step = SequenceStep::Baudrate(end);
                    if baudrate <= self.input_clock_freq.to_raw() as u32 / 8 {
                        // should not be reached for 2 reasons:
                        // - in step above we jump directly to end
                        // - after setting the chip's FastUartConfiguration with bclk_sel(BaudrateClockSelectV2::Clki) in previous step
                        //   the chip's baudrate should immediatly adapt and thus this new step with old baudrate from control side
                        //   will be ignored by the chip.
                        let pll3_param =
                            self.plls[BM1368_PLL_ID_UART].disable().unlock().parameter();
                        self.registers
                            .insert(PLL3Parameter::ADDR, pll3_param)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                PLL3Parameter::ADDR,
                                pll3_param,
                                Destination::All,
                            ),
                            delay_ms: 0,
                        })
                    } else {
                        let fbase = self.plls[BM1368_PLL_ID_UART]
                            .frequency(self.input_clock_freq, BM1368_PLL_OUT_UART)
                            .to_raw();
                        let bt8d = (fbase as u32 / (2 * baudrate)) - 1;
                        let fast_uart_cfg = FastUARTConfigurationV2(
                            *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
                        )
                        .set_pll1_div4(pll3_div4) // TODO: not sure yet where the pll3_div4 really fit into FastUartConfiguration
                        .set_bclk_sel(BaudrateClockSelectV2::Pll1) // TODO: it should be Pll3, but not sure about the BCLK_SEL field yet for it
                        .set_bt8d(bt8d as u8)
                        .val();
                        self.registers
                            .insert(FastUARTConfigurationV2::ADDR, fast_uart_cfg)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                FastUARTConfigurationV2::ADDR,
                                fast_uart_cfg,
                                Destination::All,
                            ),
                            delay_ms: 0,
                        })
                    }
                } else if step == end {
                    self.seq_step = SequenceStep::None;
                    None
                } else {
                    unreachable!("step={}", step)
                }
            }
            _ => {
                // authorize a SetBaudrate sequence start whatever the current step was
                self.seq_step = SequenceStep::Baudrate(sub_seq1_start);
                let io_drv_st_cfg = IoDriverStrenghtConfiguration(
                    *self
                        .registers
                        .get(&IoDriverStrenghtConfiguration::ADDR)
                        .unwrap(),
                )
                .set_strenght(DriverSelect::RF, 2)
                .disable(DriverRSelect::D3R)
                .disable(DriverRSelect::D2R)
                .disable(DriverRSelect::D1R)
                .enable(DriverRSelect::D0R)
                .set_strenght(DriverSelect::RO, 1)
                .set_strenght(DriverSelect::CLKO, 1)
                .set_strenght(DriverSelect::NRSTO, 1)
                .set_strenght(DriverSelect::BO, 1)
                .set_strenght(DriverSelect::CO, 1)
                .val();
                self.registers
                    .insert(IoDriverStrenghtConfiguration::ADDR, io_drv_st_cfg)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        IoDriverStrenghtConfiguration::ADDR,
                        io_drv_st_cfg,
                        Destination::All,
                    ),
                    delay_ms: 0,
                })
            }
        }
    }

    /// ## Reset the Chip Cores command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{Asic, CmdDelay};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.reset_core_next(Destination::All), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa8, 0x00, 0x07, 0x00, 0x00, 0x03], delay_ms: 0}));
    /// assert_eq!(bm1368.reset_core_next(Destination::All), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0xff, 0x0f, 0xc1, 0x00, 0x00], delay_ms: 100}));
    /// assert_eq!(bm1368.reset_core_next(Destination::All), None);
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0xa8, 0x00, 0x07, 0x01, 0xf0, 0x15], delay_ms: 10}));
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x18, 0xf0, 0x00, 0xc1, 0x00, 0x0c], delay_ms: 10}));
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x8b, 0x00, 0x1a], delay_ms: 10}));
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x18, 0x17], delay_ms: 10}));
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x82, 0xaa, 0x05], delay_ms: 10}));
    /// assert_eq!(bm1368.reset_core_next(Destination::Chip(0)), None);
    /// ```
    fn reset_core_next(&mut self, dest: Destination) -> Option<CmdDelay> {
        if dest == Destination::All {
            match self.seq_step {
                SequenceStep::ResetCore(step) => match step {
                    0 => {
                        self.seq_step = SequenceStep::ResetCore(1);
                        let misc =
                            MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
                                .set_core_return_nonce(0xf)
                                .set_b27_26(3)
                                .set_b25_24(3)
                                .set_b19_16(0xf)
                                .val();
                        self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                            delay_ms: 100,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => unreachable!(),
                },
                _ => {
                    // authorize a ResetCore sequence start whatever the current step was
                    self.seq_step = SequenceStep::ResetCore(0);
                    let soft_reset_ctrl =
                        SoftResetControl(*self.registers.get(&SoftResetControl::ADDR).unwrap())
                            .clr_b10()
                            .clr_b8()
                            .set_b7_4(0)
                            .set_b3_0(0)
                            .val();
                    self.registers
                        .insert(SoftResetControl::ADDR, soft_reset_ctrl)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(SoftResetControl::ADDR, soft_reset_ctrl, dest),
                        delay_ms: 0,
                    })
                }
            }
        } else {
            match self.seq_step {
                SequenceStep::ResetCore(step) => match step {
                    0 => {
                        self.seq_step = SequenceStep::ResetCore(1);
                        let misc =
                            MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
                                .set_core_return_nonce(0xf)
                                .set_b27_26(0)
                                .set_b25_24(0)
                                .set_b19_16(0)
                                .val();
                        self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                            delay_ms: 10,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::ResetCore(2);
                        // let core_reg_11 =
                        //     ClockSelectCtrl(*self.core_registers.get(&ClockSelectCtrl::ID).unwrap())
                        //         .val();
                        let core_reg_11 = 0x00;
                        self.core_registers
                            .insert(ClockSelectCtrl::ID, core_reg_11)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    ClockSelectCtrl(core_reg_11),
                                ),
                                dest,
                            ),
                            delay_ms: 10,
                        })
                    }
                    2 => {
                        self.seq_step = SequenceStep::ResetCore(3);
                        let clk_dly_ctrl = ClockDelayCtrlV3(0)
                            .set_ccdly(0)
                            .set_pwth(self.board.clock_delay_pwth)
                            .disable_sweep_frequency_mode()
                            .val();
                        self.core_registers
                            .insert(ClockDelayCtrlV3::ID, clk_dly_ctrl)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    ClockDelayCtrlV3(clk_dly_ctrl),
                                ),
                                dest,
                            ),
                            delay_ms: 10,
                        })
                    }
                    3 => {
                        self.seq_step = SequenceStep::ResetCore(4);
                        let core_reg2 = 0xAA;
                        self.core_registers
                            .insert(CoreEnableV2::ID, core_reg2)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(0, CoreEnableV2(core_reg2)),
                                dest,
                            ),
                            delay_ms: 10,
                        })
                    }
                    4 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => unreachable!(),
                },
                _ => {
                    // authorize a ResetCore sequence start whatever the current step was
                    self.seq_step = SequenceStep::ResetCore(0);
                    let soft_rst_ctrl =
                        SoftResetControl(*self.registers.get(&SoftResetControl::ADDR).unwrap())
                            .clr_b10()
                            .set_b8()
                            .set_b7_4(0xf)
                            .set_b3_0(0)
                            .val();
                    self.registers
                        .insert(SoftResetControl::ADDR, soft_rst_ctrl)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(SoftResetControl::ADDR, soft_rst_ctrl, dest),
                        delay_ms: 10,
                    })
                }
            }
        }
    }

    /// ## Send Hash Frequency command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_PLL_ID_HASH};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb4, 0x02, 0x74, 0x1d], delay_ms: 400}));
    /// assert_eq!(bm1368.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xaf, 0x02, 0x64, 0x0d], delay_ms: 400}));
    /// assert_eq!(bm1368.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb0, 0x02, 0x73, 0x09], delay_ms: 400}));
    /// assert_eq!(bm1368.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xa8, 0x02, 0x63, 0x14], delay_ms: 400}));
    /// assert_eq!(bm1368.set_hash_freq_next(HertzU64::MHz(75)), None);
    /// assert_eq!(bm1368.plls[BM1368_PLL_ID_HASH].parameter(), 0xc0a8_0263);
    /// // the last word sent is the target, even off the 6.25 MHz grid
    /// let mut last = [0; 11];
    /// while let Some(step) = bm1368.set_hash_freq_next(HertzU64::MHz(80)) {
    ///     last = step.cmd;
    /// }
    /// assert_eq!(last[6..10], bm1368.plls[BM1368_PLL_ID_HASH].parameter().to_be_bytes());
    /// // and going down
    /// let step = bm1368.set_hash_freq_next(HertzU64::MHz(75)).unwrap();
    /// assert_eq!(step.cmd[5], PLL0Parameter::ADDR);
    /// last = step.cmd;
    /// while let Some(step) = bm1368.set_hash_freq_next(HertzU64::MHz(75)) {
    ///     last = step.cmd;
    /// }
    /// assert_eq!(last[6..10], bm1368.plls[BM1368_PLL_ID_HASH].parameter().to_be_bytes());
    /// ```
    fn set_hash_freq_next(&mut self, target_freq: HertzU64) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::HashFreq(1) => {
                self.seq_step = SequenceStep::None;
                None
            }
            SequenceStep::HashFreq(_) => {
                let step = HertzU64::kHz(6250);
                let current = self.hash_freq();
                let freq = if current + step < target_freq {
                    current + step
                } else if current > target_freq + step {
                    current - step
                } else {
                    target_freq
                };
                let prev_pll0_param = self.plls[BM1368_PLL_ID_HASH].parameter();
                self.set_hash_freq(freq);
                let pll0_param = self.plls[BM1368_PLL_ID_HASH].parameter();
                // stop at the target, or if the PLL can't get closer to it
                if freq == target_freq || pll0_param == prev_pll0_param {
                    self.seq_step = SequenceStep::HashFreq(1);
                }
                self.registers
                    .insert(PLL0Parameter::ADDR, pll0_param)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(PLL0Parameter::ADDR, pll0_param, Destination::All),
                    delay_ms: if freq > HertzU64::MHz(550) { 2700 } else { 400 },
                })
            }
            _ => {
                // authorize a SetHashFreq sequence start whatever the current step was
                self.seq_step = SequenceStep::HashFreq(2);
                self.set_hash_freq_next(target_freq)
            }
        }
    }

    /// ## Set the hash frequency of one chip
    ///
    /// One `PLL0Parameter` write to `dest`, without ramp. `hash_freq()` keeps the frequency of the
    /// whole chain.
    ///
    /// ### Example
    /// ```
    /// use bm1368::{BM1368, BM1368_PLL_ID_HASH};
    /// use bm13xx_asic::{register::PLL0Parameter, Asic};
    /// use bm13xx_protocol::command::Destination;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1368 = BM1368::default();
    /// let step = bm1368.set_chip_hash_freq_next(Destination::Chip(4), HertzU64::MHz(75)).unwrap();
    /// assert_eq!(step.cmd[2..6], [0x41, 0x09, 0x04, PLL0Parameter::ADDR]);
    /// let mut at_75_mhz = BM1368::default();
    /// at_75_mhz.set_hash_freq(HertzU64::MHz(75));
    /// assert_eq!(step.cmd[6..10], at_75_mhz.plls[BM1368_PLL_ID_HASH].parameter().to_be_bytes());
    /// assert_eq!(bm1368.set_chip_hash_freq_next(Destination::Chip(4), HertzU64::MHz(75)), None);
    /// assert_eq!(bm1368.hash_freq(), HertzU64::MHz(50));
    /// ```
    fn set_chip_hash_freq_next(&mut self, dest: Destination, freq: HertzU64) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::ChipHashFreq(_) => {
                self.seq_step = SequenceStep::None;
                None
            }
            _ => {
                self.seq_step = SequenceStep::ChipHashFreq(0);
                let mut pll = self.plls[BM1368_PLL_ID_HASH];
                pll.set_frequency(self.input_clock_freq, BM1368_PLL_OUT_HASH, freq, false);
                Some(CmdDelay {
                    cmd: Command::write_reg(PLL0Parameter::ADDR, pll.parameter(), dest),
                    delay_ms: 0,
                })
            }
        }
    }

    /// ## Index of the chip that found a nonce
    ///
    /// With version rolling, once the nonce range is split between the chips
    /// (`split_nonce_between_chips_next()`), chip `i` of `n` covers the nonces whose bits 24:9 are
    /// from `⌈i × 0x10000 / n⌉` on. Without version rolling, `None`.
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1368 = BM1368::default();
    /// assert_eq!(bm1368.nonce_chip_index(0xfe00_01ff, true, 65, 2), Some(0));
    /// assert_eq!(bm1368.nonce_chip_index(0xfe07_e1ff, true, 65, 2), Some(0));
    /// assert_eq!(bm1368.nonce_chip_index(0xfe07_e3ff, true, 65, 2), Some(1));
    /// assert_eq!(bm1368.nonce_chip_index(0xfe0f_c3ff, true, 65, 2), Some(2));
    /// assert_eq!(bm1368.nonce_chip_index(0xfe0f_c3ff, false, 65, 2), None);
    /// ```
    fn nonce_chip_index(
        &self,
        nonce: u32,
        version_rolling: bool,
        chain_asic_num: usize,
        _asic_addr_interval: usize,
    ) -> Option<usize> {
        if !version_rolling {
            return None;
        }
        Some((((nonce >> 9) & 0xffff) as usize * chain_asic_num) >> 16)
    }

    /// ## Send Split Nonce Between Chips command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{Asic, CmdDelay};
    ///
    /// // 108 chips, address interval 2
    /// let mut bm1368 = BM1368::default();
    /// assert_eq!(bm1368.split_nonce_between_chips_next(108, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x0c, 0x80, 0x00, 0x00, 0x00, 0x1a], delay_ms: 0}));
    /// assert_eq!(bm1368.split_nonce_between_chips_next(108, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0x0c, 0x80, 0x00, 0x02, 0x5f, 0x13], delay_ms: 0}));
    /// assert_eq!(bm1368.split_nonce_between_chips_next(108, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x04, 0x0c, 0x80, 0x00, 0x04, 0xbe, 0x08], delay_ms: 0}));
    /// let mut steps = 3;
    /// while let Some(step) = bm1368.split_nonce_between_chips_next(108, 2) {
    ///     steps += 1;
    ///     if steps == 28 {
    ///         assert_eq!(step.cmd, [0x55, 0xaa, 0x41, 0x09, 0x36, 0x0c, 0x80, 0x00, 0x40, 0x00, 0x1d]); // chip 27: 0x8000_4000
    ///     }
    /// }
    /// assert_eq!(steps, 108);
    /// ```
    fn split_nonce_between_chips_next(
        &mut self,
        chain_asic_num: usize,
        asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::SplitNonce(step) if step > 0 && step < chain_asic_num => {
                self.seq_step = SequenceStep::SplitNonce(step + 1);
                let cno = ChipNonceOffsetV2::new(step, chain_asic_num).val();
                if step == 1 {
                    self.cno_interval = ChipNonceOffsetV2(cno).offset() as usize;
                }
                self.registers.insert(ChipNonceOffsetV2::ADDR, cno).unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        ChipNonceOffsetV2::ADDR,
                        cno,
                        Destination::Chip((step * asic_addr_interval) as u8),
                    ),
                    delay_ms: 0,
                })
            }
            SequenceStep::SplitNonce(step) if step == chain_asic_num => {
                self.seq_step = SequenceStep::None;
                None
            }
            _ => {
                // authorize a VersionRolling sequence start whatever the current step was
                self.seq_step = SequenceStep::SplitNonce(1);
                let cno = ChipNonceOffsetV2::new(0, chain_asic_num).val();
                self.registers.insert(ChipNonceOffsetV2::ADDR, cno).unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(ChipNonceOffsetV2::ADDR, cno, Destination::Chip(0)),
                    delay_ms: 0,
                })
            }
        }
    }

    /// ## Send Enable Version Rolling command list
    ///
    /// The Hash Counting Number is [`BM1368::hash_counting_number()`] at the current hash frequency, so
    /// set it first.
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1368 = BM1368::default();
    /// // real example from S21
    /// bm1368.set_hash_freq(HertzU64::MHz(490));
    /// assert_eq!(bm1368.set_version_rolling_next(0x1fff_e000, 108, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x10, 0x00, 0x00, 0x15, 0xa4, 0x0a], delay_ms: 1}));
    /// assert_eq!(bm1368.set_version_rolling_next(0x1fff_e000, 108, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa4, 0x90, 0x00, 0xff, 0xff, 0x1c], delay_ms: 1}));
    /// assert_eq!(bm1368.set_version_rolling_next(0x1fff_e000, 108, 2), None);
    /// ```
    fn set_version_rolling_next(
        &mut self,
        mask: u32,
        chain_asic_num: usize,
        _asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::VersionRolling(step) => match step {
                0 => {
                    self.seq_step = SequenceStep::VersionRolling(1);
                    let vers_roll = MidstateCalc(*self.registers.get(&MidstateCalc::ADDR).unwrap())
                        .enable()
                        .set_mask(mask)
                        .val();
                    self.registers
                        .insert(MidstateCalc::ADDR, vers_roll)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(MidstateCalc::ADDR, vers_roll, Destination::All),
                        delay_ms: 1,
                    })
                }
                1 => {
                    self.seq_step = SequenceStep::None;
                    None
                }
                _ => unreachable!(),
            },
            _ => {
                // authorize a VersionRolling sequence start whatever the current step was
                self.seq_step = SequenceStep::VersionRolling(0);
                let hcn = self.hash_counting_number(chain_asic_num);
                self.registers
                    .insert(HashCountingNumber::ADDR, hcn)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(HashCountingNumber::ADDR, hcn, Destination::All),
                    delay_ms: 1,
                })
            }
        }
    }

    /// ## Die temperature reading command list
    ///
    /// ### Example
    /// ```
    /// use bm1368::BM1368;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay, SensorStep};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1368 = BM1368::default();
    /// let dest = Destination::All;
    /// assert_eq!(bm1368.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xb0, 0x00, 0x02, 0x00, 0x00, 0x1f], delay_ms: 10})));
    /// assert_eq!(bm1368.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xb0, 0x01, 0x02, 0x00, 0x00, 0x16], delay_ms: 10})));
    /// assert_eq!(bm1368.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xb0, 0x10, 0x02, 0x00, 0x00, 0x1b], delay_ms: 10})));
    /// assert_eq!(bm1368.die_temp_next(dest), Some(SensorStep::Read{reg_addr: TempSensorData::ADDR, delay_ms: 0}));
    /// assert_eq!(bm1368.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xb0, 0x80, 0x00, 0x00, 0x00, 0x0f], delay_ms: 0})));
    /// assert_eq!(bm1368.die_temp_next(dest), None);
    /// assert_eq!(bm1368.die_temp_celsius(&[0x8000_0783]).unwrap() as i32, 31);
    /// assert_eq!(bm1368.die_temp_celsius(&[0x0000_0783]), None); // conversion not done
    /// ```
    fn die_temp_next(&mut self, dest: Destination) -> Option<SensorStep> {
        let step = match self.seq_step {
            SequenceStep::DieTemp(step) => step,
            _ => 0,
        };
        let ctrl = |reg: &mut TempSensorCtrl| reg.val();
        let (value, delay_ms) = match step {
            0 => (ctrl(TempSensorCtrl(0).set_ts_soft_rstn(true)), 10),
            1 => (
                ctrl(TempSensorCtrl(0).set_ts_soft_rstn(true).set_ts_cload(true)),
                10,
            ),
            2 => (
                ctrl(TempSensorCtrl(0).set_ts_soft_rstn(true).set_run_en(true)),
                10,
            ),
            3 => {
                self.seq_step = SequenceStep::DieTemp(4);
                return Some(SensorStep::Read {
                    reg_addr: TempSensorData::ADDR,
                    delay_ms: 0,
                });
            }
            4 => (ctrl(TempSensorCtrl(0).set_ts_pd(true)), 0),
            _ => {
                self.seq_step = SequenceStep::None;
                return None;
            }
        };
        self.seq_step = SequenceStep::DieTemp(step + 1);
        Some(SensorStep::Write(CmdDelay {
            cmd: Command::write_reg(TempSensorCtrl::ADDR, value, dest),
            delay_ms,
        }))
    }

    fn die_temp_celsius(&self, samples: &[u32]) -> Option<f32> {
        let data = TempSensorData(*samples.first()?);
        (data.ts_done() && !data.ts_error()).then(|| data.celsius())
    }
}
