#![no_std]
//! BM13xx protocol.

use core::time::Duration;
use heapless::Vec;

use command::Destination;

mod crc;
mod error;

pub mod command;
pub mod response;

pub use self::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct CmdDelay {
    pub cmd: [u8; 11],
    pub delay: Duration,
}

pub trait Bm13xxProtocol {
    fn init(&mut self, initial_diffculty: u32) -> Vec<CmdDelay, 5>;
    fn reset_core(&mut self, dest: Destination) -> Vec<CmdDelay, 6>;
    fn set_baudrate(&mut self, baudrate: u32) -> Vec<CmdDelay, 3>;
}
