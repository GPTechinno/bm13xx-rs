use crate::core_register::CoreRegister;

/// # Nonce Bin Overflow Control core register (`NONCE_BIN_OVERFLOW_CONTROL`)
///
/// Core register numbering of the BM1368 and later.
///
/// `NONCE_OF_STS` is a status bit.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NonceBinOverflowControl(pub u8);
impl_boilerplate_for_core_reg!(NonceBinOverflowControl);

impl NonceBinOverflowControl {
    pub const ID: u8 = 13;

    const NONCE_BIN_OF_ENABLE_OFFSET: u8 = 0;
    const GOLDEN_TICKET_EN_OFFSET: u8 = 2;
    const NONCE_OF_STS_OFFSET: u8 = 4;

    const NONCE_BIN_OF_ENABLE_MASK: u8 = 0b1;
    const GOLDEN_TICKET_EN_MASK: u8 = 0b1;
    const NONCE_OF_STS_MASK: u8 = 0b1;

    /// ## Handle the `NONCE_BIN_OF_ENABLE` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::NonceBinOverflowControl;
    ///
    /// let mut reg = NonceBinOverflowControl(0x05); // default value
    /// assert_eq!(reg.nonce_bin_of_enable(), true);
    /// assert!(reg.set_nonce_bin_of_enable(true).nonce_bin_of_enable());
    /// assert!(!reg.set_nonce_bin_of_enable(false).nonce_bin_of_enable());
    /// ```
    pub const fn nonce_bin_of_enable(&self) -> bool {
        (self.0 >> Self::NONCE_BIN_OF_ENABLE_OFFSET) & Self::NONCE_BIN_OF_ENABLE_MASK != 0
    }
    pub fn set_nonce_bin_of_enable(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::NONCE_BIN_OF_ENABLE_MASK << Self::NONCE_BIN_OF_ENABLE_OFFSET);
        self.0 |= (value as u8) << Self::NONCE_BIN_OF_ENABLE_OFFSET;
        self
    }

    /// ## Handle the `GOLDEN_TICKET_EN` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::NonceBinOverflowControl;
    ///
    /// let mut reg = NonceBinOverflowControl(0x05); // default value
    /// assert_eq!(reg.golden_ticket_en(), true);
    /// assert!(reg.set_golden_ticket_en(true).golden_ticket_en());
    /// assert!(!reg.set_golden_ticket_en(false).golden_ticket_en());
    /// ```
    pub const fn golden_ticket_en(&self) -> bool {
        (self.0 >> Self::GOLDEN_TICKET_EN_OFFSET) & Self::GOLDEN_TICKET_EN_MASK != 0
    }
    pub fn set_golden_ticket_en(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::GOLDEN_TICKET_EN_MASK << Self::GOLDEN_TICKET_EN_OFFSET);
        self.0 |= (value as u8) << Self::GOLDEN_TICKET_EN_OFFSET;
        self
    }

    /// ## Handle the `NONCE_OF_STS` field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::core_register::NonceBinOverflowControl;
    ///
    /// let mut reg = NonceBinOverflowControl(0x05); // default value
    /// assert_eq!(reg.nonce_of_sts(), false);
    /// assert!(reg.set_nonce_of_sts(true).nonce_of_sts());
    /// assert!(!reg.set_nonce_of_sts(false).nonce_of_sts());
    /// ```
    pub const fn nonce_of_sts(&self) -> bool {
        (self.0 >> Self::NONCE_OF_STS_OFFSET) & Self::NONCE_OF_STS_MASK != 0
    }
    pub fn set_nonce_of_sts(&mut self, value: bool) -> &mut Self {
        self.0 &= !(Self::NONCE_OF_STS_MASK << Self::NONCE_OF_STS_OFFSET);
        self.0 |= (value as u8) << Self::NONCE_OF_STS_OFFSET;
        self
    }
}

impl ::core::fmt::Display for NonceBinOverflowControl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("NonceBinOverflowControl")
            .field("nonce_bin_of_enable", &self.nonce_bin_of_enable())
            .field("golden_ticket_en", &self.golden_ticket_en())
            .field("nonce_of_sts", &self.nonce_of_sts())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for NonceBinOverflowControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "NonceBinOverflowControl {{ nonce_bin_of_enable: {}, golden_ticket_en: {}, nonce_of_sts: {} }}",
            self.nonce_bin_of_enable(),
            self.golden_ticket_en(),
            self.nonce_of_sts(),
        );
    }
}
