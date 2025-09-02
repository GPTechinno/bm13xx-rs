use crate::register::Register;

/// # Nonce Error Counter register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NonceErrorCounter(pub u32);
impl_boilerplate_for!(NonceErrorCounter);

impl NonceErrorCounter {
    pub const ADDR: u8 = 0x4C;

    // const ERR_CNT_OFFSET: u8 = 0;

    // const ERR_CNT_MASK: u32 = 0xffff_ffff;
}

impl core::fmt::Display for NonceErrorCounter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NonceErrorCounter").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for NonceErrorCounter {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "NonceErrorCounter {{  }}",);
    }
}

/// # Nonce Overflow Counter register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NonceOverflowCounter(pub u32);
impl_boilerplate_for!(NonceOverflowCounter);

impl NonceOverflowCounter {
    pub const ADDR: u8 = 0x50;

    // const OVRF_CNT_OFFSET: u8 = 0;

    // const OVRF_CNT_MASK: u32 = 0xffff_ffff;
}

impl core::fmt::Display for NonceOverflowCounter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NonceOverflowCounter").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for NonceOverflowCounter {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "NonceOverflowCounter {{  }}",);
    }
}
