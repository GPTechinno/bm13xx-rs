use crate::core_register::CoreRegister;

/// # Core register 13
///
/// Meaning unknown. The BM1370 stock firmware writes `0xEE` to it on all chips just before
/// ramping the hash frequency.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreReg13(pub u8);
impl_boilerplate_for_core_reg!(CoreReg13);

impl CoreReg13 {
    pub const ID: u8 = 13;

    // const SOME_OFFSET: u8 = 5;
    // const SOME2_OFFSET: u8 = 0;

    // const SOME_MASK: u8 = 0b11;
    // const SOME2_MASK: u8 = 0b1;
}

impl ::core::fmt::Display for CoreReg13 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreReg13").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreReg13 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreReg13 {{ }}",);
    }
}
