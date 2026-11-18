use crate::core_register::CoreRegister;

/// # PG Monitor core register (`PG_MONT`)
///
/// Core register numbering of the BM1368 and later.
///
/// Bits 15:8 (`PGM_SEL`, `PGM_OVERLAP_AUX`, `PGM_OVERLAP`) lie beyond the 8 bits a core register write carries.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PgMont(pub u8);
impl_boilerplate_for_core_reg!(PgMont);

impl PgMont {
    pub const ID: u8 = 6;

    const PGM_ENABLE_OFFSET: u8 = 0;
    const PGM_UPLOAD_OFFSET: u8 = 1;
    const PGM_ADD_OFFSET: u8 = 2;
    const PGM_CTRL_OFFSET: u8 = 3;
    const PGM_AUTO_OFFSET: u8 = 7;

    const PGM_ENABLE_MASK: u8 = 0b1;
    const PGM_UPLOAD_MASK: u8 = 0b1;
    const PGM_ADD_MASK: u8 = 0b1;
    const PGM_CTRL_MASK: u8 = 0b1111;
    const PGM_AUTO_MASK: u8 = 0b1;

    /// ## Handle the `PGM_ENABLE` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::PgMont;
    ///
    /// let mut reg = PgMont(0x08); // default value
    /// assert_eq!(reg.pgm_enable(), false);
    /// assert!(reg.set_pgm_enable(true).pgm_enable());
    /// assert!(!reg.set_pgm_enable(false).pgm_enable());
    /// ```
    pub const fn pgm_enable(&self) -> bool {
        (self.0 >> Self::PGM_ENABLE_OFFSET) & Self::PGM_ENABLE_MASK != 0
    }
    pub fn set_pgm_enable(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::PGM_ENABLE_MASK << Self::PGM_ENABLE_OFFSET);
        self.0 |= (value as u8) << Self::PGM_ENABLE_OFFSET;
        self
    }

    /// ## Handle the `PGM_UPLOAD` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::PgMont;
    ///
    /// let mut reg = PgMont(0x08); // default value
    /// assert_eq!(reg.pgm_upload(), false);
    /// assert!(reg.set_pgm_upload(true).pgm_upload());
    /// assert!(!reg.set_pgm_upload(false).pgm_upload());
    /// ```
    pub const fn pgm_upload(&self) -> bool {
        (self.0 >> Self::PGM_UPLOAD_OFFSET) & Self::PGM_UPLOAD_MASK != 0
    }
    pub fn set_pgm_upload(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::PGM_UPLOAD_MASK << Self::PGM_UPLOAD_OFFSET);
        self.0 |= (value as u8) << Self::PGM_UPLOAD_OFFSET;
        self
    }

    /// ## Handle the `PGM_ADD` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::PgMont;
    ///
    /// let mut reg = PgMont(0x08); // default value
    /// assert_eq!(reg.pgm_add(), false);
    /// assert!(reg.set_pgm_add(true).pgm_add());
    /// assert!(!reg.set_pgm_add(false).pgm_add());
    /// ```
    pub const fn pgm_add(&self) -> bool {
        (self.0 >> Self::PGM_ADD_OFFSET) & Self::PGM_ADD_MASK != 0
    }
    pub fn set_pgm_add(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::PGM_ADD_MASK << Self::PGM_ADD_OFFSET);
        self.0 |= (value as u8) << Self::PGM_ADD_OFFSET;
        self
    }

    /// ## Handle the `PGM_CTRL` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::PgMont;
    ///
    /// let mut reg = PgMont(0x08); // default value
    /// assert_eq!(reg.pgm_ctrl(), 0x01);
    /// assert_eq!(reg.set_pgm_ctrl(0x0f).pgm_ctrl(), 0x0f);
    /// assert_eq!(reg.set_pgm_ctrl(0x10).pgm_ctrl(), 0x00); // out of bound value
    /// ```
    pub const fn pgm_ctrl(&self) -> u8 {
        (self.0 >> Self::PGM_CTRL_OFFSET) & Self::PGM_CTRL_MASK
    }
    pub fn set_pgm_ctrl(&mut self, value: u8) -> &mut Self {
        self.0 &= !(Self::PGM_CTRL_MASK << Self::PGM_CTRL_OFFSET);
        self.0 |= (value & Self::PGM_CTRL_MASK) << Self::PGM_CTRL_OFFSET;
        self
    }

    /// ## Handle the `PGM_AUTO` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::PgMont;
    ///
    /// let mut reg = PgMont(0x08); // default value
    /// assert_eq!(reg.pgm_auto(), false);
    /// assert!(reg.set_pgm_auto(true).pgm_auto());
    /// assert!(!reg.set_pgm_auto(false).pgm_auto());
    /// ```
    pub const fn pgm_auto(&self) -> bool {
        (self.0 >> Self::PGM_AUTO_OFFSET) & Self::PGM_AUTO_MASK != 0
    }
    pub fn set_pgm_auto(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::PGM_AUTO_MASK << Self::PGM_AUTO_OFFSET);
        self.0 |= (value as u8) << Self::PGM_AUTO_OFFSET;
        self
    }
}

impl ::core::fmt::Display for PgMont {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("PgMont")
            .field("pgm_enable", &self.pgm_enable())
            .field("pgm_upload", &self.pgm_upload())
            .field("pgm_add", &self.pgm_add())
            .field("pgm_ctrl", &self.pgm_ctrl())
            .field("pgm_auto", &self.pgm_auto())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PgMont {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PgMont {{ pgm_enable: {}, pgm_upload: {}, pgm_add: {}, pgm_ctrl: {}, pgm_auto: {} }}",
            self.pgm_enable(),
            self.pgm_upload(),
            self.pgm_add(),
            self.pgm_ctrl(),
            self.pgm_auto(),
        );
    }
}
