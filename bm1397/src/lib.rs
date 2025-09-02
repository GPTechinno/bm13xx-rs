//! BM1397 ASIC implementation.

#![no_std]
#![macro_use]
pub(crate) mod fmt;

use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay, SequenceStep};
use bm13xx_protocol::command::{Command, Destination};

use fugit::HertzU64;
use heapless::index_map::FnvIndexMap;

pub const BM1397_CHIP_ID: u16 = 0x1397;
pub const BM1397_CORE_CNT: usize = 168;
pub const BM1397_SMALL_CORE_CNT: usize = 672;
pub const BM1397_CORE_SMALL_CORE_CNT: usize = 4;
pub const BM1397_DOMAIN_CNT: usize = 4;
pub const BM1397_PLL_CNT: usize = 4;
pub const BM1397_PLL_ID_HASH: usize = 0; // PLL0 is used for Hashing
pub const BM1397_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 is used for Hashing
pub const BM1397_PLL_ID_UART: usize = 3; // PLL3 can be used for UART Baudrate
pub const BM1397_PLL_OUT_UART: usize = 4; // specifically PLL3_OUT4 can be used for UART Baudrate

/// # BM1397
#[derive(Debug)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))] // FnvIndexMap doesn't implement defmt
pub struct BM1397 {
    seq_step: SequenceStep,
    pub sha: bm13xx_asic::sha::Sha<
        BM1397_CORE_CNT,
        BM1397_SMALL_CORE_CNT,
        BM1397_CORE_SMALL_CORE_CNT,
        BM1397_DOMAIN_CNT,
    >,
    pub input_clock_freq: HertzU64,
    pub plls: [bm13xx_asic::pll::Pll; BM1397_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
    pub core_registers: FnvIndexMap<u8, u8, 16>,
}

impl BM1397 {
    pub fn new_with_clk(clk: HertzU64) -> Self {
        BM1397 {
            input_clock_freq: clk,
            ..Default::default()
        }
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

    pub fn set_hash_freq(&mut self, freq: HertzU64) -> &mut Self {
        self.plls[BM1397_PLL_ID_HASH].set_frequency(
            self.input_clock_freq,
            BM1397_PLL_OUT_HASH,
            freq,
            true,
        );
        self
    }

    /*
    const BM1397_NONCE_CORES_BITS: usize = 8; // Core ID is hardcoded on Nonce[31:24] -> 8 bits
    const BM1397_NONCE_CORES_MASK: u32 = 0b1111_1111;
    const BM1397_NONCE_SMALL_CORES_BITS: usize = 2; // Small Core ID is hardcoded on Nonce[23:22] -> 2 bits
    const BM1397_NONCE_SMALL_CORES_MASK: u32 = 0b11;

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
    */
}

impl Default for BM1397 {
    fn default() -> Self {
        let mut bm1397 = Self {
            seq_step: SequenceStep::default(),
            sha: bm13xx_asic::sha::Sha::default(),
            input_clock_freq: HertzU64::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
            core_registers: FnvIndexMap::<_, _, 16>::new(),
        };
        bm1397.reset();
        bm1397
    }
}

impl Asic for BM1397 {
    /// ## Reset the Chip to default state
    fn reset(&mut self) {
        self.seq_step = SequenceStep::default();
        self.sha = bm13xx_asic::sha::Sha::default();
        self.input_clock_freq = HertzU64::MHz(25);
        self.plls = [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT];
        self.chip_addr = 0;
        self.registers = FnvIndexMap::<_, _, 64>::new();
        self.core_registers = FnvIndexMap::<_, _, 16>::new();

        // Default PLLs Parameter
        self.plls[0].set_parameter(0xC060_0161);
        self.plls[1].set_parameter(0x0064_0111);
        self.plls[2].set_parameter(0x0068_0111);
        self.plls[3].set_parameter(0x0070_0111);
        // Default PLLs Divider
        self.plls[0].set_divider(0x0304_0607);
        self.plls[1].set_divider(0x0304_0506);
        self.plls[2].set_divider(0x0304_0506);
        self.plls[3].set_divider(0x0304_0506);
        // Default Registers Value
        self.registers
            .insert(ChipIdentification::ADDR, 0x1397_1800)
            .unwrap();
        self.registers.insert(HashRate::ADDR, 0x8000_0000).unwrap();
        self.registers
            .insert(PLL0Parameter::ADDR, 0xC060_0161)
            .unwrap();
        self.registers
            .insert(ChipNonceOffset::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(HashCountingNumber::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(TicketMask::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(MiscControl::ADDR, 0x0000_3A01)
            .unwrap();
        self.registers
            .insert(I2CControl::ADDR, 0x0100_0000)
            .unwrap();
        self.registers
            .insert(OrderedClockEnable::ADDR, 0x0000_FFFF)
            .unwrap();
        self.registers
            .insert(FastUARTConfiguration::ADDR, 0x0600_000F)
            .unwrap();
        self.registers.insert(UARTRelay::ADDR, 0x000F_0000).unwrap();
        self.registers
            .insert(TicketMask2::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(CoreRegisterControl::ADDR, 0x0000_4000)
            .unwrap();
        self.registers
            .insert(CoreRegisterValue::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(ExternalTemperatureSensorRead::ADDR, 0x0000_0100)
            .unwrap();
        self.registers.insert(ErrorFlag::ADDR, 0xFF00_0000).unwrap();
        self.registers
            .insert(NonceErrorCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(NonceOverflowCounter::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(AnalogMuxControl::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(IoDriverStrenghtConfiguration::ADDR, 0x0211_2111)
            .unwrap();
        self.registers.insert(TimeOut::ADDR, 0x0000_FFFF).unwrap();
        self.registers
            .insert(PLL1Parameter::ADDR, 0x0064_0111)
            .unwrap();
        self.registers
            .insert(PLL2Parameter::ADDR, 0x0068_0111)
            .unwrap();
        self.registers
            .insert(PLL3Parameter::ADDR, 0x0070_0111)
            .unwrap();
        self.registers
            .insert(OrderedClockMonitor::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(PLL0Divider::ADDR, 0x0304_0607)
            .unwrap();
        self.registers
            .insert(PLL1Divider::ADDR, 0x0304_0506)
            .unwrap();
        self.registers
            .insert(PLL2Divider::ADDR, 0x0304_0506)
            .unwrap();
        self.registers
            .insert(PLL3Divider::ADDR, 0x0304_0506)
            .unwrap();
        self.registers
            .insert(ClockOrderControl0::ADDR, 0xD95C_8410)
            .unwrap();
        self.registers
            .insert(ClockOrderControl1::ADDR, 0xFB73_EA62)
            .unwrap();
        self.registers
            .insert(ClockOrderStatus::ADDR, 0x0000_0000)
            .unwrap();
        self.registers
            .insert(FrequencySweepControl1::ADDR, 0x0000_0070)
            .unwrap();
        self.registers
            .insert(GoldenNonceForSweepReturn::ADDR, 0x0037_6400)
            .unwrap();
        self.registers
            .insert(ReturnedGroupPatternStatus::ADDR, 0x3030_3030)
            .unwrap();
        self.registers
            .insert(NonceReturnedTimeout::ADDR, 0x0000_FFFF)
            .unwrap();
        self.registers
            .insert(ReturnedSinglePatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        // Default Core Registers Value
        self.core_registers
            .insert(ClockDelayCtrl::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(ProcessMonitorCtrl::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(ProcessMonitorData::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(CoreError::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(CoreEnable::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(HashClockCtrl::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(HashClockCounter::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
        self.core_registers
            .insert(SweepClockCtrl::ID, 0x00) // TODO: add the correct value from chip actual reading
            .unwrap();
    }

    /// ## Get the Chip ID
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.chip_id(), 0x1397);
    /// ```
    fn chip_id(&self) -> u16 {
        BM1397_CHIP_ID
    }

    /// ## Get the Chip Core count
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.core_count(), BM1397_CORE_CNT);
    /// ```
    fn core_count(&self) -> usize {
        self.sha.core_count()
    }

    /// ## Get the Chip Small Core count per Core
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_CORE_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.core_small_core_count(), BM1397_CORE_SMALL_CORE_CNT);
    /// ```
    fn core_small_core_count(&self) -> usize {
        self.sha.core_small_core_count()
    }

    /// ## Get the Chip Small Core count
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_SMALL_CORE_CNT};
    /// use bm13xx_asic::Asic;
    ///
    /// let bm1397 = BM1397::default();
    /// assert_eq!(bm1397.small_core_count(), BM1397_SMALL_CORE_CNT);
    /// ```
    fn small_core_count(&self) -> usize {
        self.sha.small_core_count()
    }

    fn cno_interval(&self) -> usize {
        0
    }

    fn cno_bits(&self) -> u32 {
        ChipNonceOffset::CNO_MASK.count_ones()
    }

    /// ## Get the SHA Hashing Frequency
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::Asic;
    /// use fugit::HertzU64;
    ///
    /// let mut bm1397 = BM1397::default();
    /// assert_eq!(bm1397.hash_freq(), HertzU64::Hz(21428571));
    /// assert_eq!(bm1397.set_hash_freq(HertzU64::MHz(425)).hash_freq(), HertzU64::MHz(425));
    /// ```
    fn hash_freq(&self) -> HertzU64 {
        self.plls[BM1397_PLL_ID_HASH].frequency(self.input_clock_freq, BM1397_PLL_OUT_HASH)
    }

    /// ## Init the Chip command list
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    ///
    /// let mut bm1397 = BM1397::default();
    /// // Seen on T17
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x1c], delay_ms:0}));
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x84, 0x00, 0x00, 0x00, 0x00, 0x11], delay_ms:100}));
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x07], delay_ms:100}));
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x20, 0x00, 0x00, 0x00, 0xff, 0x13], delay_ms:10}));
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0xb4, 0x19], delay_ms:5}));
    /// assert_eq!(bm1397.init_next(64), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xfc, 0x07], delay_ms:10}));
    /// assert_eq!(bm1397.registers.get(&ClockOrderControl0::ADDR).unwrap(), &0x0000_0000);
    /// assert_eq!(bm1397.registers.get(&ClockOrderControl1::ADDR).unwrap(), &0x0000_0000);
    /// assert_eq!(bm1397.registers.get(&OrderedClockEnable::ADDR).unwrap(), &0x0000_00ff);
    /// assert_eq!(bm1397.core_registers.get(&ClockDelayCtrl::ID).unwrap(), &0xb4);
    /// assert_eq!(bm1397.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00fc);
    /// ```
    fn init_next(&mut self, difficulty: u32) -> Option<CmdDelay> {
        match self.seq_step {
            SequenceStep::Init(step) => {
                match step {
                    0 => {
                        self.seq_step = SequenceStep::Init(1);
                        let clk_ord_ctrl1 = ClockOrderControl1(
                            *self.registers.get(&ClockOrderControl1::ADDR).unwrap(),
                        )
                        .set_clock(ClockSelect::CLK8, 0)
                        .set_clock(ClockSelect::CLK9, 0)
                        .set_clock(ClockSelect::CLK10, 0)
                        .set_clock(ClockSelect::CLK11, 0)
                        .set_clock(ClockSelect::CLK12, 0)
                        .set_clock(ClockSelect::CLK13, 0)
                        .set_clock(ClockSelect::CLK14, 0)
                        .set_clock(ClockSelect::CLK15, 0)
                        .val();
                        self.registers
                            .insert(ClockOrderControl1::ADDR, clk_ord_ctrl1)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                ClockOrderControl1::ADDR,
                                clk_ord_ctrl1,
                                Destination::All,
                            ), // all CLK_SELx = 0b0000
                            delay_ms: 100,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::Init(2);
                        let clk_ord_en = OrderedClockEnable(
                            *self.registers.get(&OrderedClockEnable::ADDR).unwrap(),
                        )
                        .disable_all()
                        .val();
                        self.registers
                            .insert(OrderedClockEnable::ADDR, clk_ord_en)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                OrderedClockEnable::ADDR,
                                clk_ord_en,
                                Destination::All,
                            ),
                            delay_ms: 100,
                        })
                    }
                    2 => {
                        self.seq_step = SequenceStep::Init(3);
                        let clk_ord_en = OrderedClockEnable(
                            *self.registers.get(&OrderedClockEnable::ADDR).unwrap(),
                        )
                        .disable_all()
                        .enable(ClockSelect::CLK0)
                        .enable(ClockSelect::CLK1)
                        .enable(ClockSelect::CLK2)
                        .enable(ClockSelect::CLK3)
                        .enable(ClockSelect::CLK4)
                        .enable(ClockSelect::CLK5)
                        .enable(ClockSelect::CLK6)
                        .enable(ClockSelect::CLK7)
                        .val();
                        self.registers
                            .insert(OrderedClockEnable::ADDR, clk_ord_en)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                OrderedClockEnable::ADDR,
                                clk_ord_en,
                                Destination::All,
                            ),
                            delay_ms: 10,
                        })
                    }
                    3 => {
                        self.seq_step = SequenceStep::Init(4);
                        let clk_dly_ctrl =
                            ClockDelayCtrl(*self.core_registers.get(&ClockDelayCtrl::ID).unwrap())
                                .set_ccdly(2)
                                .set_pwth(3)
                                .enable_multi_midstate()
                                .val();
                        self.core_registers
                            .insert(ClockDelayCtrl::ID, clk_dly_ctrl)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                CoreRegisterControl::ADDR,
                                CoreRegisterControl::write_core_reg(
                                    0,
                                    ClockDelayCtrl(clk_dly_ctrl),
                                ),
                                Destination::All,
                            ),
                            delay_ms: 5,
                        })
                    }
                    4 => {
                        self.seq_step = SequenceStep::Init(5);
                        let tck_mask = TicketMask::from_difficulty(difficulty).val();
                        self.registers.insert(TicketMask::ADDR, tck_mask).unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(TicketMask::ADDR, tck_mask, Destination::All),
                            delay_ms: 10,
                        })
                    }
                    5 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                // authorize a Init sequence start whatever the current step was
                self.seq_step = SequenceStep::Init(0);
                let clk_ord_ctrl0 =
                    ClockOrderControl0(*self.registers.get(&ClockOrderControl0::ADDR).unwrap())
                        .set_clock(ClockSelect::CLK0, 0)
                        .set_clock(ClockSelect::CLK1, 0)
                        .set_clock(ClockSelect::CLK2, 0)
                        .set_clock(ClockSelect::CLK3, 0)
                        .set_clock(ClockSelect::CLK4, 0)
                        .set_clock(ClockSelect::CLK5, 0)
                        .set_clock(ClockSelect::CLK6, 0)
                        .set_clock(ClockSelect::CLK7, 0)
                        .set_clock(ClockSelect::CLK8, 0)
                        .val();
                self.registers
                    .insert(ClockOrderControl0::ADDR, clk_ord_ctrl0)
                    .unwrap();
                Some(CmdDelay {
                    cmd: Command::write_reg(
                        ClockOrderControl0::ADDR,
                        clk_ord_ctrl0,
                        Destination::All,
                    ), // all CLK_SELx = 0b0000
                    delay_ms: 0,
                })
            }
        }
    }

    /// ## Set Baudrate command list
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_PLL_ID_UART};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    ///
    /// let mut bm1397 = BM1397::default();
    // // Seen on T17
    /// assert_eq!(bm1397.set_baudrate_next(3_125_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x00, 0x20, 0x01, 0x0d], delay_ms:200}));
    /// assert_eq!(bm1397.set_baudrate_next(3_125_000, 1, 1, 256), None);
    // assert!(!bm1397.plls[BM1397_PLL_ID_UART].enabled());
    /// assert_eq!(bm1397.registers.get(&MiscControl::ADDR).unwrap(), &0x0000_2001);
    /// assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0xc0, 0x70, 0x01, 0x11, 0x00], delay_ms:0}));
    /// assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0xc0, 0x70, 0x01, 0x11, 0x00], delay_ms:1}));
    /// assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x06, 0x00, 0x00, 0x0f, 0x18], delay_ms:0}));
    // assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x01, 0x67, 0x31, 0x06], delay_ms:200})); // real value equivalent
    /// assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x01, 0x27, 0x01, 11], delay_ms:200}));
    /// assert_eq!(bm1397.set_baudrate_next(6_250_000, 1, 1, 256), None);
    /// assert!(bm1397.plls[BM1397_PLL_ID_UART].enabled());
    /// assert_eq!(bm1397.registers.get(&PLL3Parameter::ADDR).unwrap(), &0xC070_0111);
    /// assert_eq!(bm1397.registers.get(&FastUARTConfiguration::ADDR).unwrap(), &0x0600_000F);
    /// assert_eq!(bm1397.registers.get(&MiscControl::ADDR).unwrap(), &0x0001_2701);
    /// assert_eq!(bm1397.set_baudrate_next(115_740, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x00, 0x3a, 0x01, 0x18], delay_ms:200}));
    // assert_eq!(bm1397.set_baudrate_next(115_740, 1, 1, 256), Some(CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0x00, 0x70, 0x01, 0x11, 21], delay_ms:0}));
    /// assert_eq!(bm1397.set_baudrate_next(115_740, 1, 1, 256), None);
    // assert!(!bm1397.plls[BM1397_PLL_ID_UART].enabled());
    /// assert_eq!(bm1397.registers.get(&MiscControl::ADDR).unwrap(), &0x0000_3A01);
    // assert_eq!(bm1397.registers.get(&PLL3Parameter::ADDR).unwrap(), &0x0070_0111);
    /// ```
    fn set_baudrate_next(
        &mut self,
        baudrate: u32,
        _chain_domain_cnt: usize,
        _domain_asic_cnt: usize,
        _asic_addr_interval: usize,
    ) -> Option<CmdDelay> {
        if baudrate <= self.input_clock_freq.raw() as u32 / 8 {
            let fbase = self.input_clock_freq.raw() as u32;
            let bt8d = (fbase / (8 * baudrate)) - 1;
            match self.seq_step {
                SequenceStep::Baudrate(step) => match step {
                    0 => {
                        // should not be reached for 2 reasons:
                        // - in previous step we jump directly to end
                        // - after setting the chip's FastUartConfiguration with bclk_sel(BaudrateClockSelect::Clki) in previous step
                        //   the chip's baudrate should immediatly adapt and thus this new step with old baudrate from control side
                        //   will be ignored by the chip.
                        self.seq_step = SequenceStep::Baudrate(1);
                        let pll3_param =
                            self.plls[BM1397_PLL_ID_UART].disable().unlock().parameter();
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
                    1 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => {
                    // authorize a SetBaudrate sequence start whatever the current step was
                    self.seq_step = SequenceStep::Baudrate(1);
                    let misc_ctrl = MiscControl(*self.registers.get(&MiscControl::ADDR).unwrap())
                        .set_bclk_sel(BaudrateClockSelect::Clki)
                        .set_bt8d(bt8d as u16)
                        .val();
                    self.registers.insert(MiscControl::ADDR, misc_ctrl).unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(MiscControl::ADDR, misc_ctrl, Destination::All),
                        delay_ms: 200,
                    })
                }
            }
        } else {
            let pll3_div4 = 6;
            self.plls[BM1397_PLL_ID_UART]
                // .set_parameter(0xC070_0111)
                .lock()
                .enable()
                .set_fb_div(112)
                .set_ref_div(1)
                .set_post1_div(1)
                .set_post2_div(1)
                .set_out_div(BM1397_PLL_OUT_UART, pll3_div4);
            let fbase = self.plls[BM1397_PLL_ID_UART]
                .frequency(self.input_clock_freq, BM1397_PLL_OUT_UART)
                .raw();
            match self.seq_step {
                SequenceStep::Baudrate(step) => match step {
                    0 => {
                        self.seq_step = SequenceStep::Baudrate(1);
                        let pll3_param = self.plls[BM1397_PLL_ID_UART].parameter();
                        self.registers
                            .insert(PLL3Parameter::ADDR, pll3_param)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                PLL3Parameter::ADDR,
                                pll3_param,
                                Destination::All,
                            ),
                            delay_ms: 1,
                        })
                    }
                    1 => {
                        self.seq_step = SequenceStep::Baudrate(2);
                        let fast_uart_cfg = FastUARTConfiguration(
                            *self.registers.get(&FastUARTConfiguration::ADDR).unwrap(),
                        )
                        .set_pll3_div4(pll3_div4)
                        .val();
                        self.registers
                            .insert(FastUARTConfiguration::ADDR, fast_uart_cfg)
                            .unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(
                                FastUARTConfiguration::ADDR,
                                fast_uart_cfg,
                                Destination::All,
                            ),
                            delay_ms: 0,
                        })
                    }
                    2 => {
                        self.seq_step = SequenceStep::Baudrate(3);
                        let bt8d = (fbase as u32 / (2 * baudrate)) - 1;
                        let misc_ctrl =
                            MiscControl(*self.registers.get(&MiscControl::ADDR).unwrap())
                                .set_bclk_sel(BaudrateClockSelect::Pll3)
                                .set_bt8d(bt8d as u16)
                                .val();
                        self.registers.insert(MiscControl::ADDR, misc_ctrl).unwrap();
                        Some(CmdDelay {
                            cmd: Command::write_reg(MiscControl::ADDR, misc_ctrl, Destination::All),
                            delay_ms: 200,
                        })
                    }
                    3 => {
                        self.seq_step = SequenceStep::None;
                        None
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => {
                    // authorize a SetBaudrate sequence start whatever the current step was
                    self.seq_step = SequenceStep::Baudrate(0);
                    let pll3_param = self.plls[BM1397_PLL_ID_UART].parameter();
                    self.registers
                        .insert(PLL3Parameter::ADDR, pll3_param)
                        .unwrap();
                    Some(CmdDelay {
                        cmd: Command::write_reg(PLL3Parameter::ADDR, pll3_param, Destination::All),
                        delay_ms: 0,
                    })
                }
            }
        }
    }

    /// ## Reset the Chip Cores command list
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::{core_register::*, register::*, Asic, CmdDelay};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let mut bm1397 = BM1397::default();
    /// assert_eq!(bm1397.reset_core_next(Destination::All), None);
    /// ```
    fn reset_core_next(&mut self, _dest: Destination) -> Option<CmdDelay> {
        None // TODO impl
    }

    /// ## Send Hash Frequency command list
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_PLL_ID_HASH};
    /// use bm13xx_asic::{register::*, Asic, CmdDelay};
    /// use fugit::HertzU64;
    ///
    /// let mut bm1397 = BM1397::default();
    // assert_eq!(bm1397.set_hash_freq_next(HertzU64::MHz(700)), CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x70, 0x0f, 0x0f, 0x0f, 0x00, 25], delay_ms: 0});
    // assert_eq!(bm1397.set_hash_freq_next(HertzU64::MHz(700)), CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xad, 0x02, 0x77, 13], delay_ms: 1});
    // assert_eq!(bm1397.set_hash_freq_next(HertzU64::MHz(700)), CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x70, 0x0f, 0x0f, 0x0f, 0x00, 25], delay_ms: 0});
    // assert_eq!(bm1397.set_hash_freq_next(HertzU64::MHz(700)), CmdDelay{cmd: [0x55, 0xaa, 0x51, 0x09, 0x00, 0x08, 0xc0, 0xad, 0x02, 0x76, 8], delay_ms: 1});
    /// assert_eq!(bm1397.set_hash_freq_next(HertzU64::MHz(700)), None);
    // assert_eq!(bm1397.plls[BM1397_PLL_ID_HASH].parameter(), 0xc070_0111);
    /// ```
    fn set_hash_freq_next(&mut self, _target_freq: HertzU64) -> Option<CmdDelay> {
        /*
        self.plls[BM1397_PLL_ID_HASH].set_divider(0x0f0f_0f00);
        self.registers
            .insert(PLL0Divider::ADDR, self.plls[BM1397_PLL_ID_HASH].divider())
            .unwrap();
        self.plls[BM1397_PLL_ID_HASH]
            .set_fb_div(173)
            .set_ref_div(2)
            .set_post1_div(7);
        let mut prev_post_div: u8;
        let mut found = false;
        for post2_div in (1..=7).rev() {
            prev_post_div = self.plls[BM1397_PLL_ID_HASH].post2_div();
            self.plls[BM1397_PLL_ID_HASH].set_post2_div(post2_div);
            if self.hash_freq() < target_freq {
                Some(CmdDelay {
                        cmd: Command::write_reg(
                            PLL0Divider::ADDR,
                            self.plls[BM1397_PLL_ID_HASH].divider(),
                            Destination::All,
                        ),
                        delay_ms: 1,
                    })
                Some(CmdDelay {
                        cmd: Command::write_reg(
                            PLL0Parameter::ADDR,
                            self.plls[BM1397_PLL_ID_HASH].parameter(),
                            Destination::All,
                        ),
                        delay_ms: if post2_div == 1 {
                            12000
                        } else if post2_div == 2 {
                            6000
                        } else {
                            1000
                        },
                    })
                self.registers
                    .insert(
                        PLL0Parameter::ADDR,
                        self.plls[BM1397_PLL_ID_HASH].parameter(),
                    )
                    .unwrap();
            } else {
                self.plls[BM1397_PLL_ID_HASH].set_post2_div(prev_post_div);
                found = true;
                break;
            }
        }
        if !found {
            for post1_div in (1..=6).rev() {
                prev_post_div = self.plls[BM1397_PLL_ID_HASH].post1_div();
                self.plls[BM1397_PLL_ID_HASH].set_post2_div(post1_div);
                if self.hash_freq() < target_freq {
                    Some(CmdDelay {
                            cmd: Command::write_reg(
                                PLL0Divider::ADDR,
                                self.plls[BM1397_PLL_ID_HASH].divider(),
                                Destination::All,
                            ),
                            delay_ms: 1,
                        })
                    Some(CmdDelay {
                            cmd: Command::write_reg(
                                PLL0Parameter::ADDR,
                                self.plls[BM1397_PLL_ID_HASH].parameter(),
                                Destination::All,
                            ),
                            delay_ms: if post1_div > 4 { 6000 } else { 9000 },
                        })
                    self.registers
                        .insert(
                            PLL0Parameter::ADDR,
                            self.plls[BM1397_PLL_ID_HASH].parameter(),
                        )
                        .unwrap();
                } else {
                    self.plls[BM1397_PLL_ID_HASH].set_post1_div(prev_post_div);
                    break;
                }
            }
        }
        // self.set_hash_freq(target_freq); // TODO: finish with the closest value
         */
        None // TODO: impl
    }

    /// ## Send Split Nonce Between Chips command list
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::Asic;
    ///
    /// let mut bm1397 = BM1397::default();
    /// assert_eq!(bm1397.split_nonce_between_chips_next(30, 8), None);
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
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::Asic;
    ///
    /// let mut bm1397 = BM1397::default();
    /// assert_eq!(bm1397.set_version_rolling_next(0x1fff_e000), None);
    /// ```
    fn set_version_rolling_next(&mut self, _mask: u32) -> Option<CmdDelay> {
        None
    }
}
