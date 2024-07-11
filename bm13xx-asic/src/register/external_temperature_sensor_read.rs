use crate::register::Register;

/// # External Temperature Sensor Read register
///
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ExternalTemperatureSensorRead(pub u32);
impl_boilerplate_for!(ExternalTemperatureSensorRead);

impl ExternalTemperatureSensorRead {
    pub const ADDR: u8 = 0x44;

    // const LOCAL_TEMP_ADDR_OFFSET: u8 = 24;
    // const LOCAL_TEMP_DATA_OFFSET: u8 = 16;
    // const EXTERNAL_TEMP_ADDR_OFFSET: u8 = 8;
    // const EXTERNAL_TEMP_DATA_OFFSET: u8 = 0;

    // const LOCAL_TEMP_ADDR_MASK: u32 = 0xff;
    // const LOCAL_TEMP_DATA_MASK: u32 = 0xff;
    // const EXTERNAL_TEMP_ADDR_MASK: u32 = 0xff;
    // const EXTERNAL_TEMP_DATA_MASK: u32 = 0xff;
}

impl core::fmt::Display for ExternalTemperatureSensorRead {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExternalTemperatureSensorRead").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ExternalTemperatureSensorRead {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ExternalTemperatureSensorRead {{  }}",);
    }
}
