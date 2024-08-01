use crate::register::Register;

/// # Hash Rate register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashRate(pub u32);
impl_boilerplate_for!(HashRate);

impl HashRate {
    pub const ADDR: u8 = 0x04;

    // const LONG_OFFSET: u8 = 31;
    // const HASHRATE_OFFSET: u8 = 0;

    // const LONG_MASK: u32 = 0b1;
    // const HASHRATE_MASK: u32 = 0x7fff_ffff;
}

impl core::fmt::Display for HashRate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HashRate").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for HashRate {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HashRate {{  }}",);
    }
}
