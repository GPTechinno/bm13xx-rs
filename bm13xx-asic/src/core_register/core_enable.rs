use crate::core_register::CoreRegister;

/// # Core Enable core register
///
/// Core register 4 of the older numbering (BM1397). On the BM1368 and later the core enable is
/// [`CoreEnableV2`](crate::core_register::CoreEnableV2), core register 2, and core register 4 is
/// [`HashClockCounterV2`](crate::core_register::HashClockCounterV2).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreEnable(pub u8);
impl_boilerplate_for_core_reg!(CoreEnable);

impl CoreEnable {
    pub const ID: u8 = 4;

    // const CORE_EN_I_OFFSET: u8 = 0;

    // const CORE_EN_I_MASK: u8 = 0xff;
}

impl ::core::fmt::Display for CoreEnable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreEnable").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreEnable {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreEnable {{ }}",);
    }
}

/// # Core Enable core register (`CORE_EN`)
///
/// Core register numbering of the BM1368 and later.
///
/// Core register 2 on the BM1368 and later; `CoreEnable` is core register 4 of the older chips.
///
/// Bits 15:8 (`CORE_EN_N`) lie beyond the 8 bits a core register write carries.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreEnableV2(pub u8);
impl_boilerplate_for_core_reg!(CoreEnableV2);

impl CoreEnableV2 {
    pub const ID: u8 = 2;

    const CORE_EN_I_OFFSET: u8 = 0;

    const CORE_EN_I_MASK: u8 = 0b11111111;

    /// ## Handle the `CORE_EN_I` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::CoreEnableV2;
    ///
    /// let mut reg = CoreEnableV2(0x00); // default value
    /// assert_eq!(reg.core_en_i(), 0x00);
    /// assert_eq!(reg.set_core_en_i(0xaa).core_en_i(), 0xaa);
    /// ```
    pub const fn core_en_i(&self) -> u8 {
        (self.0 >> Self::CORE_EN_I_OFFSET) & Self::CORE_EN_I_MASK
    }
    pub fn set_core_en_i(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::CORE_EN_I_MASK << Self::CORE_EN_I_OFFSET);
        self.0 |= (value & Self::CORE_EN_I_MASK) << Self::CORE_EN_I_OFFSET;
        self
    }
}

impl ::core::fmt::Display for CoreEnableV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreEnableV2")
            .field("core_en_i", &self.core_en_i())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreEnableV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreEnableV2 {{ core_en_i: {} }}", self.core_en_i(),);
    }
}
