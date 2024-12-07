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
use heapless::Vec;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct CmdDelay {
    pub cmd: [u8; 11],
    pub delay_ms: u32,
}

pub trait Asic {
    fn chip_id(&self) -> u16;
    fn has_version_rolling(&self) -> bool;
    fn send_init(
        &mut self,
        initial_diffculty: u32,
        chain_domain_cnt: u8,
        domain_asic_cnt: u8,
        asic_addr_interval: u16,
    ) -> Vec<CmdDelay, 2048>;
    fn send_baudrate(&mut self, baudrate: u32,         chain_domain_cnt: u8,
        domain_asic_cnt: u8,
        asic_addr_interval: u16,) -> Vec<CmdDelay, 800>;
    fn send_reset_core(&mut self, dest: Destination,) -> Vec<CmdDelay, 800>;
    fn send_hash_freq(&mut self, target_freq: HertzU64) -> Vec<CmdDelay, 800>;
    fn send_version_rolling(&mut self, mask: u32,        chain_domain_cnt: u8,
        domain_asic_cnt: u8,
        asic_addr_interval: u16,) -> Vec<CmdDelay, 800>;

    // TODO: Findout where should be placed and remove from here
    fn between_reset_and_set_freq(&mut self) -> Vec<CmdDelay, 40>;
}
