//! BM13xx Core Registers.

pub trait CoreRegister {
    fn id(&self) -> u8;
    fn val(&self) -> u8;
}

macro_rules! impl_boilerplate_for_core_reg {
    ($REG:ident) => {
        impl From<u8> for $REG {
            fn from(val: u8) -> Self {
                Self(val)
            }
        }

        impl From<$REG> for u8 {
            fn from(val: $REG) -> u8 {
                val.0
            }
        }

        impl CoreRegister for $REG {
            fn id(&self) -> u8 {
                Self::ID
            }
            fn val(&self) -> u8 {
                self.0
            }
        }
    };
}

mod auto_clk_en;
mod clock_delay;
mod clock_select_ctrl;
mod core_enable;
mod core_error;
mod hash_clock;
mod nonce_bin_overflow;
mod ocv_pulse_width;
mod pg_mont;
mod process_monitor;
mod sweep_clock;

pub use auto_clk_en::AutoClkEn;
pub use clock_delay::{ClockDelayCtrl, ClockDelayCtrlV2, ClockDelayCtrlV3};
pub use clock_select_ctrl::ClockSelectCtrl;
pub use core_enable::{CoreEnable, CoreEnableV2};
pub use core_error::CoreError;
pub use hash_clock::{HashClockCounter, HashClockCounterV2, HashClockCtrl, HashClockCtrlV2};
pub use nonce_bin_overflow::NonceBinOverflowControl;
pub use ocv_pulse_width::OcvPulseWidth;
pub use pg_mont::PgMont;
pub use process_monitor::{ProcessMonitorCtrl, ProcessMonitorData, ProcessMonitorSelect};
pub use sweep_clock::SweepClockCtrl;
