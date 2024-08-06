//! BM13xx protocol.

#![no_std]
#![macro_use]
pub(crate) mod fmt;

mod crc;
mod error;

pub mod command;
pub mod response;

pub use self::error::{Error, Result};
