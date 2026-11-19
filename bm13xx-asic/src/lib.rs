//! BM13xx ASIC representation.

#![no_std]
pub(crate) mod fmt;

pub mod core_register;
mod error;
pub mod hash_counting;
pub mod pll;
pub mod register;
pub mod sha;
pub mod temperature;

pub use self::error::{Error, Result};

use bm13xx_protocol::command::Destination;

use fugit::HertzU64;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CmdDelay {
    pub cmd: [u8; 11],
    pub delay_ms: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SequenceStep {
    #[default]
    None,
    Init(usize),
    Baudrate(usize),
    ResetCore(usize),
    HashFreq(usize),
    SplitNonce(usize),
    VersionRolling(usize),
    DieTemp(usize),
}

/// One step of a sensor reading sequence.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SensorStep {
    /// Send this command, then wait.
    Write(CmdDelay),
    /// Read this register of the chip the sequence is for, wait `delay_ms`, then collect the
    /// chip's answer.
    Read { reg_addr: u8, delay_ms: u32 },
}

pub trait Asic {
    fn reset(&mut self);
    fn chip_id(&self) -> u16;
    /// Whether a chip identification read with this chip ID is one of these chips.
    fn chip_id_matches(&self, chip_id: u16) -> bool {
        chip_id == self.chip_id()
    }
    /// Largest interval between two chip addresses.
    fn max_addr_interval(&self) -> usize {
        256
    }
    fn core_count(&self) -> usize;
    fn core_small_core_count(&self) -> usize;
    fn small_core_count(&self) -> usize;
    fn cno_interval(&self) -> usize;
    fn cno_bits(&self) -> u32;
    fn hash_freq(&self) -> HertzU64;
    fn init_next(&mut self, difficulty: u32) -> Option<CmdDelay>;
    fn set_baudrate_next(
        &mut self,
        baudrate: u32,
        chain_domain_cnt: usize,
        domain_asic_cnt: usize,
        asic_addr_interval: usize,
    ) -> Option<CmdDelay>;
    fn reset_core_next(&mut self, dest: Destination) -> Option<CmdDelay>;
    fn set_hash_freq_next(&mut self, target_freq: HertzU64) -> Option<CmdDelay>;
    fn split_nonce_between_chips_next(
        &mut self,
        chain_asic_num: usize,
        asic_addr_interval: usize,
    ) -> Option<CmdDelay>;
    fn set_version_rolling_next(
        &mut self,
        mask: u32,
        asic_addr_interval: usize,
    ) -> Option<CmdDelay>;
    /// Die temperature reading sequence for one chip (`dest`). `None` at once when the chip has no
    /// die temperature reading.
    fn die_temp_next(&mut self, _dest: Destination) -> Option<SensorStep> {
        None
    }
    /// Die temperature in °C from the values read by `die_temp_next()`, in order.
    fn die_temp_celsius(&self, _samples: &[u32]) -> Option<f32> {
        None
    }
}
