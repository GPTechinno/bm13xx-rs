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
mod version_rolling;

pub use analog_mux_control::AnalogMuxControl;
pub use chip_identification::ChipIdentification;
pub use chip_nonce_offset::ChipNonceOffset;
pub use clock_order::{
    ClockOrderControl0, ClockOrderControl1, ClockOrderStatus, ClockSelect, OrderedClockMonitor,
};
pub use core_register::{CoreRegisterControl, CoreRegisterValue};
pub use error_flag::ErrorFlag;
pub use external_temperature_sensor_read::ExternalTemperatureSensorRead;
pub use fast_uart_configuration::{
    BaudrateClockSelectV2, FastUARTConfiguration, FastUARTConfigurationV2,
};
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
    Reg24, Reg30, Reg34, RegA8, RegAC, RegB0, RegB4, RegB8, RegBC, RegC0, RegC4, RegC8, RegCC,
    RegD0, RegD4, RegD8, RegDC, RegE0, RegE4, RegE8, RegEC, RegF0, RegF4, RegF8, RegFC,
};
pub use version_rolling::VersionRolling;
