//! BM1362 ASIC implementation.

#![no_std]
pub(crate) mod fmt;

use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay, SequenceStep};
use bm13xx_protocol::command::{Command, Destination};

use fugit::HertzU64;
use heapless::index_map::FnvIndexMap;

pub const BM1362_CHIP_ID: u16 = 0x1362;
pub const BM1362_CORE_CNT: usize = 65;
pub const BM1362_SMALL_CORE_CNT: usize = 514; // cores 4 and 64 have 5 small cores, the others 8
pub const BM1362_CORE_SMALL_CORE_CNT: usize = 8;
pub const BM1362_DOMAIN_CNT: usize = 1;
pub const BM1362_PLL_CNT: usize = 2;
pub const BM1362_PLL_ID_HASH: usize = 0; // PLL0 is used for Hashing
pub const BM1362_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 is used for Hashing
pub const BM1362_PLL_ID_UART: usize = 1; // PLL1 can be used for UART Baudrate
pub const BM1362_PLL_OUT_UART: usize = 4; // specifically PLL1_OUT4 can be used for UART Baudrate

/// # BM1362
#[derive(Debug)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))] // FnvIndexMap doesn't implement defmt
pub struct BM1362 {
    seq_step: SequenceStep,
    pub sha: bm13xx_asic::sha::Sha<
        BM1362_CORE_CNT,
        BM1362_SMALL_CORE_CNT,
        BM1362_CORE_SMALL_CORE_CNT,
        BM1362_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU64,
    pub plls: [bm13xx_asic::pll::Pll; BM1362_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub core_registers: FnvIndexMap<u8, u8, 16>,
}

impl BM1362 {
    pub fn new_with_clk(clk: HertzU64) -> Self {
        BM1362 {
            input_clock_freq: clk,
            ..Default::default()
        }
    }

    /// ## Set the Chip Address
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    ///
    /// let mut bm1362 = BM1362::default();
    /// bm1362.set_chip_addr(2);
    /// assert_eq!(bm1362.chip_addr, 2);
    /// ```
    pub fn set_chip_addr(&mut self, chip_addr: u8) {
        self.chip_addr = chip_addr;
    }

    pub fn set_hash_freq(&mut self, freq: HertzU64) -> &mut Self {
        self.plls[BM1362_PLL_ID_HASH].set_frequency(
            self.input_clock_freq,
            BM1362_PLL_OUT_HASH,
            freq,
            false,
        );
        self
    }

    /// `MiscControl` with the core soft reset released (`0xB` in bits 31:28).
    fn misc_control_cores_released(&mut self) -> u32 {
        let misc = MiscControlV2(*self.registers.get(&MiscControlV2::ADDR).unwrap())
            .set_core_return_nonce(0xb)
            .set_b27_26(0)
            .set_b25_24(0)
            .set_b19_16(0)
            .val();
        self.registers.insert(MiscControlV2::ADDR, misc).unwrap();
        misc
    }

    /// Core register 5: hash clock enabled, from PLL0.
    fn hash_clock_ctrl(&mut self) -> u8 {
        let hash_clk_ctrl = HashClockCtrl(*self.core_registers.get(&HashClockCtrl::ID).unwrap())
            .enable()
            .set_pll_source(0)
            .val();
        self.core_registers
            .insert(HashClockCtrl::ID, hash_clk_ctrl)
            .unwrap();
        hash_clk_ctrl
    }

    /// Core register 0: `ccdly` 0, `pwth` 1, no frequency sweep.
    fn clock_delay_ctrl(&mut self) -> u8 {
        let clk_dly_ctrl =
            ClockDelayCtrlV2(*self.core_registers.get(&ClockDelayCtrlV2::ID).unwrap())
                .set_ccdly(0)
                .set_pwth(1)
                .disable_sweep_frequency_mode()
                .val();
        self.core_registers
            .insert(ClockDelayCtrlV2::ID, clk_dly_ctrl)
            .unwrap();
        clk_dly_ctrl
    }
}

impl Default for BM1362 {
    fn default() -> Self {
        let mut bm1362 = Self {
            seq_step: SequenceStep::default(),
            sha: bm13xx_asic::sha::Sha::default(),
            input_clock_freq: HertzU64::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1362_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            core_registers: FnvIndexMap::<_, _, 16>::new(),
        };
        bm1362.reset();
        bm1362
    }
}

impl Asic for BM1362 {
    /// ## Reset the Chip to default state
    fn reset(&mut self) {
        self.seq_step = SequenceStep::default();
        self.sha = bm13xx_asic::sha::Sha::default();
        self.input_clock_freq = HertzU64::MHz(25);
        self.plls = [bm13xx_asic::pll::Pll::default(); BM1362_PLL_CNT];
        self.chip_addr = 0;
        self.registers = FnvIndexMap::<_, _, 64>::new();
        self.core_registers = FnvIndexMap::<_, _, 16>::new();

        // Default PLLs Parameter
        self.plls[0].set_parameter(0xC054_0165);
        self.plls[1].set_parameter(0x2050_0174);
        // Default PLLs Divider
        self.plls[0].set_divider(0x0000_0000);
        self.plls[1].set_divider(0x0000_0000);
        // Default Registers Value
        self.registers
            .insert(ChipIdentification::ADDR, 0x1362_0300)
            .unwrap();
        self.registers
            .insert(PLL0Parameter::ADDR, 0xc054_0165)
            .unwrap();
        self.registers
            .insert(HashCountingNumber::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(TicketMask::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(MiscControlV2::ADDR, 0x0000_c100)
            .unwrap();
        self.registers
            .insert(FastUARTConfigurationV2::ADDR, 0x0130_1a00)
            .unwrap();
        self.registers
            .insert(CoreRegisterControl::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(AnalogMuxControlV2::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(IoDriverStrenghtConfiguration::ADDR, 0x0001_2111)
            .unwrap();
        self.registers
            .insert(PLL1Parameter::ADDR, 0x2050_0174)
            .unwrap();
        self.registers
            .insert(PLL0Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL1Divider::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(MidstateCalc::ADDR, 0x0000_ffff)
            .unwrap();
        self.registers
            .insert(SoftResetControl::ADDR, 0x0000_0000)
            .unwrap();
        // Default Core Registers Value
        self.core_registers
            .insert(ClockDelayCtrlV2::ID, 0x98)
            .unwrap();
        self.core_registers.insert(CoreEnableV2::ID, 0x55).unwrap();
        self.core_registers.insert(HashClockCtrl::ID, 0x40).unwrap();
    }

    /// ## Get the Chip ID
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1362 = BM1362::default();
    /// assert_eq!(bm1362.chip_id(), 0x1362);
    /// ```
    fn chip_id(&self) -> u16 {
        BM1362_CHIP_ID
    }

    /// ## Get the Chip Core count
    ///
    /// ### Example
    /// ```
    /// use bm1362::{BM1362, BM1362_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1362 = BM1362::default();
    /// assert_eq!(bm1362.core_count(), BM1362_CORE_CNT);
    /// ```
    fn core_count(&self) -> usize {
        self.sha.core_count()
    }

    /// ## Get the Chip Small Core count per Core
    ///
    /// ### Example
    /// ```
    /// use bm1362::{BM1362, BM1362_CORE_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1362 = BM1362::default();
    /// assert_eq!(bm1362.core_small_core_count(), BM1362_CORE_SMALL_CORE_CNT);
    /// ```
    fn core_small_core_count(&self) -> usize {
        self.sha.core_small_core_count()
    }

    /// ## Get the Chip Small Core count
    ///
    /// ### Example
    /// ```
    /// use bm1362::{BM1362, BM1362_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1362 = BM1362::default();
    /// assert_eq!(bm1362.small_core_count(), BM1362_SMALL_CORE_CNT);
    /// ```
    fn small_core_count(&self) -> usize {
        self.sha.small_core_count()
    }

    fn cno_interval(&self) -> usize {
        0
    }

    fn cno_bits(&self) -> u32 {
        ChipNonceOffsetV2::CNO_MASK.count_ones()
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::Asic;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.hash_freq(), HertzU64::MHz(50));
    /// assert_eq!(bm1362.set_hash_freq(HertzU64::MHz(525)).hash_freq(), HertzU64::MHz(525));
    /// ```
    fn hash_freq(&self) -> HertzU64 {
        self.plls[BM1362_PLL_ID_HASH].frequency(self.input_clock_freq, BM1362_PLL_OUT_HASH)
    }

    /// ## Init the Chip command list
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    ///
    /// // real example from S19j Pro
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x0c], delay_ms: 10}));
    /// assert_eq!(bm1362.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x08, 0x05], delay_ms: 10}));
    /// assert_eq!(bm1362.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xff, 0x08], delay_ms: 10}));
    /// assert_eq!(bm1362.init_next(256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x54, 0x00, 0x00, 0x00, 0x03, 0x1d], delay_ms: 0}));
    /// assert_eq!(bm1362.init_next(256), None);
    /// assert_eq!(bm1362.core_registers.get(&HashClockCtrl::ID).unwrap(), &0x40);
    /// assert_eq!(bm1362.core_registers.get(&ClockDelayCtrlV2::ID).unwrap(), &0x08);
    /// assert_eq!(bm1362.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00ff);
    /// assert_eq!(bm1362.registers.get(&AnalogMuxControlV2::ADDR).unwrap(), &0x0000_0003);
    /// ```
    fn init_next(&mut self, difficulty: u32) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::Init(step) => match step {
                0 => {
                    self.seq_step = SequenceStep::Init(1);
                    let clk_dly_ctrl = self.clock_delay_ctrl();
                    Some(CmdDelay {
                        cmd: Command::write_reg(
                            CoreRegisterControl::ADDR,
                            CoreRegisterControl::write_core_reg(0, ClockDelayCtrlV2(clk_dly_ctrl)),
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
                    let ana_mux_ctrl =
                        AnalogMuxControlV2(*self.registers.get(&AnalogMuxControlV2::ADDR).unwrap())
                            .set_diode_vdd_mux_sel(3)
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
            },
            _ => {
                // authorize an Init sequence start whatever the current step was
                self.seq_step = SequenceStep::Init(0);
                let hash_clk_ctrl = self.hash_clock_ctrl();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        CoreRegisterControl::ADDR,
                        CoreRegisterControl::write_core_reg(0, HashClockCtrl(hash_clk_ctrl)),
                        Destination::All,
                    ),
                    delay_ms: 10,
                })
            }
        }
    }

    /// ## Send Baudrate command list
    ///
    /// Up to 3.125 Mbaud the chips divide their 25 MHz input clock. Above, they divide PLL1 at
    /// 2400 MHz by 4 and by 6, down to 100 MHz, then by `2 * (BT8D + 1)`: 12.5 Mbaud with `BT8D` 3.
    /// The voltage domains need no `UARTRelay`.
    ///
    /// ### Example
    /// ```
    /// use bm1362::{BM1362, BM1362_PLL_ID_UART};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    ///
    /// // real example from S19j Pro
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.set_baudrate_next(3_000_000, 42, 3, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x00, 0x01, 0x11, 0x11, 0x0d], delay_ms: 10}));
    /// assert_eq!(bm1362.set_baudrate_next(3_000_000, 42, 3, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x11, 0x30, 0x00, 0x00, 0x1e], delay_ms: 200}));
    /// assert_eq!(bm1362.set_baudrate_next(3_000_000, 42, 3, 2), None);
    /// assert!(!bm1362.plls[BM1362_PLL_ID_UART].enabled());
    /// assert_eq!(bm1362.registers.get(&IoDriverStrenghtConfiguration::ADDR).unwrap(), &0x0001_1111);
    /// assert_eq!(bm1362.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(), &0x1130_0000);
    ///
    /// // 12.5 Mbaud from PLL1
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.set_baudrate_next(12_500_000, 42, 3, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x58, 0x00, 0x01, 0x11, 0x11, 0x0d], delay_ms: 10}));
    /// let pll1 = bm1362.set_baudrate_next(12_500_000, 42, 3, 2).unwrap();
    /// assert_eq!(pll1.cmd[5..10], [PLL1Parameter::ADDR, 0x50, 0x60, 0x01, 0x11]);
    /// assert_eq!(bm1362.set_baudrate_next(12_500_000, 42, 3, 2), Some(pll1));
    /// let fast_uart = bm1362.set_baudrate_next(12_500_000, 42, 3, 2).unwrap();
    /// assert_eq!(fast_uart.cmd[5..10], [FastUARTConfigurationV2::ADDR, 0x84, 0x50, 0x03, 0x00]);
    /// assert_eq!(fast_uart.delay_ms, 200);
    /// assert_eq!(bm1362.set_baudrate_next(12_500_000, 42, 3, 2), None);
    /// assert!(bm1362.plls[BM1362_PLL_ID_UART].enabled());
    /// assert_eq!(bm1362.registers.get(&PLL1Parameter::ADDR).unwrap(), &0x5060_0111);
    /// ```
    fn set_baudrate_next(
        &mut self,
        baudrate: u32,
        _chain_domain_cnt: usize,
        _domain_asic_cnt: usize,
        _asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        let from_clki = baudrate <= self.input_clock_freq.to_raw() as u32 / 8;
        match self.seq_step {
            SequenceStep::Baudrate(step) => match (from_clki, step) {
                (true, 0) => {
                    self.seq_step = SequenceStep::Baudrate(3);
                    let fbase = self.input_clock_freq.to_raw() as u32;
                    let bt8d = (fbase / (8 * baudrate)).saturating_sub(1);
                    let fast_uart_cfg = FastUARTConfigurationV2(
                        *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
                    )
                    .set_b28()
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
                }
                (false, 0) | (false, 1) => {
                    // PLL1 is written twice
                    self.seq_step = SequenceStep::Baudrate(step + 1);
                    self.plls[BM1362_PLL_ID_UART]
                        .unlock()
                        .enable()
                        .set_vco_high_freq()
                        .set_fb_div(96)
                        .set_ref_div(1)
                        .set_post1_div(1)
                        .set_post2_div(1)
                        .set_out_div(BM1362_PLL_OUT_UART, 5);
                    let pll1_param = self.plls[BM1362_PLL_ID_UART].parameter();
                    self.registers
                        .insert(PLL1Parameter::ADDR, pll1_param)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(PLL1Parameter::ADDR, pll1_param, Destination::All),
                        delay_ms: 10,
                    })
                }
                (false, 2) => {
                    self.seq_step = SequenceStep::Baudrate(3);
                    // PLL1 has locked since it was written
                    let fbase = self.plls[BM1362_PLL_ID_UART]
                        .lock()
                        .frequency(self.input_clock_freq, BM1362_PLL_OUT_UART)
                        .to_raw() as u32;
                    let bt8d = (fbase / (2 * baudrate)).saturating_sub(1);
                    let fast_uart_cfg = FastUARTConfigurationV2(
                        *self.registers.get(&FastUARTConfigurationV2::ADDR).unwrap(),
                    )
                    .set_b31()
                    .clr_b24()
                    .set_pll1_div4(self.plls[BM1362_PLL_ID_UART].out_div(BM1362_PLL_OUT_UART))
                    .set_bclk_sel(BaudrateClockSelectV2::Pll1)
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
                }
                (_, 3) => {
                    self.seq_step = SequenceStep::None;
                    None
                }
                _ => unreachable!("step={}", step),
            },
            _ => {
                // authorize a SetBaudrate sequence start whatever the current step was
                self.seq_step = SequenceStep::Baudrate(0);
                let io_drv_st_cfg = IoDriverStrenghtConfiguration(
                    *self
                        .registers
                        .get(&IoDriverStrenghtConfiguration::ADDR)
                        .unwrap(),
                )
                .set_strenght(DriverSelect::RF, 0)
                .disable(DriverRSelect::D3R)
                .disable(DriverRSelect::D2R)
                .disable(DriverRSelect::D1R)
                .disable(DriverRSelect::D0R)
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
                    delay_ms: 10,
                })
            }
        }
    }

    /// ## Reset the Chip Cores command list
    ///
    /// To all chips, before they get their address: clear the soft reset bits and release the core
    /// soft reset. To one chip: reset its cores, release them, then set core registers 5, 0 and 2.
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// // real example from S19j Pro
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.reset_core_next(Destination::All), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa8, 0x00, 0x00, 0x00, 0x00, 0x01], delay_ms: 1}));
    /// assert_eq!(bm1362.reset_core_next(Destination::All), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0xb0, 0x00, 0xc1, 0x00, 0x14], delay_ms: 200}));
    /// assert_eq!(bm1362.reset_core_next(Destination::All), None);
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0xa8, 0x00, 0x00, 0x00, 0x02, 0x1c], delay_ms: 10}));
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0x18, 0xb0, 0x00, 0xc1, 0x00, 0x03], delay_ms: 10}));
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0x3c, 0x80, 0x00, 0x85, 0x40, 0x1b], delay_ms: 10}));
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0x3c, 0x80, 0x00, 0x80, 0x08, 0x12], delay_ms: 10}));
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x41, 0x09, 0x02, 0x3c, 0x80, 0x00, 0x82, 0xaa, 0x1a], delay_ms: 10}));
    /// assert_eq!(bm1362.reset_core_next(Destination::Chip(2)), None);
    /// assert_eq!(bm1362.registers.get(&MiscControlV2::ADDR).unwrap(), &0xb000_c100);
    /// assert_eq!(bm1362.core_registers.get(&CoreEnableV2::ID).unwrap(), &0xaa);
    /// ```
    fn reset_core_next(&mut self, dest: Destination) -> Option<CmdDelay> {
        if dest == Destination::All {
            match self.seq_step {
                SequenceStep::ResetCore(step) => match step {
                    0 => {
                        self.seq_step = SequenceStep::ResetCore(1);
                        let misc = self.misc_control_cores_released();
                        Some(CmdDelay {
                            cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                            delay_ms: 200,
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
                    let soft_rst_ctrl =
                        SoftResetControl(*self.registers.get(&SoftResetControl::ADDR).unwrap())
                            .set_b3_0(0)
                            .val();
                    self.registers
                        .insert(SoftResetControl::ADDR, soft_rst_ctrl)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(SoftResetControl::ADDR, soft_rst_ctrl, dest),
                        delay_ms: 1,
                    })
                }
            }
        } else {
            match self.seq_step {
                SequenceStep::ResetCore(step) => match step {
                    0 => {
                        self.seq_step = SequenceStep::ResetCore(1);
                        let misc = self.misc_control_cores_released();
                        Some(CmdDelay {
                            cmd: Command::write_reg(MiscControlV2::ADDR, misc, dest),
                            delay_ms: 10,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::ResetCore(2);
                        let hash_clk_ctrl = self.hash_clock_ctrl();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    HashClockCtrl(hash_clk_ctrl),
                                ),
                                dest,
                            ),
                            delay_ms: 10,
                        })
                    }
                    2 => {
                        self.seq_step = SequenceStep::ResetCore(3);
                        let clk_dly_ctrl = self.clock_delay_ctrl();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    ClockDelayCtrlV2(clk_dly_ctrl),
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
                            .set_b3_0(2)
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
    /// Move the hash frequency by steps of 6.25 MHz until the target, 300 ms per step.
    ///
    /// ### Example
    /// ```
    /// use bm1362::{BM1362, BM1362_PLL_ID_HASH};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb4, 0x02, 0x74, 0x1d], delay_ms: 300}));
    /// assert_eq!(bm1362.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xaf, 0x02, 0x64, 0x0d], delay_ms: 300}));
    /// assert_eq!(bm1362.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xb0, 0x02, 0x73, 0x09], delay_ms: 300}));
    /// assert_eq!(bm1362.set_hash_freq_next(HertzU64::MHz(75)), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xa8, 0x02, 0x63, 0x14], delay_ms: 300}));
    /// assert_eq!(bm1362.set_hash_freq_next(HertzU64::MHz(75)), None);
    /// assert_eq!(bm1362.plls[BM1362_PLL_ID_HASH].parameter(), 0xc0a8_0263);
    /// // 76 steps from 56.25 MHz to 525 MHz, as on an S19j Pro
    /// let mut bm1362 = BM1362::default();
    /// let mut steps = 0;
    /// while bm1362.set_hash_freq_next(HertzU64::MHz(525)).is_some() {
    ///     steps += 1;
    /// }
    /// assert_eq!(steps, 76);
    /// assert_eq!(bm1362.hash_freq(), HertzU64::MHz(525));
    /// // and going down
    /// while bm1362.set_hash_freq_next(HertzU64::MHz(75)).is_some() {}
    /// assert_eq!(bm1362.hash_freq(), HertzU64::MHz(75));
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
                let prev_pll0_param = self.plls[BM1362_PLL_ID_HASH].parameter();
                self.set_hash_freq(freq);
                let pll0_param = self.plls[BM1362_PLL_ID_HASH].parameter();
                // stop at the target, or if the PLL can't get closer to it
                if freq == target_freq || pll0_param == prev_pll0_param {
                    self.seq_step = SequenceStep::HashFreq(1);
                }
                self.registers
                    .insert(PLL0Parameter::ADDR, pll0_param)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(PLL0Parameter::ADDR, pll0_param, Destination::All),
                    delay_ms: 300,
                })
            }
            _ => {
                // authorize a SetHashFreq sequence start whatever the current step was
                self.seq_step = SequenceStep::HashFreq(2);
                self.set_hash_freq_next(target_freq)
            }
        }
    }

    /// ## Send Split Nonce Between Chips command list
    ///
    /// Nothing to send: the chip address sits in nonce bits 24:17.
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::Asic;
    ///
    /// let mut bm1362 = BM1362::default();
    /// assert_eq!(bm1362.split_nonce_between_chips_next(126, 2), None);
    /// ```
    fn split_nonce_between_chips_next(
        &mut self,
        _chain_asic_num: usize,
        _asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        None
    }

    /// ## Send Enable Version Rolling command list
    ///
    /// The Hash Counting Number is computed from the current hash frequency, so set it first.
    ///
    /// ### Example
    /// ```
    /// use bm1362::BM1362;
    /// use bm13xx_asic::{register::HashCountingNumber, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// // real example from S19j Pro
    /// let mut bm1362 = BM1362::default();
    /// bm1362.set_hash_freq(HertzU64::MHz(525));
    /// assert_eq!(bm1362.set_version_rolling_next(0x1fff_e000, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x10, 0x00, 0x00, 0x13, 0x81, 0x08], delay_ms: 1}));
    /// assert_eq!(bm1362.set_version_rolling_next(0x1fff_e000, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0xa4, 0x90, 0x00, 0xff, 0xff, 0x1c], delay_ms: 1}));
    /// assert_eq!(bm1362.set_version_rolling_next(0x1fff_e000, 2), None);
    ///
    /// // real example from S19 (88 chips)
    /// let mut bm1362 = BM1362::default();
    /// bm1362.set_hash_freq(HertzU64::MHz(675));
    /// assert_eq!(bm1362.set_version_rolling_next(0x1fff_e000, 2), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x10, 0x00, 0x00, 0x0f, 0x2b, 0x07], delay_ms: 1}));
    /// ```
    fn set_version_rolling_next(
        &mut self,
        mask: u32,
        asic_addr_interval: usize,
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
                let hcn = bm13xx_asic::hash_counting::hash_counting_number(
                    asic_addr_interval,
                    BM1362_CORE_CNT,
                    self.hash_freq(),
                    80,
                );
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
}
