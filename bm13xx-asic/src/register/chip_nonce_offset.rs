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
    pub const CNO_MASK: u32 = 0b111;
}

impl core::fmt::Display for ChipNonceOffset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChipNonceOffset").finish()
    }
}

#[cfg(feature = "defmt")]
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

    const CNOV_OFFSET: u8 = 31;
    const CNO_OFFSET: u8 = 0;

    const CNOV_MASK: u32 = 0b1;
    pub const CNO_MASK: u32 = 0xffff;

    /// ## Create a chip nonce offset for a given asic in a given chain.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipNonceOffsetV2;
    ///
    /// assert_eq!(ChipNonceOffsetV2::new(1, 65), ChipNonceOffsetV2(0x8000_03f1));
    /// assert_eq!(ChipNonceOffsetV2::new(64, 65), ChipNonceOffsetV2(0x8000_fc10));
    /// ```
    pub fn new(asic_index: usize, chain_asic_num: usize) -> Self {
        if chain_asic_num == 0 {
            Self(0)
        } else {
            Self(
                (Self::CNOV_MASK << Self::CNOV_OFFSET)
                    + (Self::CNO_MASK + 1) * (asic_index as u32) / (chain_asic_num as u32)
                    + if asic_index > 0 { 1 } else { 0 },
            )
        }
    }

    /// ## Get the chip nonce offset value.
    ///
    /// This returns an `u16` with the offset value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipNonceOffsetV2;
    ///
    /// assert_eq!(ChipNonceOffsetV2(0x8000_03f1).offset(), 0x03f1);
    /// ```
    pub const fn offset(&self) -> u16 {
        ((self.0 >> Self::CNO_OFFSET) & Self::CNO_MASK) as u16
    }

    /// ## Get the chip nonce offset validity.
    ///
    /// This returns an `bool` with the offset validity.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipNonceOffsetV2;
    ///
    /// assert_eq!(ChipNonceOffsetV2(0x8000_03f1).offset(), 0x03f1);
    /// ```
    pub const fn valid(&self) -> bool {
        (self.0 >> (Self::CNOV_OFFSET)) & Self::CNOV_MASK == Self::CNOV_MASK
    }
}

impl core::fmt::Display for ChipNonceOffsetV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChipNonceOffsetV2").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ChipNonceOffsetV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ChipNonceOffsetV2 {{  }}",);
    }
}
