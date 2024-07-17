use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreReg2(pub u8);
impl_boilerplate_for_core_reg!(CoreReg2);

impl CoreReg2 {
    pub const ID: u8 = 2;
}

impl ::core::fmt::Display for CoreReg2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreReg2").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreReg2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreReg2 {{  }}", self.enabled(),);
    }
}
