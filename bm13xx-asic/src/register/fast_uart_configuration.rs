use crate::register::Register;

/// # Fast UART Configuration register
///
/// Used to configure UART settings.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FastUARTConfiguration(pub u32);
impl_boilerplate_for!(FastUARTConfiguration);

impl FastUARTConfiguration {
    pub const ADDR: u8 = 0x28;

    // const DIV4_ODDSET_OFFSET: u8 = 30;
    const PLL3_DIV4_OFFSET: u8 = 24;
    // const USRC_ODDSET_OFFSET: u8 = 22;
    // const USRC_DIV_OFFSET: u8 = 16;
    // const FORCE_CORE_EN_OFFSET: u8 = 15;
    // const CLKO_SEL_OFFSET: u8 = 14;
    // const CLKO_ODDSET_OFFSET: u8 = 12;
    // const CLKO_DIV_OFFSET: u8 = 0;

    // const DIV4_ODDSET_MASK: u32 = 0b11;
    const PLL3_DIV4_MASK: u32 = 0b1111;
    // const USRC_ODDSET_MASK: u32 = 0b11;
    // const USRC_DIV_MASK: u32 = 0x3f;
    // const FORCE_CORE_EN_MASK: u32 = 0b1;
    // const CLKO_SEL_MASK: u32 = 0b1;
    // const CLKO_ODDSET_MASK: u32 = 0b11;
    // const CLKO_DIV_MASK: u32 = 0xff;

    /// ## Get the PLL3_DIV4.
    ///
    /// This returns an `u8` with the PLL3_DIV4 value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::FastUARTConfiguration;
    ///
    /// let uart_conf = FastUARTConfiguration(0x0600_000F);
    /// assert_eq!(uart_conf.pll3_div4(), 0x06);
    /// ```
    pub const fn pll3_div4(&self) -> u8 {
        ((self.0 >> Self::PLL3_DIV4_OFFSET) & Self::PLL3_DIV4_MASK) as u8
    }
}

impl core::fmt::Display for FastUARTConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FastUARTConfiguration")
            .field("pll3_div4", &self.pll3_div4())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for FastUARTConfiguration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "FastUARTConfiguration {{ pll3_div4: {} }}",
            self.pll3_div4(),
        );
    }
}
