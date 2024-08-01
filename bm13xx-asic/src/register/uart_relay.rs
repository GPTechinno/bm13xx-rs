use crate::register::Register;

/// # UART Relay register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct UARTRelay(pub u32);
impl_boilerplate_for!(UARTRelay);

impl UARTRelay {
    pub const ADDR: u8 = 0x2C;

    const GAP_CNT_OFFSET: u8 = 16;
    const RO_REL_EN_OFFSET: u8 = 1;
    const CO_REL_EN_OFFSET: u8 = 0;

    const GAP_CNT_MASK: u32 = 0xffff;
    const RO_REL_EN_MASK: u32 = 0b1;
    const CO_REL_EN_MASK: u32 = 0b1;

    /// ## Handle the GAP_CNT field.
    ///
    /// Get and set the GAP_CNT value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::UARTRelay;
    ///
    /// let mut uart_relay = UARTRelay(0x000f_0000); // BM1366 default value
    /// assert_eq!(uart_relay.gap_cnt(), 0x000f);
    /// assert_eq!(uart_relay.set_gap_cnt(0).gap_cnt(), 0); // min value
    /// assert_eq!(uart_relay.set_gap_cnt(0xffff).gap_cnt(), 0xffff); // max value
    /// ```
    pub const fn gap_cnt(&self) -> u16 {
        ((self.0 >> Self::GAP_CNT_OFFSET) & Self::GAP_CNT_MASK) as u16
    }
    pub fn set_gap_cnt(&mut self, gap_cnt: u16) -> &mut Self {
        self.0 &= !(Self::GAP_CNT_MASK << Self::GAP_CNT_OFFSET);
        self.0 |= ((gap_cnt as u32) & Self::GAP_CNT_MASK) << Self::GAP_CNT_OFFSET;
        self
    }

    /// ## Handle the RO_REL_EN field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::register::UARTRelay;
    ///
    /// let mut uart_relay = UARTRelay(0x000f_0000); // BM1366 default value
    /// assert!(!uart_relay.ro_relay_enabled());
    /// assert!(uart_relay.enable_ro_relay().ro_relay_enabled());
    /// assert!(!uart_relay.disable_ro_relay().ro_relay_enabled());
    /// ```
    pub const fn ro_relay_enabled(&self) -> bool {
        (self.0 >> Self::RO_REL_EN_OFFSET) & Self::RO_REL_EN_MASK != 0
    }
    pub fn enable_ro_relay(&mut self) -> &mut Self {
        self.0 |= Self::RO_REL_EN_MASK << Self::RO_REL_EN_OFFSET;
        self
    }
    pub fn disable_ro_relay(&mut self) -> &mut Self {
        self.0 &= !(Self::RO_REL_EN_MASK << Self::RO_REL_EN_OFFSET);
        self
    }

    /// ## Handle the CO_REL_EN field.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::register::UARTRelay;
    ///
    /// let mut uart_relay = UARTRelay(0x000f_0000); // BM1366 default value
    /// assert!(!uart_relay.co_relay_enabled());
    /// assert!(uart_relay.enable_co_relay().co_relay_enabled());
    /// assert!(!uart_relay.disable_co_relay().co_relay_enabled());
    /// ```
    pub const fn co_relay_enabled(&self) -> bool {
        (self.0 >> Self::CO_REL_EN_OFFSET) & Self::CO_REL_EN_MASK != 0
    }
    pub fn enable_co_relay(&mut self) -> &mut Self {
        self.0 |= Self::CO_REL_EN_MASK << Self::CO_REL_EN_OFFSET;
        self
    }
    pub fn disable_co_relay(&mut self) -> &mut Self {
        self.0 &= !(Self::CO_REL_EN_MASK << Self::CO_REL_EN_OFFSET);
        self
    }
}

impl core::fmt::Display for UARTRelay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UARTRelay").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for UARTRelay {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "UARTRelay {{  }}",);
    }
}
