#![no_std]
//! BM13xx protocol.

mod crc;
mod error;

pub mod command;
pub mod response;

pub use self::error::{Error, Result};
