use crate::register::Register;

/// # Hash Counting Number register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashCountingNumber(pub u32);
impl_boilerplate_for!(HashCountingNumber);

impl HashCountingNumber {
    pub const ADDR: u8 = 0x10;

    // const HCN_OFFSET: u8 = 0;

    // const HCN_MASK: u32 = 0xffff_ffff;
}

impl ::core::fmt::Display for HashCountingNumber {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("HashCountingNumber").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HashCountingNumber {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "HashCountingNumber {{  }}",);
    }
}
