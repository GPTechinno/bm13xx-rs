use crate::register::Register;

/// # Misc Control Add register (BM1372)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MiscControlAdd(pub u32);
impl_boilerplate_for!(MiscControlAdd);

impl MiscControlAdd {
    pub const ADDR: u8 = 0x19;

    const RTST_M_EN_OFFSET: u8 = 8;
    const RTST_F_EN_OFFSET: u8 = 0;

    const RTST_M_EN_MASK: u32 = 0b11111111;
    const RTST_F_EN_MASK: u32 = 0b11111111;

    /// ## Handle the `RTST_M_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlAdd;
    ///
    /// let mut reg = MiscControlAdd(0x00000000); // reset value
    /// assert_eq!(reg.rtst_m_en(), 0x0);
    /// assert_eq!(reg.set_rtst_m_en(0xff).rtst_m_en(), 0xff); // max value
    /// ```
    pub const fn rtst_m_en(&self) -> u8 {
        ((self.0 >> Self::RTST_M_EN_OFFSET) & Self::RTST_M_EN_MASK) as u8
    }
    pub fn set_rtst_m_en(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::RTST_M_EN_MASK << Self::RTST_M_EN_OFFSET);
        self.0 |= ((value as u32) & Self::RTST_M_EN_MASK) << Self::RTST_M_EN_OFFSET;
        self
    }

    /// ## Handle the `RTST_F_EN` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::MiscControlAdd;
    ///
    /// let mut reg = MiscControlAdd(0x00000000); // reset value
    /// assert_eq!(reg.rtst_f_en(), 0x0);
    /// assert_eq!(reg.set_rtst_f_en(0xff).rtst_f_en(), 0xff); // max value
    /// ```
    pub const fn rtst_f_en(&self) -> u8 {
        ((self.0 >> Self::RTST_F_EN_OFFSET) & Self::RTST_F_EN_MASK) as u8
    }
    pub fn set_rtst_f_en(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::RTST_F_EN_MASK << Self::RTST_F_EN_OFFSET);
        self.0 |= ((value as u32) & Self::RTST_F_EN_MASK) << Self::RTST_F_EN_OFFSET;
        self
    }
}

impl core::fmt::Display for MiscControlAdd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MiscControlAdd")
            .field("rtst_m_en", &self.rtst_m_en())
            .field("rtst_f_en", &self.rtst_f_en())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MiscControlAdd {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "MiscControlAdd {{ rtst_m_en: {}, rtst_f_en: {} }}",
            self.rtst_m_en(),
            self.rtst_f_en(),
        );
    }
}
