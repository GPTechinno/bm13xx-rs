use crate::register::Register;

/// # Ring Oscillator Pad Disable register
///
/// BM1368 and later. On the BM1397, `0x68` is [`PLL3Parameter`](crate::register::PLL3Parameter).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RoscPadDisable(pub u32);
impl_boilerplate_for!(RoscPadDisable);

impl RoscPadDisable {
    pub const ADDR: u8 = 0x68;

    /// Value that disables the ring oscillator pads.
    pub const DISABLE: u32 = 0x5AA5_5AA5;

    /// ## Handle the `ROSC_DIS_DATA` field.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::RoscPadDisable;
    ///
    /// let mut rosc = RoscPadDisable(0x0000_0000); // default value
    /// assert_eq!(rosc.data(), 0);
    /// assert!(!rosc.disabled());
    /// assert_eq!(rosc.disable().data(), RoscPadDisable::DISABLE);
    /// assert!(rosc.disabled());
    /// assert_eq!(rosc.set_data(0x1234_5678).data(), 0x1234_5678);
    /// assert!(!rosc.disabled());
    /// ```
    pub const fn data(&self) -> u32 {
        self.0
    }
    pub fn set_data(&mut self, data: u32) -> &mut Self {
        self.0 = data;
        self
    }
    pub const fn disabled(&self) -> bool {
        self.0 == Self::DISABLE
    }
    pub fn disable(&mut self) -> &mut Self {
        self.0 = Self::DISABLE;
        self
    }
}

impl core::fmt::Display for RoscPadDisable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RoscPadDisable")
            .field("disabled", &self.disabled())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RoscPadDisable {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "RoscPadDisable {{ disabled: {} }}", self.disabled());
    }
}
