use crate::register::Register;

/// # Analog Mux Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnalogMuxControl(pub u32);
impl_boilerplate_for!(AnalogMuxControl);

impl AnalogMuxControl {
    pub const ADDR: u8 = 0x54;

    // const DIODE_VDD_MUX_SEL_OFFSET: u8 = 0;

    // const DIODE_VDD_MUX_SEL_MASK: u32 = 0b111;
}

impl core::fmt::Display for AnalogMuxControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnalogMuxControl").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AnalogMuxControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "AnalogMuxControl {{  }}",);
    }
}

/// # Analog Mux Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnalogMuxControlV2(pub u32);
impl_boilerplate_for!(AnalogMuxControlV2);

impl AnalogMuxControlV2 {
    pub const ADDR: u8 = 0x54;

    // const DIODE_VDD_MUX_SEL_OFFSET: u8 = 0;

    // const DIODE_VDD_MUX_SEL_MASK: u32 = 0b1111;
}

impl core::fmt::Display for AnalogMuxControlV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnalogMuxControlV2").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AnalogMuxControlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "AnalogMuxControlV2 {{  }}",);
    }
}
