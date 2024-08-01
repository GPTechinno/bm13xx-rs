use crate::register::Register;

/// # I2C Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct I2CControl(pub u32);
impl_boilerplate_for!(I2CControl);

impl I2CControl {
    pub const ADDR: u8 = 0x1C;

    // const BUSY_OFFSET: u8 = 31;
    // const DO_CMD_OFFSET: u8 = 24;
    // const I2C_ADDR_OFFSET: u8 = 17;
    // const RD_WR_OFFSET: u8 = 16;
    // const I2C_REG_ADDR_OFFSET: u8 = 8;
    // const I2C_REG_VAL_OFFSET: u8 = 0;

    // const BUSY_MASK: u32 = 0b1;
    // const DO_CMD_MASK: u32 = 0b1;
    // const I2C_ADDR_MASK: u32 = 0x7f;
    // const RD_WR_MASK: u32 = 0b1;
    // const I2C_REG_ADDR_MASK: u32 = 0xff;
    // const I2C_REG_VAL_MASK: u32 = 0xff;
}

impl core::fmt::Display for I2CControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("I2CControl").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for I2CControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "I2CControl {{  }}",);
    }
}
