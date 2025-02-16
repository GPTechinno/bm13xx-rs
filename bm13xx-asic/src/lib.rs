//! BM13xx ASIC representation.

#![no_std]
#![macro_use]
pub(crate) mod fmt;

pub mod core_register;
mod error;
pub mod pll;
pub mod register;
pub mod sha;

pub use self::error::{Error, Result};

use bm13xx_protocol::command::Destination;

use fugit::HertzU64;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct CmdDelay {
    pub cmd: [u8; 11],
    pub delay_ms: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum SequenceStep {
    #[default]
    None,
    Init(usize),
    Baudrate(usize),
    ResetCore(usize),
    HashFreq(usize),
    SplitNonce(usize),
    VersionRolling(usize),
}

pub trait Asic {
    fn reset(&mut self);
    fn chip_id(&self) -> u16;
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
    fn set_version_rolling_next(&mut self, mask: u32) -> Option<CmdDelay>;
}
