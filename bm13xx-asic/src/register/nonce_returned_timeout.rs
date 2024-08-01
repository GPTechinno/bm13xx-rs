use crate::register::Register;

/// # Nonce Returned Timeout register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NonceReturnedTimeout(pub u32);
impl_boilerplate_for!(NonceReturnedTimeout);

impl NonceReturnedTimeout {
    pub const ADDR: u8 = 0x9C;

    // const SWEEP_TIMEOUT_OFFSET: u8 = 0;

    // const SWEEP_TIMEOUT_MASK: u32 = 0xffff;
}

impl core::fmt::Display for NonceReturnedTimeout {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NonceReturnedTimeout").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for NonceReturnedTimeout {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "NonceReturnedTimeout {{  }}",);
    }
}
