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

mod clock_delay;
mod core_enable;
mod core_error;
mod hash_clock;
mod process_monitor;
mod sweep_clock;

pub use clock_delay::ClockDelayCtrl;
pub use core_enable::CoreEnable;
pub use core_error::CoreError;
pub use hash_clock::{HashClockCounter, HashClockCtrl};
pub use process_monitor::{ProcessMonitorCtrl, ProcessMonitorData, ProcessMonitorSelect};
pub use sweep_clock::SweepClockCtrl;
