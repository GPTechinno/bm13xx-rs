use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreReg11(pub u8);
impl_boilerplate_for_core_reg!(CoreReg11);

impl CoreReg11 {
    pub const ID: u8 = 11;

    // const SOME_OFFSET: u8 = 5;
    // const SOME2_OFFSET: u8 = 0;

    // const SOME_MASK: u8 = 0b11;
    // const SOME2_MASK: u8 = 0b1;
}

impl ::core::fmt::Display for CoreReg11 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreReg11").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for CoreReg11 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreReg11 {{ }}",);
    }
}
