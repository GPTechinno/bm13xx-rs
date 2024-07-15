use crate::{core_register::CoreRegister, register::Register};

/// # Core Register Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreRegisterControl(pub u32);
impl_boilerplate_for!(CoreRegisterControl);

impl CoreRegisterControl {
    pub const ADDR: u8 = 0x3C;

    const DO_CMD_OFFSET: u8 = 31;
    const CORE_ID_OFFSET: u8 = 16;
    const RD_WR_OFFSET: u8 = 15;
    const CORE_REG_ID_OFFSET: u8 = 8;
    const CORE_REG_VAL_OFFSET: u8 = 0;

    const DO_CMD_MASK: u32 = 0b1;
    const RD_WR_MASK: u32 = 0b1;
    const CORE_ID_MASK: u32 = 0x1ff;
    const CORE_REG_ID_MASK: u32 = 0x1f;
    const CORE_REG_VAL_MASK: u32 = 0xff;

    /// ## Set CoreRegisterControl for a Core Register Read.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{CoreRegisterControl, Register};
    /// use bm13xx_asic::core_register::{ClockDelayCtrl};
    ///
    /// assert_eq!(CoreRegisterControl::read_core_reg(0, ClockDelayCtrl(0x74)), 0x8000_00ff);
    /// ```
    pub fn read_core_reg(core_id: u8, core_reg: impl CoreRegister) -> u32 {
        (Self::DO_CMD_MASK << Self::DO_CMD_OFFSET)
            | (((core_id as u32) & Self::CORE_ID_MASK) << Self::CORE_ID_OFFSET)
            | (((core_reg.id() as u32) & Self::CORE_REG_ID_MASK) << Self::CORE_REG_ID_OFFSET)
            | Self::CORE_REG_VAL_MASK
    }
    /// ## Set CoreRegisterControl for a Core Register Write.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{CoreRegisterControl, Register};
    /// use bm13xx_asic::core_register::{ClockDelayCtrl};
    ///
    /// assert_eq!(CoreRegisterControl::write_core_reg(0, ClockDelayCtrl(0x74)), 0x8000_8074);
    pub fn write_core_reg(core_id: u8, core_reg: impl CoreRegister) -> u32 {
        (Self::DO_CMD_MASK << Self::DO_CMD_OFFSET)
            | (Self::RD_WR_MASK << Self::RD_WR_OFFSET)
            | (((core_id as u32) & Self::CORE_ID_MASK) << Self::CORE_ID_OFFSET)
            | (((core_reg.id() as u32) & Self::CORE_REG_ID_MASK) << Self::CORE_REG_ID_OFFSET)
            | (((core_reg.val() as u32) & Self::CORE_REG_VAL_MASK) << Self::CORE_REG_VAL_OFFSET)
    }
}

impl core::fmt::Display for CoreRegisterControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CoreRegisterControl").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreRegisterControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreRegisterControl {{  }}",);
    }
}

/// # Core Register Value register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreRegisterValue(pub u32);
impl_boilerplate_for!(CoreRegisterValue);

impl CoreRegisterValue {
    pub const ADDR: u8 = 0x40;

    const CORE_ID_OFFSET: u8 = 16;
    const FOUND_OFFSET: u8 = 8;
    const CORE_REG_VAL_OFFSET: u8 = 0;

    const CORE_ID_MASK: u32 = 0x1ff;
    const FOUND_MASK: u32 = 0xff;
    const CORE_REG_VAL_MASK: u32 = 0xff;

    /// ## Get the CORE_ID.
    ///
    /// This returns an `u16` with the CORE_ID value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::CoreRegisterValue;
    ///
    /// let crv: CoreRegisterValue = CoreRegisterValue(0x0001_1234);
    /// assert_eq!(crv.core_id(), 0x0001);
    /// ```
    pub const fn core_id(&self) -> u16 {
        ((self.0 >> Self::CORE_ID_OFFSET) & Self::CORE_ID_MASK) as u16
    }

    /// ## Get the FOUND.
    ///
    /// This returns an `u8` with the FOUND value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::CoreRegisterValue;
    ///
    /// let crv: CoreRegisterValue = CoreRegisterValue(0x0001_1234);
    /// assert_eq!(crv.found(), 0x12);
    /// ```
    pub const fn found(&self) -> u8 {
        ((self.0 >> Self::FOUND_OFFSET) & Self::FOUND_MASK) as u8
    }

    /// ## Get the CORE_REG_VAL.
    ///
    /// This returns an `u8` with the CORE_REG_VAL value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::CoreRegisterValue;
    ///
    /// let crv: CoreRegisterValue = CoreRegisterValue(0x0001_1234);
    /// assert_eq!(crv.core_reg_val(), 0x34);
    /// ```
    pub const fn core_reg_val(&self) -> u8 {
        ((self.0 >> Self::CORE_REG_VAL_OFFSET) & Self::CORE_REG_VAL_MASK) as u8
    }
}

impl core::fmt::Display for CoreRegisterValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CoreRegisterValue").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreRegisterValue {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreRegisterValue {{  }}",);
    }
}
