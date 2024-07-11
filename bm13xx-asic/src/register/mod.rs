use crate::{Error, Result};

pub trait Register {
    fn addr(&self) -> u8;
    fn val(&self) -> u32;
}

macro_rules! impl_boilerplate_for {
    ($REG:ident) => {
        impl From<u32> for $REG {
            fn from(val: u32) -> Self {
                Self(val)
            }
        }

        impl From<$REG> for u32 {
            fn from(val: $REG) -> u32 {
                val.0
            }
        }

        impl Register for $REG {
            fn addr(&self) -> u8 {
                Self::ADDR
            }
            fn val(&self) -> u32 {
                self.0
            }
        }
    };
}

mod analog_mux_control;
mod chip_identification;
mod chip_nonce_offset;
mod clock_order;
mod core_register;
mod error_flag;
mod external_temperature_sensor_read;
mod fast_uart_configuration;
mod frequency_sweep_control;
mod golden_nonce_for_sweep_return;
mod hash_counting_number;
mod hash_rate;
mod i2c_control;
mod io_driver_strenght_configuration;
mod misc_control;
mod nonce_counter;
mod nonce_returned_timeout;
mod ordered_clock_enable;
mod pll_divider;
mod pll_parameter;
mod return_group_pattern_status;
mod returned_single_pattern_status;
mod ticket_mask;
mod timeout;
mod uart_relay;
mod unknown;

pub use analog_mux_control::AnalogMuxControl;
pub use chip_identification::ChipIdentification;
pub use chip_nonce_offset::ChipNonceOffset;
pub use clock_order::{
    ClockOrderControl0, ClockOrderControl1, ClockOrderStatus, ClockSelect, OrderedClockMonitor,
};
pub use core_register::{CoreRegisterControl, CoreRegisterValue};
pub use error_flag::ErrorFlag;
pub use external_temperature_sensor_read::ExternalTemperatureSensorRead;
pub use fast_uart_configuration::FastUARTConfiguration;
pub use frequency_sweep_control::FrequencySweepControl1;
pub use golden_nonce_for_sweep_return::GoldenNonceForSweepReturn;
pub use hash_counting_number::HashCountingNumber;
pub use hash_rate::HashRate;
pub use i2c_control::I2CControl;
pub use io_driver_strenght_configuration::IoDriverStrenghtConfiguration;
pub use misc_control::{BaudrateClockSelect, MiscControl};
pub use nonce_counter::{NonceErrorCounter, NonceOverflowCounter};
pub use nonce_returned_timeout::NonceReturnedTimeout;
pub use ordered_clock_enable::OrderedClockEnable;
pub use pll_divider::{PLL0Divider, PLL1Divider, PLL2Divider, PLL3Divider};
pub use pll_parameter::{
    PLL0Parameter, PLL1Parameter, PLL2Parameter, PLL3Parameter, PLLParameterRegister,
};
pub use return_group_pattern_status::ReturnedGroupPatternStatus;
pub use returned_single_pattern_status::ReturnedSinglePatternStatus;
pub use ticket_mask::{TicketMask, TicketMask2};
pub use timeout::TimeOut;
pub use uart_relay::UARTRelay;
pub use unknown::{
    Reg24, Reg30, Reg34, RegA4, RegA8, RegAC, RegB0, RegB4, RegB8, RegBC, RegC0, RegC4, RegC8,
    RegCC, RegD0, RegD4, RegD8, RegDC, RegE0, RegE4, RegE8, RegEC, RegF0, RegF4, RegF8, RegFC,
};

#[derive(Debug, PartialEq)]
pub enum Registers {
    ChipIdentification(ChipIdentification),       // 0x00
    HashRate(HashRate),                           // 0x04
    PLL0Parameter(PLL0Parameter),                 // 0x08
    ChipNonceOffset(ChipNonceOffset),             // 0x0C
    HashCountingNumber(HashCountingNumber),       // 0x10
    TicketMask(TicketMask),                       // 0x14
    MiscControl(MiscControl),                     // 0x18
    I2CControl(I2CControl),                       // 0x1C
    OrderedClockEnable(OrderedClockEnable),       // 0x20
    Reg24(Reg24),                                 // 0x24
    FastUARTConfiguration(FastUARTConfiguration), // 0x28
    UARTRelay(UARTRelay),                         // 0x2C
    Reg30(Reg30),                                 // 0x30
    Reg34(Reg34),                                 // 0x34
    TicketMask2(TicketMask2),
    CoreRegisterControl(CoreRegisterControl),
    CoreRegisterValue(CoreRegisterValue),
    ExternalTemperatureSensorRead(ExternalTemperatureSensorRead),
    ErrorFlag(ErrorFlag),
    NonceErrorCounter(NonceErrorCounter),
    NonceOverflowCounter(NonceOverflowCounter),
    AnalogMuxControl(AnalogMuxControl),
    IoDriverStrenghtConfiguration(IoDriverStrenghtConfiguration),
    TimeOut(TimeOut),
    PLL1Parameter(PLL1Parameter),
    PLL2Parameter(PLL2Parameter),
    PLL3Parameter(PLL3Parameter),
    OrderedClockMonitor(OrderedClockMonitor),
    PLL0Divider(PLL0Divider),
    PLL1Divider(PLL1Divider),
    PLL2Divider(PLL2Divider),
    PLL3Divider(PLL3Divider),
    ClockOrderControl0(ClockOrderControl0),
    ClockOrderControl1(ClockOrderControl1),
    ClockOrderStatus(ClockOrderStatus),
    FrequencySweepControl1(FrequencySweepControl1),
    GoldenNonceForSweepReturn(GoldenNonceForSweepReturn),
    ReturnedGroupPatternStatus(ReturnedGroupPatternStatus),
    NonceReturnedTimeout(NonceReturnedTimeout),
    ReturnedSinglePatternStatus(ReturnedSinglePatternStatus),
    RegA4(RegA4),
    RegA8(RegA8),
    RegAC(RegAC),
    RegB0(RegB0),
    RegB4(RegB4),
    RegB8(RegB8),
    RegBC(RegBC),
    RegC0(RegC0),
    RegC4(RegC4),
    RegC8(RegC8),
    RegCC(RegCC),
    RegD0(RegD0),
    RegD4(RegD4),
    RegD8(RegD8),
    RegDC(RegDC),
    RegE0(RegE0),
    RegE4(RegE4),
    RegE8(RegE8),
    RegEC(RegEC),
    RegF0(RegF0),
    RegF4(RegF4),
    RegF8(RegF8),
    RegFC(RegFC),
}

pub fn parse(addr: u8, val: u32) -> Result<Registers> {
    match addr {
        ChipIdentification::ADDR => Ok(Registers::ChipIdentification(ChipIdentification(val))),
        HashRate::ADDR => Ok(Registers::HashRate(HashRate(val))),
        PLL0Parameter::ADDR => Ok(Registers::PLL0Parameter(PLL0Parameter(val))),
        ChipNonceOffset::ADDR => Ok(Registers::ChipNonceOffset(ChipNonceOffset(val))),
        HashCountingNumber::ADDR => Ok(Registers::HashCountingNumber(HashCountingNumber(val))),
        TicketMask::ADDR => Ok(Registers::TicketMask(TicketMask(val))),
        MiscControl::ADDR => Ok(Registers::MiscControl(MiscControl(val))),
        I2CControl::ADDR => Ok(Registers::I2CControl(I2CControl(val))),
        OrderedClockEnable::ADDR => Ok(Registers::OrderedClockEnable(OrderedClockEnable(val))),
        FastUARTConfiguration::ADDR => {
            Ok(Registers::FastUARTConfiguration(FastUARTConfiguration(val)))
        }
        UARTRelay::ADDR => Ok(Registers::UARTRelay(UARTRelay(val))),
        TicketMask2::ADDR => Ok(Registers::TicketMask2(TicketMask2(val))),
        CoreRegisterControl::ADDR => Ok(Registers::CoreRegisterControl(CoreRegisterControl(val))),
        CoreRegisterValue::ADDR => Ok(Registers::CoreRegisterValue(CoreRegisterValue(val))),
        ExternalTemperatureSensorRead::ADDR => Ok(Registers::ExternalTemperatureSensorRead(
            ExternalTemperatureSensorRead(val),
        )),
        ErrorFlag::ADDR => Ok(Registers::ErrorFlag(ErrorFlag(val))),
        NonceErrorCounter::ADDR => Ok(Registers::NonceErrorCounter(NonceErrorCounter(val))),
        NonceOverflowCounter::ADDR => {
            Ok(Registers::NonceOverflowCounter(NonceOverflowCounter(val)))
        }
        AnalogMuxControl::ADDR => Ok(Registers::AnalogMuxControl(AnalogMuxControl(val))),
        IoDriverStrenghtConfiguration::ADDR => Ok(Registers::IoDriverStrenghtConfiguration(
            IoDriverStrenghtConfiguration(val),
        )),
        TimeOut::ADDR => Ok(Registers::TimeOut(TimeOut(val))),
        PLL1Parameter::ADDR => Ok(Registers::PLL1Parameter(PLL1Parameter(val))),
        PLL2Parameter::ADDR => Ok(Registers::PLL2Parameter(PLL2Parameter(val))),
        PLL3Parameter::ADDR => Ok(Registers::PLL3Parameter(PLL3Parameter(val))),
        OrderedClockMonitor::ADDR => Ok(Registers::OrderedClockMonitor(OrderedClockMonitor(val))),
        PLL0Divider::ADDR => Ok(Registers::PLL0Divider(PLL0Divider(val))),
        PLL1Divider::ADDR => Ok(Registers::PLL1Divider(PLL1Divider(val))),
        PLL2Divider::ADDR => Ok(Registers::PLL2Divider(PLL2Divider(val))),
        PLL3Divider::ADDR => Ok(Registers::PLL3Divider(PLL3Divider(val))),
        ClockOrderControl0::ADDR => Ok(Registers::ClockOrderControl0(ClockOrderControl0(val))),
        ClockOrderControl1::ADDR => Ok(Registers::ClockOrderControl1(ClockOrderControl1(val))),
        ClockOrderStatus::ADDR => Ok(Registers::ClockOrderStatus(ClockOrderStatus(val))),
        FrequencySweepControl1::ADDR => Ok(Registers::FrequencySweepControl1(
            FrequencySweepControl1(val),
        )),
        GoldenNonceForSweepReturn::ADDR => Ok(Registers::GoldenNonceForSweepReturn(
            GoldenNonceForSweepReturn(val),
        )),
        ReturnedGroupPatternStatus::ADDR => Ok(Registers::ReturnedGroupPatternStatus(
            ReturnedGroupPatternStatus(val),
        )),
        NonceReturnedTimeout::ADDR => {
            Ok(Registers::NonceReturnedTimeout(NonceReturnedTimeout(val)))
        }
        ReturnedSinglePatternStatus::ADDR => Ok(Registers::ReturnedSinglePatternStatus(
            ReturnedSinglePatternStatus(val),
        )),
        RegA4::ADDR => Ok(Registers::RegA4(RegA4(val))),
        RegA8::ADDR => Ok(Registers::RegA8(RegA8(val))),
        RegAC::ADDR => Ok(Registers::RegAC(RegAC(val))),
        RegB0::ADDR => Ok(Registers::RegB0(RegB0(val))),
        RegB4::ADDR => Ok(Registers::RegB4(RegB4(val))),
        RegB8::ADDR => Ok(Registers::RegB8(RegB8(val))),
        RegBC::ADDR => Ok(Registers::RegBC(RegBC(val))),
        RegC0::ADDR => Ok(Registers::RegC0(RegC0(val))),
        RegC4::ADDR => Ok(Registers::RegC4(RegC4(val))),
        RegC8::ADDR => Ok(Registers::RegC8(RegC8(val))),
        RegCC::ADDR => Ok(Registers::RegCC(RegCC(val))),
        RegD0::ADDR => Ok(Registers::RegD0(RegD0(val))),
        RegD4::ADDR => Ok(Registers::RegD4(RegD4(val))),
        RegD8::ADDR => Ok(Registers::RegD8(RegD8(val))),
        RegDC::ADDR => Ok(Registers::RegDC(RegDC(val))),
        RegE0::ADDR => Ok(Registers::RegE0(RegE0(val))),
        RegE4::ADDR => Ok(Registers::RegE4(RegE4(val))),
        RegE8::ADDR => Ok(Registers::RegE8(RegE8(val))),
        RegEC::ADDR => Ok(Registers::RegEC(RegEC(val))),
        RegF0::ADDR => Ok(Registers::RegF0(RegF0(val))),
        RegF4::ADDR => Ok(Registers::RegF4(RegF4(val))),
        RegF8::ADDR => Ok(Registers::RegF8(RegF8(val))),
        RegFC::ADDR => Ok(Registers::RegFC(RegFC(val))),
        addr => Err(Error::UnknownRegister { reg_addr: addr }),
    }
}
