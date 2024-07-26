#![no_std]
//! BM13xx ASIC representation.

pub mod core_register;
mod error;
pub mod pll;
pub mod register;
pub mod sha;

pub use self::error::{Error, Result};

pub trait Asic {
    fn chip_id(&self) -> u16;
}
