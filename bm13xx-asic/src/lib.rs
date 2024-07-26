#![no_std]
//! BM13xx ASIC representation.

pub mod core_register;
mod error;
pub mod pll;
pub mod register;
pub mod sha;

pub use self::error::{Error, Result};

use bm13xx_protocol::command::Destination;

use heapless::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct CmdDelay {
    pub cmd: [u8; 11],
    pub delay_ms: u32,
}

pub trait Asic {
    fn chip_id(&self) -> u16;
    fn init(
        &mut self,
        initial_diffculty: u32,
        chain_domain_cnt: u8,
        domain_asic_cnt: u8,
        asic_addr_interval: u16,
    ) -> Vec<CmdDelay, 14>;
    fn reset_core(&mut self, dest: Destination) -> Vec<CmdDelay, 6>;
    fn set_baudrate(&mut self, baudrate: u32) -> Vec<CmdDelay, 3>;
}
