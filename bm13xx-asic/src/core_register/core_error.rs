use crate::core_register::CoreRegister;

/// # Core Error core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CoreError(pub u8);
impl_boilerplate_for_core_reg!(CoreError);

impl CoreError {
    pub const ID: u8 = 0x03;

    const INI_NONCE_ERR_OFFSET: u8 = 4;
    const CMD_ERR_CNT_OFFSET: u8 = 0;

    const INI_NONCE_ERR_MASK: u8 = 0b1;
    const CMD_ERR_CNT_MASK: u8 = 0b1111;

    /// ## Get the Ini Nonce Error state.
    ///
    /// This returns an `bool` with the Ini Nonce Error state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::CoreError;
    ///
    /// assert!(!CoreError(0x00).ini_nonce_err());
    /// ```
    pub const fn ini_nonce_err(&self) -> bool {
        (self.0 >> Self::INI_NONCE_ERR_OFFSET) & Self::INI_NONCE_ERR_MASK
            == Self::INI_NONCE_ERR_MASK
    }

    /// ## Get the Command Error Count.
    ///
    /// This returns an `u8` with the Command Error Count.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::CoreError;
    ///
    /// assert_eq!(CoreError(0x00).cmd_err_cnt(), 0x00);
    /// ```
    pub const fn cmd_err_cnt(&self) -> u8 {
        (self.0 >> Self::CMD_ERR_CNT_OFFSET) & Self::CMD_ERR_CNT_MASK
    }
}

impl ::core::fmt::Display for CoreError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("CoreError")
            .field("ini_nonce_err", &self.ini_nonce_err())
            .field("cmd_err_cnt", &self.cmd_err_cnt())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CoreError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "CoreError {{ ini_nonce_err: {}, cmd_err_cnt: {} }}",
            self.ini_nonce_err(),
            self.cmd_err_cnt()
        );
    }
}
