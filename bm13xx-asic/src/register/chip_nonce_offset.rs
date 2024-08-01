use crate::register::Register;

/// # Chip Nonce Offset register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChipNonceOffset(pub u32);
impl_boilerplate_for!(ChipNonceOffset);

impl ChipNonceOffset {
    pub const ADDR: u8 = 0x0C;

    // const CNOV_OFFSET: u8 = 31;
    // const CNO_OFFSET: u8 = 0;

    // const CNOV_MASK: u32 = 0b1;
    // const CNO_MASK: u32 = 0b111;
}

impl core::fmt::Display for ChipNonceOffset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChipNonceOffset").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ChipNonceOffset {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ChipNonceOffset {{  }}",);
    }
}

/// # Chip Nonce Offset register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChipNonceOffsetV2(pub u32);
impl_boilerplate_for!(ChipNonceOffsetV2);

impl ChipNonceOffsetV2 {
    pub const ADDR: u8 = 0x0C;

    // const CNOV_OFFSET: u8 = 31;
    // const CNO_OFFSET: u8 = 0;

    // const CNOV_MASK: u32 = 0b1;
    // const CNO_MASK: u32 = 0xffff;
}

impl core::fmt::Display for ChipNonceOffsetV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChipNonceOffsetV2").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ChipNonceOffsetV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ChipNonceOffsetV2 {{  }}",);
    }
}
