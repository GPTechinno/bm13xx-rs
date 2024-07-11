use crate::register::Register;

/// # Chip Identification register
///
/// Used to identify a chip.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChipIdentification(pub u32);
impl_boilerplate_for!(ChipIdentification);

impl ChipIdentification {
    pub const ADDR: u8 = 0x00;

    const CHIP_ID_OFFSET: u8 = 16;
    const CORE_NUM_OFFSET: u8 = 8;
    const ADDR_OFFSET: u8 = 0;

    const CHIP_ID_MASK: u32 = 0xffff;
    const CORE_NUM_MASK: u32 = 0xff;
    const ADDR_MASK: u32 = 0xff;

    /// ## Get the chip identifier.
    ///
    /// This returns an `u16` with the chip_id value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipIdentification;
    ///
    /// assert_eq!(ChipIdentification(0x1397_1800).chip_id(), 0x1397);
    /// ```
    pub const fn chip_id(&self) -> u16 {
        ((self.0 >> Self::CHIP_ID_OFFSET) & Self::CHIP_ID_MASK) as u16
    }

    /// ## Get the number of internal cores.
    ///
    /// This returns an `u8` with the core_num value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipIdentification;
    ///
    /// assert_eq!(ChipIdentification(0x1397_1800).core_num(), 0x18);
    /// ```
    pub const fn core_num(&self) -> u8 {
        ((self.0 >> Self::CORE_NUM_OFFSET) & Self::CORE_NUM_MASK) as u8
    }

    /// ## Get the chip address on the chain.
    ///
    /// This returns an `u8` with the address value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::ChipIdentification;
    ///
    /// assert_eq!(ChipIdentification(0x1397_1800).chip_addr(), 0x00);
    /// ```
    pub const fn chip_addr(&self) -> u8 {
        ((self.0 >> Self::ADDR_OFFSET) & Self::ADDR_MASK) as u8
    }
}

impl core::fmt::Display for ChipIdentification {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ChipIdentification")
            .field("chip_id", &self.chip_id())
            .field("core_num", &self.core_num())
            .field("chip_addr", &self.chip_addr())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ChipIdentification {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ChipIdentification {{ chip_id: {}, core_num: {}, chip_addr: {} }}",
            self.chip_id(),
            self.core_num(),
            self.chip_addr(),
        );
    }
}
