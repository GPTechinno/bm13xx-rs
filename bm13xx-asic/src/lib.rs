#![no_std]
//! BM13xx ASIC representation.

mod error;
pub mod pll;
pub mod register;
pub mod sha;

pub use self::error::{Error, Result};
