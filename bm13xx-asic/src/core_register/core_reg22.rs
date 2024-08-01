use crate::core_register::CoreRegister;

/// # Hash Clock Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreReg22(pub u8);
impl_boilerplate_for_core_reg!(CoreReg22);

impl CoreReg22 {
    pub const ID: u8 = 22;

    const EN_OFFSET: u8 = 6;

    const EN_MASK: u8 = 0b1;

    /// ## Handle the enable field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::CoreReg22;
    ///
    /// let mut core_reg_22 = CoreReg22(0x00); // BM1366 default value
    /// assert!(!core_reg_22.enabled());
    /// assert!(core_reg_22.enable().enabled());
    /// assert!(!core_reg_22.disable().enabled());
    /// ```
    pub const fn enabled(&self) -> bool {
        (self.0 >> Self::EN_OFFSET) & Self::EN_MASK != 0
    }
    pub fn enable(&mut self) -> &mut Self {
        self.0 |= Self::EN_MASK << Self::EN_OFFSET;
        self
    }
    pub fn disable(&mut self) -> &mut Self {
        self.0 &= !(Self::EN_MASK << Self::EN_OFFSET);
        self
    }
}

impl ::core::fmt::Display for CoreReg22 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreReg22")
            .field("enabled", &self.enabled())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for CoreReg22 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "CoreReg22 {{ enabled: {} }}", self.enabled(),);
    }
}
