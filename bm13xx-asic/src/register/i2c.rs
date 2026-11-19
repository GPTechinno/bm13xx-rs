use crate::register::Register;

/// # I2C Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct I2CControl(pub u32);
impl_boilerplate_for!(I2CControl);

impl I2CControl {
    pub const ADDR: u8 = 0x1C;

    const BUSY_OFFSET: u8 = 31;
    const DO_CMD_OFFSET: u8 = 24;
    const I2C_ADDR_OFFSET: u8 = 17;
    const RD_WR_OFFSET: u8 = 16;
    const I2C_REG_ADDR_OFFSET: u8 = 8;
    const I2C_REG_VAL_OFFSET: u8 = 0;

    const BUSY_MASK: u32 = 0b1;
    const DO_CMD_MASK: u32 = 0b1;
    const I2C_ADDR_MASK: u32 = 0x7f;
    const RD_WR_MASK: u32 = 0b1;
    const I2C_REG_ADDR_MASK: u32 = 0xff;
    const I2C_REG_VAL_MASK: u32 = 0xff;

    /// ## Command reading register `reg` of the I2C device at `i2c_addr` (7 bits)
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{I2CControl, Register};
    ///
    /// assert_eq!(I2CControl::read(0x4C, 0x01).val(), 0x0198_0100);
    /// ```
    pub const fn read(i2c_addr: u8, reg: u8) -> Self {
        Self(
            (Self::DO_CMD_MASK << Self::DO_CMD_OFFSET)
                | ((i2c_addr as u32 & Self::I2C_ADDR_MASK) << Self::I2C_ADDR_OFFSET)
                | ((reg as u32) << Self::I2C_REG_ADDR_OFFSET),
        )
    }

    /// ## Command writing `val` to register `reg` of the I2C device at `i2c_addr` (7 bits)
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{I2CControl, Register};
    ///
    /// assert_eq!(I2CControl::write(0x4C, 0x09, 0x04).val(), 0x0199_0904);
    /// ```
    pub const fn write(i2c_addr: u8, reg: u8, val: u8) -> Self {
        Self(
            Self::read(i2c_addr, reg).0
                | (Self::RD_WR_MASK << Self::RD_WR_OFFSET)
                | ((val as u32) << Self::I2C_REG_VAL_OFFSET),
        )
    }

    /// ## Handle the Busy field.
    ///
    /// Set while a transaction is in progress.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::I2CControl;
    ///
    /// assert!(I2CControl(0x8198_FE00).busy());
    /// assert!(!I2CControl(0x0198_FE55).busy());
    /// ```
    pub const fn busy(&self) -> bool {
        (self.0 >> Self::BUSY_OFFSET) & Self::BUSY_MASK == Self::BUSY_MASK
    }

    /// ## Handle the I2C Address field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::I2CControl;
    ///
    /// assert_eq!(I2CControl(0x0198_FE55).i2c_addr(), 0x4C);
    /// ```
    pub const fn i2c_addr(&self) -> u8 {
        ((self.0 >> Self::I2C_ADDR_OFFSET) & Self::I2C_ADDR_MASK) as u8
    }

    /// ## Handle the Read/Write field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::I2CControl;
    ///
    /// assert!(I2CControl(0x0199_0904).is_write());
    /// assert!(!I2CControl(0x0198_0100).is_write());
    /// ```
    pub const fn is_write(&self) -> bool {
        (self.0 >> Self::RD_WR_OFFSET) & Self::RD_WR_MASK == Self::RD_WR_MASK
    }

    /// ## Handle the Register Address field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::I2CControl;
    ///
    /// assert_eq!(I2CControl(0x0198_FE55).reg_addr(), 0xFE);
    /// ```
    pub const fn reg_addr(&self) -> u8 {
        ((self.0 >> Self::I2C_REG_ADDR_OFFSET) & Self::I2C_REG_ADDR_MASK) as u8
    }

    /// ## Handle the Register Value field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::I2CControl;
    ///
    /// assert_eq!(I2CControl(0x0198_FE55).reg_val(), 0x55);
    /// ```
    pub const fn reg_val(&self) -> u8 {
        ((self.0 >> Self::I2C_REG_VAL_OFFSET) & Self::I2C_REG_VAL_MASK) as u8
    }
}

impl core::fmt::Display for I2CControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("I2CControl")
            .field("busy", &self.busy())
            .field("i2c_addr", &self.i2c_addr())
            .field("is_write", &self.is_write())
            .field("reg_addr", &self.reg_addr())
            .field("reg_val", &self.reg_val())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for I2CControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "I2CControl {{ busy: {}, i2c_addr: {}, is_write: {}, reg_addr: {}, reg_val: {} }}",
            self.busy(),
            self.i2c_addr(),
            self.is_write(),
            self.reg_addr(),
            self.reg_val(),
        );
    }
}
