use crate::core_register::CoreRegister;

/// # Core Enable core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreEnable(pub u8);
impl_boilerplate_for_core_reg!(CoreEnable);

impl CoreEnable {
    pub const ID: u8 = 4;

    // const CORE_EN_I_OFFSET: u8 = 0;

    // const CORE_EN_I_MASK: u8 = 0xff;
}

impl ::core::fmt::Display for CoreEnable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreEnable").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for CoreEnable {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreEnable {{ }}",);
    }
}
