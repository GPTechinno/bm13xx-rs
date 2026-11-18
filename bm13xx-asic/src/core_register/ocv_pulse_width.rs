use crate::core_register::CoreRegister;

/// # OCV Pulse Width core register (`OCV_PULSE_WIDTH`)
///
/// Core register numbering of the BM1368 and later.
///
/// Bits 11:8 (`OCV_DOUT`) lie beyond the 8 bits a core register write carries.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OcvPulseWidth(pub u8);
impl_boilerplate_for_core_reg!(OcvPulseWidth);

impl OcvPulseWidth {
    pub const ID: u8 = 22;

    const PULSE_WIDTH_OFFSET: u8 = 0;

    const PULSE_WIDTH_MASK: u8 = 0b11111111;

    /// ## Handle the `PULSE_WIDTH` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::OcvPulseWidth;
    ///
    /// let mut reg = OcvPulseWidth(0x00); // default value
    /// assert_eq!(reg.pulse_width(), 0x00);
    /// assert_eq!(reg.set_pulse_width(0x40).pulse_width(), 0x40);
    /// ```
    pub const fn pulse_width(&self) -> u8 {
        (self.0 >> Self::PULSE_WIDTH_OFFSET) & Self::PULSE_WIDTH_MASK
    }
    pub fn set_pulse_width(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::PULSE_WIDTH_MASK << Self::PULSE_WIDTH_OFFSET);
        self.0 |= (value & Self::PULSE_WIDTH_MASK) << Self::PULSE_WIDTH_OFFSET;
        self
    }
}

impl ::core::fmt::Display for OcvPulseWidth {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("OcvPulseWidth")
            .field("pulse_width", &self.pulse_width())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for OcvPulseWidth {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "OcvPulseWidth {{ pulse_width: {} }}",
            self.pulse_width(),
        );
    }
}
