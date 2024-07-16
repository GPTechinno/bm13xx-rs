#![no_std]
//! BM1397 ASIC implementation.

use bm13xx_asic::{core_register::ClockDelayCtrl, register::*};
use bm13xx_protocol::{
    command::{Command, Destination},
    Bm13xxProtocol, CmdDelay,
};

use core::time::Duration;
use fugit::HertzU32;
use heapless::{FnvIndexMap, Vec};

pub const BM1397_CORE_CNT: usize = 168;
pub const BM1397_SMALL_CORE_CNT: usize = 672;
pub const BM1397_CORE_SMALL_CORE_CNT: usize = 4;
pub const BM1397_DOMAIN_CNT: usize = 4;
pub const BM1397_PLL_CNT: usize = 4;
pub const BM1397_PLL_ID_HASH: usize = 0; // PLL0 isused for Hashing
pub const BM1397_PLL_OUT_HASH: usize = 0; // specifically PLL0_OUT0 can be used for Hashing
pub const BM1397_PLL_ID_UART: usize = 3; // PLL3 can be used for UART Baudrate
pub const BM1397_PLL_OUT_UART: usize = 4; // specifically PLL3_OUT4 can be used for UART Baudrate
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
    pub input_clock_freq: HertzU32,
    pub plls: [bm13xx_asic::pll::Pll; BM1397_PLL_CNT],
    pub chip_addr: u8,
    pub registers: FnvIndexMap<u8, u32, 64>,
}

impl BM1397 {
    pub fn new_with_clk(clk: HertzU32) -> Self {
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
        self.plls[BM1397_PLL_ID_HASH].frequency(self.input_clock_freq, BM1397_PLL_OUT_HASH)
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
            input_clock_freq: HertzU32::MHz(25),
            plls: [bm13xx_asic::pll::Pll::default(); BM1397_PLL_CNT],
            chip_addr: 0,
            registers: FnvIndexMap::<_, _, 64>::new(),
        };
        // Default PLLs Parameter
        bm1397.plls[0].set_parameter(0xC060_0161);
        bm1397.plls[1].set_parameter(0x0064_0111);
        bm1397.plls[2].set_parameter(0x0068_0111);
        bm1397.plls[3].set_parameter(0x0070_0111);
        // Default PLLs Divider
        bm1397.plls[0].set_divider(0x0304_0607);
        bm1397.plls[1].set_divider(0x0304_0506);
        bm1397.plls[2].set_divider(0x0304_0506);
        bm1397.plls[3].set_divider(0x0304_0506);
        // Default Registers Value
        bm1397
            .registers
            .insert(ChipIdentification::ADDR, 0x1397_1800)
            .unwrap();
        bm1397
            .registers
            .insert(HashRate::ADDR, 0x8000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(PLL0Parameter::ADDR, 0xC060_0161)
            .unwrap();
        bm1397
            .registers
            .insert(ChipNonceOffset::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(HashCountingNumber::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(TicketMask::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(MiscControl::ADDR, 0x0000_3A01)
            .unwrap();
        bm1397
            .registers
            .insert(I2CControl::ADDR, 0x0100_0000)
            .unwrap();
        bm1397
            .registers
            .insert(OrderedClockEnable::ADDR, 0x0000_FFFF)
            .unwrap();
        bm1397
            .registers
            .insert(FastUARTConfiguration::ADDR, 0x0600_000F)
            .unwrap();
        bm1397
            .registers
            .insert(UARTRelay::ADDR, 0x000F_0000)
            .unwrap();
        bm1397
            .registers
            .insert(TicketMask2::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(CoreRegisterControl::ADDR, 0x0000_4000)
            .unwrap();
        bm1397
            .registers
            .insert(CoreRegisterValue::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(ExternalTemperatureSensorRead::ADDR, 0x0000_0100)
            .unwrap();
        bm1397
            .registers
            .insert(ErrorFlag::ADDR, 0xFF00_0000)
            .unwrap();
        bm1397
            .registers
            .insert(NonceErrorCounter::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(NonceOverflowCounter::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(AnalogMuxControl::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(IoDriverStrenghtConfiguration::ADDR, 0x0211_2111)
            .unwrap();
        bm1397.registers.insert(TimeOut::ADDR, 0x0000_FFFF).unwrap();
        bm1397
            .registers
            .insert(PLL1Parameter::ADDR, 0x0064_0111)
            .unwrap();
        bm1397
            .registers
            .insert(PLL2Parameter::ADDR, 0x0068_0111)
            .unwrap();
        bm1397
            .registers
            .insert(PLL3Parameter::ADDR, 0x0070_0111)
            .unwrap();
        bm1397
            .registers
            .insert(OrderedClockMonitor::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(PLL0Divider::ADDR, 0x0304_0607)
            .unwrap();
        bm1397
            .registers
            .insert(PLL1Divider::ADDR, 0x0304_0506)
            .unwrap();
        bm1397
            .registers
            .insert(PLL2Divider::ADDR, 0x0304_0506)
            .unwrap();
        bm1397
            .registers
            .insert(PLL3Divider::ADDR, 0x0304_0506)
            .unwrap();
        bm1397
            .registers
            .insert(ClockOrderControl0::ADDR, 0xD95C_8410)
            .unwrap();
        bm1397
            .registers
            .insert(ClockOrderControl1::ADDR, 0xFB73_EA62)
            .unwrap();
        bm1397
            .registers
            .insert(ClockOrderStatus::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
            .registers
            .insert(FrequencySweepControl1::ADDR, 0x0000_0070)
            .unwrap();
        bm1397
            .registers
            .insert(GoldenNonceForSweepReturn::ADDR, 0x0037_6400)
            .unwrap();
        bm1397
            .registers
            .insert(ReturnedGroupPatternStatus::ADDR, 0x3030_3030)
            .unwrap();
        bm1397
            .registers
            .insert(NonceReturnedTimeout::ADDR, 0x0000_FFFF)
            .unwrap();
        bm1397
            .registers
            .insert(ReturnedSinglePatternStatus::ADDR, 0x0000_0000)
            .unwrap();
        bm1397
    }
}

impl Bm13xxProtocol for BM1397 {
    /// ## Init the Chip
    ///
    /// ### Example
    /// ```
    /// use bm1397::BM1397;
    /// use bm13xx_asic::register::*;
    /// use bm13xx_protocol::Bm13xxProtocol;
    ///
    /// let mut bm1397 = BM1397::default();
    /// let mut init_seq = bm1397.init(256);
    /// assert_eq!(init_seq.len(), 5);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x14, 0x00, 0x00, 0x00, 0xff, 0x08]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x3c, 0x80, 0x00, 0x80, 0x74, 0x10]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x20, 0x00, 0x00, 0x00, 0x01, 0x02]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x84, 0x00, 0x00, 0x00, 0x00, 0x11]);
    /// assert_eq!(init_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x1c]);
    /// assert_eq!(bm1397.registers.get(&ClockOrderControl0::ADDR).unwrap(), &0x0000_0000);
    /// assert_eq!(bm1397.registers.get(&ClockOrderControl1::ADDR).unwrap(), &0x0000_0000);
    /// assert_eq!(bm1397.registers.get(&OrderedClockEnable::ADDR).unwrap(), &0x0000_0001);
    // assert_eq!(bm1397.core_registers.get(&ClockDelayCtrl::ID).unwrap(), &0x74);
    /// assert_eq!(bm1397.registers.get(&TicketMask::ADDR).unwrap(), &0x0000_00ff);
    /// ```
    ///
    fn init(&mut self, initial_diffculty: u32) -> Vec<CmdDelay, 20> {
        let mut init_seq = Vec::new();
        // let clk_ord_ctrl = ClockOrderControl0(
        //     *self.registers.get(&ClockOrderControl0::ADDR).unwrap(),
        // )
        // .set_clk_sel(0)
        // .val();
        let clk_ord_ctrl = 0x0000_0000;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(ClockOrderControl0::ADDR, clk_ord_ctrl, Destination::All), // all CLK_SELx = 0b0000
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers
            .insert(ClockOrderControl0::ADDR, clk_ord_ctrl)
            .unwrap();
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(ClockOrderControl1::ADDR, clk_ord_ctrl, Destination::All), // all CLK_SELx = 0b0000
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers
            .insert(ClockOrderControl1::ADDR, clk_ord_ctrl)
            .unwrap();
        // let clk_ord_en = OrderedClockEnable(
        //     *self.registers.get(&OrderedClockEnable::ADDR).unwrap(),
        // )
        // .disable_all()
        // .enable(0)
        // .val();
        let clk_ord_en = 0x0000_0001;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(OrderedClockEnable::ADDR, clk_ord_en, Destination::All), // Only enable the first one
                delay: Duration::from_millis(0),
            })
            .unwrap();
        self.registers
            .insert(OrderedClockEnable::ADDR, clk_ord_en)
            .unwrap();
        // let clk_dly_ctrl = ClockDelayCtrl(
        //     *self.core_registers.get(&ClockDelayCtrl::ID).unwrap(),
        // )
        // .set_ccdly_sel(1)
        // .set_pwth_sel(3)
        // .enable_multi_midstate()
        // .val();
        let clk_dly_ctrl = 0x74;
        init_seq
            .push(CmdDelay {
                cmd: Command::write_reg(
                    CoreRegisterControl::ADDR,
                    CoreRegisterControl::write_core_reg(0, ClockDelayCtrl(clk_dly_ctrl)), // CCDLY_SEL=1 PWTH_SEL=3 HASH_CLKEN=0 MMEN=1 SWPF_MODE=0
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
        init_seq
    }

    /// ## Set Baudrate
    ///
    /// ### Example
    /// ```
    /// use bm1397::{BM1397, BM1397_PLL_ID_UART};
    /// use bm13xx_asic::register::*;
    /// use bm13xx_protocol::Bm13xxProtocol;
    ///
    /// let mut bm1397 = BM1397::default();
    /// let mut baud_seq = bm1397.set_baudrate(6_250_000);
    /// assert_eq!(baud_seq.len(), 3);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x01, 0x27, 0x01, 11]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x28, 0x06, 0x00, 0x00, 0x0f, 24]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0xc0, 0x70, 0x01, 0x11, 0]);
    /// assert!(bm1397.plls[BM1397_PLL_ID_UART].enabled());
    /// assert_eq!(bm1397.registers.get(&MiscControl::ADDR).unwrap(), &0x0001_2701);
    /// assert_eq!(bm1397.registers.get(&PLL3Parameter::ADDR).unwrap(), &0xC070_0111);
    /// assert_eq!(bm1397.registers.get(&FastUARTConfiguration::ADDR).unwrap(), &0x0600_000F);
    /// let mut baud_seq = bm1397.set_baudrate(115_740);
    /// assert_eq!(baud_seq.len(), 2);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x68, 0x00, 0x70, 0x01, 0x11, 21]);
    /// assert_eq!(baud_seq.pop().unwrap().cmd, [0x55, 0xaa, 0x51, 0x09, 0x00, 0x18, 0x00, 0x00, 0x3a, 0x01, 24]);
    /// assert!(!bm1397.plls[BM1397_PLL_ID_UART].enabled());
    /// assert_eq!(bm1397.registers.get(&MiscControl::ADDR).unwrap(), &0x0000_3A01);
    /// assert_eq!(bm1397.registers.get(&PLL3Parameter::ADDR).unwrap(), &0x0070_0111);
    /// ```
    fn set_baudrate(&mut self, baudrate: u32) -> Vec<CmdDelay, 3> {
        let mut baud_seq = Vec::new();
        if baudrate <= self.input_clock_freq.raw() / 8 {
            let fbase = self.input_clock_freq.raw();
            let bt8d = (fbase / (8 * baudrate)) - 1;
            let misc_ctrl = MiscControl(*self.registers.get(&MiscControl::ADDR).unwrap())
                .set_bclk_sel(BaudrateClockSelect::Clki)
                .set_bt8d(bt8d as u16)
                .val();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(MiscControl::ADDR, misc_ctrl, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers.insert(MiscControl::ADDR, misc_ctrl).unwrap();
            let pll3_param = self.plls[BM1397_PLL_ID_UART].disable().unlock().parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL3Parameter::ADDR, pll3_param, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(PLL3Parameter::ADDR, pll3_param)
                .unwrap();
        } else {
            let pll3_div4 = 6;
            // self.plls[BM1397_PLL_ID_UART].enable().set_fbdiv(112).set_refdiv(1).set_postdiv1(1).set_postdiv2(1).set_out_div(BM1397_PLL_OUT_UART, pll3_div4);
            self.plls[BM1397_PLL_ID_UART]
                .set_parameter(0xC070_0111)
                .set_out_div(BM1397_PLL_OUT_UART, pll3_div4);
            let fbase = self.plls[BM1397_PLL_ID_UART]
                .frequency(self.input_clock_freq, BM1397_PLL_OUT_UART)
                .raw();
            let pll3_param = self.plls[BM1397_PLL_ID_UART].parameter();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(PLL3Parameter::ADDR, pll3_param, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(PLL3Parameter::ADDR, pll3_param)
                .unwrap();
            let fast_uart_cfg =
                FastUARTConfiguration(*self.registers.get(&FastUARTConfiguration::ADDR).unwrap())
                    .set_pll3_div4(pll3_div4)
                    .val();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(
                        FastUARTConfiguration::ADDR,
                        fast_uart_cfg,
                        Destination::All,
                    ),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers
                .insert(FastUARTConfiguration::ADDR, fast_uart_cfg)
                .unwrap();
            let bt8d = (fbase / (8 * baudrate)) - 1;
            let misc_ctrl = MiscControl(*self.registers.get(&MiscControl::ADDR).unwrap())
                .set_bclk_sel(BaudrateClockSelect::Pll3)
                .set_bt8d(bt8d as u16)
                .val();
            baud_seq
                .push(CmdDelay {
                    cmd: Command::write_reg(MiscControl::ADDR, misc_ctrl, Destination::All),
                    delay: Duration::from_millis(0),
                })
                .unwrap();
            self.registers.insert(MiscControl::ADDR, misc_ctrl).unwrap();
        }
        baud_seq
    }
}
