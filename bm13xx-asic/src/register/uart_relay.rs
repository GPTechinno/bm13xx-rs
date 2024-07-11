use crate::register::Register;

/// # UART Relay register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct UARTRelay(pub u32);
impl_boilerplate_for!(UARTRelay);

impl UARTRelay {
    pub const ADDR: u8 = 0x2C;

    // const GAP_CNT_OFFSET: u8 = 16;
    // const RO_REL_EN_OFFSET: u8 = 1;
    // const CO_REL_EN_OFFSET: u8 = 0;

    // const GAP_CNT_MASK: u32 = 0xffff;
    // const RO_REL_EN_MASK: u32 = 0b1;
    // const CO_REL_EN_MASK: u32 = 0b1;
}

impl ::core::fmt::Display for UARTRelay {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("UARTRelay").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for UARTRelay {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "UARTRelay {{  }}",);
    }
}
