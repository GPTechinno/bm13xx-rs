//! BM1372 ASIC implementation.
//!
//! Also drives the chips that identify as BM1373.

#![no_std]
pub(crate) mod fmt;

use bm13xx_asic::{
    adc::SupplyAdc, core_register::*, register::*, Asic, CmdDelay, SensorStep, SequenceStep,
};
use bm13xx_protocol::command::{Command, Destination};

use fugit::HertzU64;
use heapless::index_map::FnvIndexMap;

pub const BM1372_CHIP_ID: u16 = 0x1372;
pub const BM1373_CHIP_ID: u16 = 0x1373;
pub const BM1372_CORE_CNT: usize = 432;
pub const BM1372_SMALL_CORE_CNT: usize = 6904;
pub const BM1372_CORE_SMALL_CORE_CNT: usize = 16;
pub const BM1372_DOMAIN_CNT: usize = 8;
pub const BM1372_PLL_CNT: usize = 3;
pub const BM1372_PLL_ID_HASH: usize = 0; // PLL0 is used for Hashing
pub const BM1372_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 is used for Hashing
/// Largest interval between two chip addresses.
pub const BM1372_MAX_ADDR_INTERVAL: usize = 0x10;

/// Supply voltage readings through the ADC.
const SUPPLY_ADC: SupplyAdc = SupplyAdc {
    ctrl0_setup: 0x230C_000D,
    ctrl1: 0x3F01_4081,
    ctrl0_start: 0x232C_000D,
    scale_mv: 600.0,
};

/// # Hashboard settings
///
/// Settings that differ from one hashboard to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BoardConfig {
    /// `ccdly` of core register `ClockDelayCtrlV3`, written by `reset_core_next()`.
    pub clock_delay_ccdly: u8,
    /// `pwth` of core register `ClockDelayCtrlV3`, written by `reset_core_next()`.
    pub clock_delay_pwth: u8,
    /// Analog mux selection of `AnalogMuxControlV2`, written by `init_next()` when the board
    /// sets it.
    pub ana_mux_sel: Option<u8>,
    /// `CLKO` driver strength in `IoDriverStrenghtConfiguration` of the last chip of each voltage
    /// domain, written by `set_baudrate_next()`.
    pub domain_end_clko_strength: u8,
}

impl BoardConfig {
    /// Bitaxe NajaDuo: 2 chips, each in its own voltage domain.
    pub const NAJA_DUO: Self = Self {
        clock_delay_ccdly: 2,
        clock_delay_pwth: 6,
        ana_mux_sel: Some(2),
        domain_end_clko_strength: 15,
    };
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self::NAJA_DUO
    }
}

/// # BM1372
#[derive(Debug)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))] // FnvIndexMap doesn't implement defmt
pub struct BM1372 {
    seq_step: SequenceStep,
    pub sha: bm13xx_asic::sha::Sha<
        BM1372_CORE_CNT,
        BM1372_SMALL_CORE_CNT,
        BM1372_CORE_SMALL_CORE_CNT,
        BM1372_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU64,
    pub plls: [bm13xx_asic::pll::Pll; BM1372_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub core_registers: FnvIndexMap<u8, u8, 16>,
    cno_interval: usize,
    /// Settings of the hashboard the chip is on.
    pub board: BoardConfig,
}

impl BM1372 {
    pub fn new_with_clk(clk: HertzU64) -> Self {
        BM1372 {
            input_clock_freq: clk,
            ..Default::default()
        }
    }

    /// ## Create a BM1372 on a given hashboard
    ///
    /// ### Example
    /// ```
    /// use bm1372::{BoardConfig, BM1372};
    ///
    /// assert_eq!(BM1372::default().board, BoardConfig::NAJA_DUO);
    /// let bm1372 = BM1372::new_with_board(BoardConfig::NAJA_DUO);
    /// assert_eq!(bm1372.board, BoardConfig::NAJA_DUO);
    /// ```
    pub fn new_with_board(board: BoardConfig) -> Self {
        BM1372 {
            board,
            ..Default::default()
        }
    }

    /// ## Set the Chip Address
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    ///
    /// let mut bm1372 = BM1372::default();
    /// bm1372.set_chip_addr(0x10);
    /// assert_eq!(bm1372.chip_addr, 0x10);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8) {
        self.chip_addr = chip_addr;
    }

    pub fn set_hash_freq(&mut self, freq: HertzU64) -> &mut Self {
        self.plls[BM1372_PLL_ID_HASH].set_frequency(
            self.input_clock_freq,
            BM1372_PLL_OUT_HASH,
            freq,
            false,
        );
        self
    }

    /// ## Hash counting number for a chain of `chain_asic_num` chips
    ///
    /// 85 % of a core's half nonce range time: `⌊85 × ⌊⌊⌊((2 × interval) << 24) / 512⌋ × 25 / f⌋ / 2⌋ / 100⌋`,
    /// `f` the current hash frequency in MHz. The BM1372 counts its address interval over 7 address
    /// bits, `min(128 / chips, 3)`, so twice that in the 8-bit address space.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1372 = BM1372::default();
    /// bm1372.set_hash_freq(HertzU64::MHz(490));
    /// assert_eq!(bm1372.hash_counting_number(2), 4262);
    /// assert_eq!(bm1372.hash_counting_number(60), 2841);
    /// ```
    pub fn hash_counting_number(&self, chain_asic_num: usize) -> u32 {
        let addr_interval = (128 / chain_asic_num.max(1)).clamp(1, 3);
        bm13xx_asic::hash_counting::hash_counting_number(
            ((2 * addr_interval) as u64) << 24,
            BM1372_CORE_CNT,
            self.hash_freq(),
            100,
        ) * 85
            / 100
    }
}

impl Default for BM1372 {
    fn default() -> Self {
        let mut bm1372 = Self {
            seq_step: SequenceStep::default(),
            sha: bm13xx_asic::sha::Sha::default(),
            input_clock_freq: HertzU64::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1372_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            core_registers: FnvIndexMap::<_, _, 16>::new(),
            cno_interval: 0,
            board: BoardConfig::default(),
        };
        bm1372.reset();
        bm1372
    }
}

impl Asic for BM1372 {
    /// ## Register of the hash counter
    ///
    /// `GotNonceCounter`, the total of the eight hash-domain counters, one count per 2^32 hashes.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::Asic;
    ///
    /// assert_eq!(BM1372::default().hash_counter_reg(), Some(0x8C));
    /// ```
    fn hash_counter_reg(&self) -> Option<u8> {
        Some(GotNonceCounter::ADDR)
    }

    /// ## Registers of the hash-domain counters
    ///
    /// `Domain0GotNonceCounter` to `Domain7GotNonceCounter`, one per hash domain.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::Asic;
    ///
    /// assert_eq!(
    ///     BM1372::default().hash_domain_counter_regs(),
    ///     [0x88, 0x89, 0x8A, 0x8B, 0xAC, 0xAD, 0xAE, 0xAF]
    /// );
    /// ```
    fn hash_domain_counter_regs(&self) -> &'static [u8] {
        &[
            Domain0GotNonceCounter::ADDR,
            Domain1GotNonceCounter::ADDR,
            Domain2GotNonceCounter::ADDR,
            Domain3GotNonceCounter::ADDR,
            Domain4GotNonceCounter::ADDR,
            Domain5GotNonceCounter::ADDR,
            Domain6GotNonceCounter::ADDR,
            Domain7GotNonceCounter::ADDR,
        ]
    }

    /// ## Reset the Chip to default state
    fn reset(&mut self) {
        self.seq_step = SequenceStep::default();
        self.sha = bm13xx_asic::sha::Sha::default();
        self.input_clock_freq = HertzU64::MHz(25);
        self.plls = [bm13xx_asic::pll::Pll::default(); BM1372_PLL_CNT];
        self.chip_addr = 0;
        self.registers = FnvIndexMap::<_, _, 64>::new();
        self.core_registers = FnvIndexMap::<_, _, 16>::new();
        self.cno_interval = 0;
        // Default PLLs Parameter: PLL0 (0x08), PLL1 (0x60), PLL2 (0x61)
        self.plls[0].set_parameter(0xc054_0165);
        self.plls[1].set_parameter(0x2050_0174);
        self.plls[2].set_parameter(0x2050_0174);
        for pll in self.plls.iter_mut() {
            pll.set_divider(0x0000_0000);
        }
        // Default Registers Value
        // 0x40, 0x9C and 0xBD hold a value of their own on each chip.
        for (addr, value) in [
            (ChipIdentification::ADDR, 0x1372_0000),
            (Time1sCounter::ADDR, 0x0000_0000),
            (PLL0Parameter::ADDR, 0xc054_0165),
            (ChipNonceOffsetV2::ADDR, 0x0000_0000),
            (HashCountingNumber::ADDR, 0x0000_0000),
            (TicketMask::ADDR, 0x0000_0000),
            (MiscControlV3::ADDR, 0x0000_c100),
            (MiscControlAdd::ADDR, 0x0000_0000),
            (OrderedClockEnable::ADDR, 0x0000_000f),
            (TopProcessMonitor::ADDR, 0x0010_0000),
            (FastUARTConfigurationV3::ADDR, 0x0000_1a00),
            (FastUARTConfigurationAdd::ADDR, 0x0000_0000),
            (UARTRelay::ADDR, 0x000f_0000),
            (CoreNumber::ADDR, 0x0000_01b0),
            (TicketMask2::ADDR, 0x0000_0000),
            (CoreRegisterControl::ADDR, 0x0000_0000),
            (ErrorFlag::ADDR, 0x0000_0000),
            (NonceErrorCounter::ADDR, 0x0000_0000),
            (NonceOverflowCounter::ADDR, 0x0000_0000),
            (AnalogMuxControlV2::ADDR, 0x0000_0002),
            (IoDriverStrenghtConfiguration::ADDR, 0x0001_2111),
            (TimeOut::ADDR, 0x0000_ffff),
            (PLL1Parameter::ADDR, 0x2050_0174),
            (RoscPadDisable::ADDR, 0x0000_0000),
            (OrderedClockMonitor::ADDR, 0x0001_0200),
            (Domain0GotNonceCounter::ADDR, 0x0000_0000),
            (Domain1GotNonceCounter::ADDR, 0x0000_0000),
            (Domain2GotNonceCounter::ADDR, 0x0000_0000),
            (Domain3GotNonceCounter::ADDR, 0x0000_0000),
            (GotNonceCounter::ADDR, 0x0000_0000),
            (NtickNonceCounter::ADDR, 0x0000_0000),
            (TickNonceCounter::ADDR, 0x0000_0000),
            (MidstateCalc::ADDR, 0x0000_ffff),
            (SoftResetControlV2::ADDR, 0x0007_0000),
            (TempSensorCtrl0::ADDR, 0x8000_0000),
            (TempSensorData0::ADDR, 0x0000_0000),
            (AdcCtrl0::ADDR, 0x2000_0000),
            (AdcCtrl1::ADDR, 0x0000_4480),
            (AutoWorkCfg::ADDR, 0x0000_b850),
            (FrequencySweepControl::ADDR, 0x0000_0000),
        ] {
            self.registers.insert(addr, value).unwrap();
        }
        // Default Core Registers Value
        for (id, value) in [
            (ClockDelayCtrlV3::ID, 0x52),
            (CoreEnableV2::ID, 0x00),
            (HashClockCtrlV2::ID, 0x00),
            (HashClockCounterV2::ID, 0x00),
            (PgMont::ID, 0x08),
            (AutoClkEn::ID, 0x00),
            (ClockSelectCtrl::ID, 0x00),
            (NonceBinOverflowControl::ID, 0x05),
            (OcvPulseWidth::ID, 0x00),
        ] {
            self.core_registers.insert(id, value).unwrap();
        }
    }

    /// ## Get the Chip ID
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1372 = BM1372::default();
    /// assert_eq!(bm1372.chip_id(), 0x1372);
    /// assert!(bm1372.chip_id_matches(0x1372));
    /// assert!(bm1372.chip_id_matches(0x1373));
    /// assert!(!bm1372.chip_id_matches(0x1370));
    /// ```
    fn chip_id(&self) -> u16 {
        BM1372_CHIP_ID
    }

    fn chip_id_matches(&self, chip_id: u16) -> bool {
        chip_id == BM1372_CHIP_ID || chip_id == BM1373_CHIP_ID
    }

    /// ## Largest interval between two chip addresses
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::Asic;
    ///
    /// assert_eq!(BM1372::default().max_addr_interval(), 0x10);
    /// ```
    fn max_addr_interval(&self) -> usize {
        BM1372_MAX_ADDR_INTERVAL
    }

    /// ## Get the Chip Core count
    ///
    /// ### Example
    /// ```
    /// use bm1372::{BM1372, BM1372_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1372 = BM1372::default();
    /// assert_eq!(bm1372.core_count(), BM1372_CORE_CNT);
    /// ```
    fn core_count(&self) -> usize {
        self.sha.core_count()
    }

    /// ## Get the Chip Small Core count per Core
    ///
    /// ### Example
    /// ```
    /// use bm1372::{BM1372, BM1372_CORE_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1372 = BM1372::default();
    /// assert_eq!(bm1372.core_small_core_count(), BM1372_CORE_SMALL_CORE_CNT);
    /// ```
    fn core_small_core_count(&self) -> usize {
        self.sha.core_small_core_count()
    }

    /// ## Get the Chip Small Core count
    ///
    /// ### Example
    /// ```
    /// use bm1372::{BM1372, BM1372_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1372 = BM1372::default();
    /// assert_eq!(bm1372.small_core_count(), BM1372_SMALL_CORE_CNT);
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

    fn input_clock_freq(&self) -> HertzU64 {
        self.input_clock_freq
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::Asic;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1372 = BM1372::default();
    /// assert_eq!(bm1372.hash_freq(), HertzU64::MHz(50));
    /// assert_eq!(bm1372.set_hash_freq(HertzU64::MHz(490)).hash_freq(), HertzU64::MHz(490));
    /// ```
    fn hash_freq(&self) -> HertzU64 {
        self.plls[BM1372_PLL_ID_HASH].frequency(self.input_clock_freq, BM1372_PLL_OUT_HASH)
    }

    /// ## Init the Chip command list
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    ///
    /// let mut bm1372 = BM1372::default();
    /// assert_eq!(bm1372.init_next(2048), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x8d, 0x04, 0x04], delay_ms: 10}));
    /// assert_eq!(bm1372.init_next(2048), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0xe0, 0xff, 0x01], delay_ms: 10}));
    /// assert_eq!(bm1372.init_next(2048), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x54, 0x00, 0x00, 0x00, 0x02, 0x18], delay_ms: 0}));
    /// assert_eq!(bm1372.init_next(2048), None);
    /// assert_eq!(bm1372.core_registers.get(&NonceBinOverflowControl::ID).unwrap(), &0x04);
    /// assert_eq!(bm1372.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_e0ff);
    /// ```
    fn init_next(&mut self, difficulty: u32) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::Init(step) => match step {
                0 => {
                    self.seq_step = SequenceStep::Init(1);
                    let tck_mask = TicketMask::from_difficulty(difficulty).val();
                    self.registers.insert(TicketMask::ADDR, tck_mask).unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(TicketMask::ADDR, tck_mask, Destination::All),
                        delay_ms: 10,
                    })
                }
                1 => {
                    self.seq_step = SequenceStep::Init(2);
                    // only on the boards that set it
                    let Some(ana_mux_sel) = self.board.ana_mux_sel else {
                        self.seq_step = SequenceStep::None;
                        return None;
                    };
                    let ana_mux_ctrl =
                        AnalogMuxControlV2(*self.registers.get(&AnalogMuxControlV2::ADDR).unwrap())
                            .set_diode_vdd_mux_sel(ana_mux_sel)
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
                2 => {
                    self.seq_step = SequenceStep::None;
                    None
                }
                _ => unreachable!(),
            },
            _ => {
                // authorize an Init sequence start whatever the current step was
                self.seq_step = SequenceStep::Init(0);
                let nonce_bin_of_ctrl = NonceBinOverflowControl(
                    *self
                        .core_registers
                        .get(&NonceBinOverflowControl::ID)
                        .unwrap(),
                )
                .set_nonce_bin_of_enable(false)
                .set_golden_ticket_en(true)
                .val();
                self.core_registers
                    .insert(NonceBinOverflowControl::ID, nonce_bin_of_ctrl)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        CoreRegisterControl::ADDR,
                        CoreRegisterControl::write_core_reg(
                            0,
                            NonceBinOverflowControl(nonce_bin_of_ctrl),
                        ),
                        Destination::All,
                    ),
                    delay_ms: 10,
                })
            }
        }
    }

    /// ## Send Baudrate command list
    ///
    /// The UART clock is the 25 MHz input clock: baud rates up to 3.125 Mbaud.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    ///
    /// // 2 chips, each in its own voltage domain, address interval 0x10
    /// let mut bm1372 = BM1372::default();
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x00, 0x01, 0x11, 0x11, 0x0d], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x58, 0x00, 0x01, 0xf1, 0x11, 0x0f], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x58, 0x00, 0x01, 0xf1, 0x11, 0x0c], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0x5a, 0xa5, 0x5a, 0xa5, 0x1c], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x2c, 0x00, 0x0f, 0x00, 0x03, 0x08], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x2c, 0x00, 0x0f, 0x00, 0x03, 0x08], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x2c, 0x00, 0x10, 0x00, 0x03, 0x0e], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x00, 0x2c, 0x00, 0x10, 0x00, 0x03, 0x0e], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xc4, 0x00, 0x00, 0x00, 0x00, 0x1f], delay_ms: 0}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x80, 0x00, 0x00, 0x00, 0x18], delay_ms: 200}));
    /// assert_eq!(bm1372.set_baudrate_next(3_125_000, 2, 1, 0x10), None);
    /// assert_eq!(bm1372.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(), &0x0001_1111);
    /// assert_eq!(bm1372.registers.get(&RoscPadDisable::ADDR).unwrap(), &0x5aa5_5aa5);
    /// assert_eq!(bm1372.registers.get(&AutoWorkCfg::ADDR).unwrap(), &0x0000_0000);
    /// assert_eq!(bm1372.registers.get(&FastUARTConfigurationV3::ADDR).unwrap(), &0x8000_0000);
    /// // 1 Mbaud
    /// let mut last = None;
    /// while let Some(step) = bm1372.set_baudrate_next(1_000_000, 2, 1, 0x10) {
    ///     last = Some(step);
    /// }
    /// assert_eq!(last.unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x80, 0x00, 0x02, 0x00, 0x05]);
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
        let uart_relay = |dom: usize, registers: &FnvIndexMap<u8, u32, 64>| {
            UARTRelay(*registers.get(&UARTRelay::ADDR).unwrap())
                .set_gap_cnt(
                    (domain_asic_cnt as u16) * ((chain_domain_cnt as u16) - (dom as u16)) + 14,
                )
                .enable_ro_relay()
                .enable_co_relay()
                .val()
        };
        let last_chip_addr =
            |dom: usize| (((dom + 1) * domain_asic_cnt - 1) * asic_addr_interval) as u8;
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
                            Destination::Chip(last_chip_addr(dom)),
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
                    // do not save any chip-specific value
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            UARTRelay::ADDR,
                            uart_relay(dom, &self.registers),
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
                    // do not save any chip-specific value
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            UARTRelay::ADDR,
                            uart_relay(dom, &self.registers),
                            Destination::Chip(last_chip_addr(dom)),
                        ),
                        delay_ms: 0,
                    })
                } else if step == sub_seq5_start {
                    self.seq_step = SequenceStep::Baudrate(sub_seq6_start);
                    let auto_work_cfg = 0;
                    self.registers
                        .insert(AutoWorkCfg::ADDR, auto_work_cfg)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(AutoWorkCfg::ADDR, auto_work_cfg, Destination::All),
                        delay_ms: 0,
                    })
                } else if step == sub_seq6_start {
                    self.seq_step = SequenceStep::Baudrate(end);
                    let fbase = self.input_clock_freq.to_raw() as u32;
                    if baudrate > fbase / 8 {
                        warn!("{} baud is above {} baud", baudrate, fbase / 8);
                    }
                    let bt8d = (fbase / (8 * baudrate.max(1))).saturating_sub(1);
                    let fast_uart_cfg = FastUARTConfigurationV3(
                        *self.registers.get(&FastUARTConfigurationV3::ADDR).unwrap(),
                    )
                    .set_vco_clk_en(true)
                    .set_bclk_sel(0)
                    .set_bt8d(bt8d as u16)
                    .val();
                    self.registers
                        .insert(FastUARTConfigurationV3::ADDR, fast_uart_cfg)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            FastUARTConfigurationV3::ADDR,
                            fast_uart_cfg,
                            Destination::All,
                        ),
                        delay_ms: 200,
                    })
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
                .set_strenght(DriverSelect::CLKO, 1)
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
    /// The same sequence for one chip or for all chips. The reset bits of `SoftResetControlV2`
    /// clear themselves.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1372 = BM1372::default();
    /// let dest = Destination::Chip(0x10);
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0xa8, 0xf0, 0x07, 0x01, 0xf0, 0x0f], delay_ms: 20}));
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x18, 0xff, 0x00, 0xc1, 0x00, 0x17], delay_ms: 10}));
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x19, 0x00, 0x00, 0x00, 0x00, 0x0a], delay_ms: 10}));
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x3c, 0x80, 0x00, 0x8b, 0x00, 0x19], delay_ms: 10}));
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x3c, 0x80, 0x00, 0x80, 0x5a, 0x19], delay_ms: 10}));
    /// assert_eq!(bm1372.reset_core_next(dest), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0x3c, 0x80, 0x00, 0x82, 0xaa, 0x06], delay_ms: 10}));
    /// assert_eq!(bm1372.reset_core_next(dest), None);
    /// assert_eq!(bm1372.registers.get(&SoftResetControlV2::ADDR).unwrap(), &0x0007_0000);
    /// assert_eq!(bm1372.registers.get(&MiscControlV3::ADDR).unwrap(), &0xff00_c100);
    /// assert_eq!(bm1372.core_registers.get(&ClockDelayCtrlV3::ID).unwrap(), &0x5a);
    /// // all chips
    /// assert_eq!(bm1372.reset_core_next(Destination::All), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa8, 0xf0, 0x07, 0x01, 0xf0, 0x04], delay_ms: 20}));
    /// ```
    fn reset_core_next(&mut self, dest: Destination) -> Option<CmdDelay> {
        let step = match self.seq_step {
            SequenceStep::ResetCore(step) => step,
            // authorize a ResetCore sequence start whatever the current step was
            _ => 0,
        };
        let core_reg = |reg: u32| Command::write_reg(CoreRegisterControl::ADDR, reg, dest);
        let (cmd, delay_ms) = match step {
            0 => {
                // do not save the reset bits, they clear themselves
                let soft_rst_ctrl =
                    SoftResetControlV2(*self.registers.get(&SoftResetControlV2::ADDR).unwrap())
                        .set_core_srst_fast1(0xf)
                        .set_core_srst_fast(0xf)
                        .set_tver_reset(true)
                        .val();
                (
                    Command::write_reg(SoftResetControlV2::ADDR, soft_rst_ctrl, dest),
                    20,
                )
            }
            1 => {
                let misc = MiscControlV3(*self.registers.get(&MiscControlV3::ADDR).unwrap())
                    .set_rno_en(0xff)
                    .val();
                self.registers.insert(MiscControlV3::ADDR, misc).unwrap();
                (Command::write_reg(MiscControlV3::ADDR, misc, dest), 10)
            }
            2 => {
                let misc_add = MiscControlAdd(*self.registers.get(&MiscControlAdd::ADDR).unwrap())
                    .set_rtst_m_en(0)
                    .set_rtst_f_en(0)
                    .val();
                self.registers
                    .insert(MiscControlAdd::ADDR, misc_add)
                    .unwrap();
                (Command::write_reg(MiscControlAdd::ADDR, misc_add, dest), 10)
            }
            3 => {
                let clk_sel_ctrl = 0x00;
                self.core_registers
                    .insert(ClockSelectCtrl::ID, clk_sel_ctrl)
                    .unwrap();
                (
                    core_reg(CoreRegisterControl::write_core_reg(
                        0,
                        ClockSelectCtrl(clk_sel_ctrl),
                    )),
                    10,
                )
            }
            4 => {
                let clk_dly_ctrl =
                    ClockDelayCtrlV3(*self.core_registers.get(&ClockDelayCtrlV3::ID).unwrap())
                        .set_ccdly(self.board.clock_delay_ccdly)
                        .set_pwth(self.board.clock_delay_pwth)
                        .enable_hash_clock()
                        .disable_sweep_frequency_mode()
                        .val();
                self.core_registers
                    .insert(ClockDelayCtrlV3::ID, clk_dly_ctrl)
                    .unwrap();
                (
                    core_reg(CoreRegisterControl::write_core_reg(
                        0,
                        ClockDelayCtrlV3(clk_dly_ctrl),
                    )),
                    10,
                )
            }
            5 => {
                let core_en = 0xaa;
                self.core_registers
                    .insert(CoreEnableV2::ID, core_en)
                    .unwrap();
                (
                    core_reg(CoreRegisterControl::write_core_reg(
                        0,
                        CoreEnableV2(core_en),
                    )),
                    10,
                )
            }
            _ => {
                self.seq_step = SequenceStep::None;
                return None;
            }
        };
        self.seq_step = SequenceStep::ResetCore(step + 1);
        Some(CmdDelay { cmd, delay_ms })
    }

    /// ## Send Hash Frequency command list
    ///
    /// ### Example
    /// ```
    /// use bm1372::{BM1372, BM1372_PLL_ID_HASH};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1372 = BM1372::default();
    /// let mut steps = 0;
    /// let mut last = [0; 11];
    /// while let Some(step) = bm1372.set_hash_freq_next(HertzU64::MHz(490)) {
    ///     assert_eq!(step.cmd[5], PLL0Parameter::ADDR);
    ///     steps += 1;
    ///     last = step.cmd;
    /// }
    /// assert_eq!(steps, 71);
    /// assert_eq!(bm1372.hash_freq(), HertzU64::MHz(490));
    /// assert_eq!(last[6..10], bm1372.plls[BM1372_PLL_ID_HASH].parameter().to_be_bytes());
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
                let prev_pll0_param = self.plls[BM1372_PLL_ID_HASH].parameter();
                self.set_hash_freq(freq);
                let pll0_param = self.plls[BM1372_PLL_ID_HASH].parameter();
                // stop at the target, or if the PLL can't get closer to it
                if freq == target_freq || pll0_param == prev_pll0_param {
                    self.seq_step = SequenceStep::HashFreq(1);
                }
                self.registers
                    .insert(PLL0Parameter::ADDR, pll0_param)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(PLL0Parameter::ADDR, pll0_param, Destination::All),
                    delay_ms: 400,
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
    /// use bm1372::{BM1372, BM1372_PLL_ID_HASH};
    /// use bm13xx_asic::{register::PLL0Parameter, Asic};
    /// use bm13xx_protocol::command::Destination;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1372 = BM1372::default();
    /// let step = bm1372.set_chip_hash_freq_next(Destination::Chip(4), HertzU64::MHz(75)).unwrap();
    /// assert_eq!(step.cmd[2..6], [0x41, 0x09, 0x04, PLL0Parameter::ADDR]);
    /// let mut at_75_mhz = BM1372::default();
    /// at_75_mhz.set_hash_freq(HertzU64::MHz(75));
    /// assert_eq!(step.cmd[6..10], at_75_mhz.plls[BM1372_PLL_ID_HASH].parameter().to_be_bytes());
    /// assert_eq!(bm1372.set_chip_hash_freq_next(Destination::Chip(4), HertzU64::MHz(75)), None);
    /// assert_eq!(bm1372.hash_freq(), HertzU64::MHz(50));
    /// ```
    fn set_chip_hash_freq_next(&mut self, dest: Destination, freq: HertzU64) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::ChipHashFreq(_) => {
                self.seq_step = SequenceStep::None;
                None
            }
            _ => {
                self.seq_step = SequenceStep::ChipHashFreq(0);
                let mut pll = self.plls[BM1372_PLL_ID_HASH];
                pll.set_frequency(self.input_clock_freq, BM1372_PLL_OUT_HASH, freq, false);
                Some(CmdDelay {
                    cmd: Command::write_reg(PLL0Parameter::ADDR, pll.parameter(), dest),
                    delay_ms: 0,
                })
            }
        }
    }

    /// ## Send Split Nonce Between Chips command list
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    ///
    /// // 2 chips, address interval 0x10
    /// let mut bm1372 = BM1372::default();
    /// let first = bm1372.split_nonce_between_chips_next(2, 0x10).unwrap();
    /// assert_eq!(first.cmd[4..10], [0x00, ChipNonceOffsetV2::ADDR, 0x80, 0x00, 0x00, 0x00]);
    /// let second = bm1372.split_nonce_between_chips_next(2, 0x10).unwrap();
    /// assert_eq!(second.cmd[4..10], [0x10, ChipNonceOffsetV2::ADDR, 0x80, 0x00, 0x80, 0x00]);
    /// assert_eq!(bm1372.split_nonce_between_chips_next(2, 0x10), None);
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
                // authorize a SplitNonce sequence start whatever the current step was
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
    /// The Hash Counting Number is [`BM1372::hash_counting_number()`] at the current hash frequency, so
    /// set it first.
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1372 = BM1372::default();
    /// bm1372.set_hash_freq(HertzU64::MHz(490));
    /// let hcn = bm1372.set_version_rolling_next(0x1fff_e000, 2, 0x10).unwrap();
    /// assert_eq!(hcn.cmd[5..10], [HashCountingNumber::ADDR, 0x00, 0x00, 0x10, 0xa6]); // 4262
    /// assert_eq!(bm1372.set_version_rolling_next(0x1fff_e000, 2, 0x10), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa4, 0x90, 0x00, 0xff, 0xff, 0x1c], delay_ms: 1}));
    /// assert_eq!(bm1372.set_version_rolling_next(0x1fff_e000, 2, 0x10), None);
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

    /// ## Supply voltage reading command list
    ///
    /// One of the four sense channels (0 to 3) through the ADC: four set-up writes, then ten
    /// conversions 60 ms apart, each read in `ADC_VOLTAGE_DATAOUT` (`0xBD`).
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{adc::SUPPLY_CONVERSIONS, Asic, SensorStep};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1372 = BM1372::default();
    /// let dest = Destination::Chip(0);
    /// let mut reads = 0;
    /// while let Some(step) = bm1372.supply_voltage_next(dest, 1) {
    ///     if let SensorStep::Read { reg_addr, .. } = step {
    ///         assert_eq!(reg_addr, 0xBD);
    ///         reads += 1;
    ///     }
    /// }
    /// assert_eq!(reads, SUPPLY_CONVERSIONS);
    /// assert_eq!(bm1372.supply_voltage_next(dest, 4), None);
    /// let samples = [0x8000_6000u32; SUPPLY_CONVERSIONS];
    /// assert_eq!(bm1372.supply_voltage_mv(0, &samples), Some(300.0));
    /// ```
    fn supply_voltage_next(&mut self, dest: Destination, channel: u8) -> Option<SensorStep> {
        let step = match self.seq_step {
            SequenceStep::SupplyVoltage(step) => step,
            _ => 0,
        };
        let out = SUPPLY_ADC.step(step, dest, channel);
        self.seq_step = if out.is_some() {
            SequenceStep::SupplyVoltage(step + 1)
        } else {
            SequenceStep::None
        };
        out
    }

    fn supply_voltage_mv(&self, channel: u8, samples: &[u32]) -> Option<f32> {
        SUPPLY_ADC.millivolts(channel, samples)
    }

    /// ## Die temperature reading command list
    ///
    /// ### Example
    /// ```
    /// use bm1372::BM1372;
    /// use bm13xx_asic::{register::*, Asic, CmdDelay, SensorStep};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1372 = BM1372::default();
    /// let dest = Destination::Chip(0x10);
    /// assert_eq!(bm1372.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0xb2, 0x40, 0x02, 0x00, 0x00, 0x10], delay_ms: 10})));
    /// assert_eq!(bm1372.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0xb2, 0x60, 0x02, 0x00, 0x00, 0x18], delay_ms: 10})));
    /// assert_eq!(bm1372.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0xb2, 0x61, 0x02, 0x00, 0x00, 0x11], delay_ms: 10})));
    /// assert_eq!(bm1372.die_temp_next(dest), Some(SensorStep::Write(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x10, 0xb2, 0x70, 0x02, 0x00, 0x00, 0x1c], delay_ms: 10})));
    /// assert_eq!(bm1372.die_temp_next(dest), Some(SensorStep::Read{reg_addr: TempSensorData0::ADDR, delay_ms: 0}));
    /// assert_eq!(bm1372.die_temp_next(dest), None);
    /// assert_eq!(bm1372.die_temp_celsius(&[0x8000_075c]).unwrap() as i32, 25); // DOUT 1884
    /// assert_eq!(bm1372.die_temp_celsius(&[0x0000_075c]), None); // conversion not done
    /// ```
    fn die_temp_next(&mut self, dest: Destination) -> Option<SensorStep> {
        let step = match self.seq_step {
            SequenceStep::DieTemp(step) => step,
            _ => 0,
        };
        let ctrl = |reg: &mut TempSensorCtrl0| reg.set_ts_clk_en(true).set_ts_soft_rstn(true).val();
        let value = match step {
            0 => ctrl(&mut TempSensorCtrl0(0)),
            1 => ctrl(TempSensorCtrl0(0).set_run_ctrl(true)),
            2 => ctrl(TempSensorCtrl0(0).set_run_ctrl(true).set_ts_cload(true)),
            3 => ctrl(TempSensorCtrl0(0).set_run_ctrl(true).set_run_en(true)),
            4 => {
                self.seq_step = SequenceStep::DieTemp(5);
                return Some(SensorStep::Read {
                    reg_addr: TempSensorData0::ADDR,
                    delay_ms: 0,
                });
            }
            _ => {
                self.seq_step = SequenceStep::None;
                return None;
            }
        };
        self.seq_step = SequenceStep::DieTemp(step + 1);
        Some(SensorStep::Write(CmdDelay {
            cmd: Command::write_reg(TempSensorCtrl0::ADDR, value, dest),
            delay_ms: 10,
        }))
    }

    fn die_temp_celsius(&self, samples: &[u32]) -> Option<f32> {
        let data = TempSensorData0(*samples.first()?);
        (data.ts_done() && !data.ts_error()).then(|| data.celsius())
    }
}
