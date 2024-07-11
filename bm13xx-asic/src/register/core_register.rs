use crate::register::Register;

/// # Core Register Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreRegisterControl(pub u32);
impl_boilerplate_for!(CoreRegisterControl);

impl CoreRegisterControl {
    pub const ADDR: u8 = 0x3C;

    // const RD_WR1_OFFSET: u8 = 31;
    // const CORE_ID_OFFSET: u8 = 16;
    // const RD_WR2_OFFSET: u8 = 15;
    // const CORE_REG_ID_OFFSET: u8 = 8;
    // const CORE_REG_VAL_OFFSET: u8 = 0;

    // const RD_WR_MASK: u32 = 0b1;
    // const CORE_ID_MASK: u32 = 0xff;
    // const CORE_REG_ID_MASK: u32 = 0b1111;
    // const CORE_REG_VAL_MASK: u32 = 0xff;

    /* TODO
    /// ## Set CoreRegisterControl for a Core Register Read.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{CoreRegisterControl, Register};
    /// use bm13xx_asic::core_register::{ClockDelayCtrl};
    ///
    /// let crc: CoreRegisterControl = CoreRegisterControl::DEFAULT;
    /// assert_eq!(crc.val(), 0x0000_0000);
    /// let cdc: ClockDelayCtrl = ClockDelayCtrl::default();
    /// let crc: CoreRegisterControl = crc.read(0, cdc);
    /// assert_eq!(crc.val(), 0x0000_00ff);
    /// let cdc: ClockDelayCtrl = cdc.enable_multi_midstate();
    /// let crc: CoreRegisterControl = crc.write(0, cdc);
    /// assert_eq!(crc.val(), 0x8000_8004);
    /// ```
    #[must_use = "read returns a modified CoreRegisterControl"]
    pub fn read(mut self, core_id: u8, core_reg: impl CoreRegister) -> Self {
        self.0 &= !Self::RD_WR_MASK;
        self.0 &= !Self::CORE_ID_MASK;
        self.0 |= ((core_id as u32) << Self::CORE_ID_OFFSET) & Self::CORE_ID_MASK;
        self.0 &= !Self::CORE_REG_ID_MASK;
        self.0 |= ((core_reg.id() as u32) << Self::CORE_REG_ID_OFFSET) & Self::CORE_REG_ID_MASK;
        self.0 |= Self::CORE_REG_VAL_MASK;
        self
    }
    /// ## Set CoreRegisterControl for a Core Register Write.
    #[must_use = "write returns a modified CoreRegisterControl"]
    pub fn write(mut self, core_id: u8, core_reg: impl CoreRegister) -> Self {
        self.0 |= Self::RD_WR_MASK;
        self.0 &= !Self::CORE_ID_MASK;
        self.0 |= ((core_id as u32) << Self::CORE_ID_OFFSET) & Self::CORE_ID_MASK;
        self.0 &= !Self::CORE_REG_ID_MASK;
        self.0 |= ((core_reg.id() as u32) << Self::CORE_REG_ID_OFFSET) & Self::CORE_REG_ID_MASK;
        self.0 &= !Self::CORE_REG_VAL_MASK;
        self.0 |= ((core_reg.val() as u32) << Self::CORE_REG_VAL_OFFSET) & Self::CORE_REG_VAL_MASK;
        self
    } */
}

impl ::core::fmt::Display for CoreRegisterControl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
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

    /* TODO
    /// ## Get the CoreRegister according to the given core_reg_id
    /// and the current CORE_REG_VAL.
    ///
    /// ## Return
    /// - `Ok(CoreRegisters)` with the corresponding `CoreRegister`.
    /// - `Err(Error::UnknownCoreRegister(u8))` with the core register id
    ///    if it do not match a known `CoreRegisters`.
    ///
    /// ### Examples
    /// ```
    /// use bm13xx_asic::core_register::{ProcessMonitorData, CoreRegisters};
    /// use bm13xx_asic::Error;
    /// use bm13xx_asic::register::CoreRegisterValue;
    ///
    /// let crv: CoreRegisterValue = CoreRegisterValue(0x0001_0234);
    /// // ProcessMonitorData
    /// let resp = crv.core_reg(0x02);
    /// assert!(resp.is_ok());
    /// assert_eq!(resp.unwrap(), CoreRegisters::ProcessMonitorData(ProcessMonitorData(0x34)));
    ///
    /// // Error::UnknownCoreRegister(0xF0)
    /// let resp = crv.core_reg(0xF0);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::UnknownCoreRegister(0xF0));
    /// ```
    pub fn core_reg(&self, core_reg_id: u8) -> Result<CoreRegisters, Error> {
        let core_reg = match core_reg_id {
            ClockDelayCtrl::ID => {
                CoreRegisters::ClockDelayCtrl(ClockDelayCtrl(self.core_reg_val()))
            }
            ProcessMonitorCtrl::ID => {
                CoreRegisters::ProcessMonitorCtrl(ProcessMonitorCtrl(self.core_reg_val()))
            }
            ProcessMonitorData::ID => {
                CoreRegisters::ProcessMonitorData(ProcessMonitorData(self.core_reg_val()))
            }
            CoreError::ID => CoreRegisters::CoreError(CoreError(self.core_reg_val())),
            CoreEnable::ID => CoreRegisters::CoreEnable(CoreEnable(self.core_reg_val())),
            HashClockCtrl::ID => CoreRegisters::HashClockCtrl(HashClockCtrl(self.core_reg_val())),
            HashClockCounter::ID => {
                CoreRegisters::HashClockCounter(HashClockCounter(self.core_reg_val()))
            }
            SweepClockCtrl::ID => {
                CoreRegisters::SweepClockCtrl(SweepClockCtrl(self.core_reg_val()))
            }
            id => return Err(Error::UnknownCoreRegister(id)),
        };
        Ok(core_reg)
    } */
}

impl ::core::fmt::Display for CoreRegisterValue {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreRegisterValue").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreRegisterValue {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreRegisterValue {{  }}",);
    }
}
