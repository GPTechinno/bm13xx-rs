use crate::register::Register;

/// # Ticket Mask register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TicketMask(pub u32);
impl_boilerplate_for!(TicketMask);

impl TicketMask {
    pub const ADDR: u8 = 0x14;

    // const TM3_OFFSET: u8 = 24;
    // const TM2_OFFSET: u8 = 16;
    // const TM1_OFFSET: u8 = 8;
    // const TM0_OFFSET: u8 = 0;

    // const TM3_MASK: u32 = 0xff;
    // const TM2_MASK: u32 = 0xff;
    // const TM1_MASK: u32 = 0xff;
    // const TM0_MASK: u32 = 0xff;

    /// ## Create a new `TicketMask` from a difficulty.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{Register, TicketMask};
    ///
    /// assert_eq!(TicketMask::from_difficulty(256), TicketMask(0x0000_00ff));
    /// assert_eq!(TicketMask::from_difficulty(512), TicketMask(0x0000_80ff));
    /// ```
    pub fn from_difficulty(diff: u32) -> Self {
        let largest_power_of_two = (1u32 << (31 - diff.leading_zeros())) - 1u32;
        Self(largest_power_of_two.to_le().reverse_bits().to_be())
    }
}

impl ::core::fmt::Display for TicketMask {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("TicketMask").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TicketMask {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "TicketMask {{  }}",);
    }
}

/// # Ticket Mask 2 register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TicketMask2(pub u32);
impl_boilerplate_for!(TicketMask2);

impl TicketMask2 {
    pub const ADDR: u8 = 0x38;

    // const TM_OFFSET: u8 = 0;

    // const TM_MASK: u32 = 0xffff_ffff;
}

impl ::core::fmt::Display for TicketMask2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("TicketMask2").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for TicketMask2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "TicketMask2 {{  }}",);
    }
}
