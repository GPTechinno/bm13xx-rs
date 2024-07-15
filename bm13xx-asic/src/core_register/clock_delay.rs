use crate::core_register::CoreRegister;

/// # Clock Delay Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClockDelayCtrl(pub u8);
impl_boilerplate_for_core_reg!(ClockDelayCtrl);

impl ClockDelayCtrl {
    pub const ID: u8 = 0x00;

    const CCDLY_SEL_OFFSET: u8 = 6;
    const PWTH_SEL_OFFSET: u8 = 4;
    const HASH_CLKEN_OFFSET: u8 = 3;
    const MMEN_OFFSET: u8 = 2;
    const SWPF_MODE_OFFSET: u8 = 0;

    const CCDLY_SEL_MASK: u8 = 0b11;
    const PWTH_SEL_MASK: u8 = 0b11;
    const HASH_CLKEN_MASK: u8 = 0b1;
    const MMEN_MASK: u8 = 0b1;
    const SWPF_MODE_MASK: u8 = 0b1;

    /// ## Get the CCdly value.
    ///
    /// This returns an `u8` with the CCdly value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// assert_eq!(ClockDelayCtrl(0x00).ccdly(), 0x00);
    /// assert_eq!(ClockDelayCtrl(0x74).ccdly(), 0x01); // value sent in BM1397 init()
    /// ```
    pub const fn ccdly(&self) -> u8 {
        (self.0 >> Self::CCDLY_SEL_OFFSET) & Self::CCDLY_SEL_MASK
    }

    /// ## Get the PWth value.
    ///
    /// This returns an `u8` with the PWth value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// assert_eq!(ClockDelayCtrl(0x00).pwth(), 0x00);
    /// assert_eq!(ClockDelayCtrl(0x74).pwth(), 0x03); // value sent in BM1397 init()
    /// ```
    pub const fn pwth(&self) -> u8 {
        (self.0 >> Self::PWTH_SEL_OFFSET) & Self::PWTH_SEL_MASK
    }

    /// ## Get the Hash Clock state.
    ///
    /// This returns an `bool` with the Hash Clock state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// assert!(!ClockDelayCtrl(0x00).hash_clock_enabled());
    /// assert!(!ClockDelayCtrl(0x74).hash_clock_enabled()); // value sent in BM1397 init()
    /// ```
    pub const fn hash_clock_enabled(&self) -> bool {
        (self.0 >> Self::HASH_CLKEN_OFFSET) & Self::HASH_CLKEN_MASK == Self::HASH_CLKEN_MASK
    }

    /// ## Get the Multi Midstate state.
    ///
    /// This returns an `bool` with the Multi Midstate state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// assert!(!ClockDelayCtrl(0x00).multi_midstate_enabled());
    /// assert!(ClockDelayCtrl(0x74).multi_midstate_enabled()); // value sent in BM1397 init()
    /// ```
    pub const fn multi_midstate_enabled(&self) -> bool {
        (self.0 >> Self::MMEN_OFFSET) & Self::MMEN_MASK == Self::MMEN_MASK
    }

    /// ## Get the Sweep Frequency Mode state.
    ///
    /// This returns an `bool` with the Sweep Frequency Mode state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// assert!(!ClockDelayCtrl(0x00).sweep_frequency_mode_enabled());
    /// assert!(!ClockDelayCtrl(0x74).sweep_frequency_mode_enabled()); // value sent in BM1397 init()
    /// ```
    pub const fn sweep_frequency_mode_enabled(&self) -> bool {
        (self.0 >> Self::SWPF_MODE_OFFSET) & Self::SWPF_MODE_MASK == Self::SWPF_MODE_MASK
    }
}

impl ::core::fmt::Display for ClockDelayCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockDelayCtrl")
            .field("ccdly", &self.ccdly())
            .field("pwth", &self.pwth())
            .field("hash_clock_enabled", &self.hash_clock_enabled())
            .field("multi_midstate_enabled", &self.multi_midstate_enabled())
            .field(
                "sweep_frequency_mode_enabled",
                &self.sweep_frequency_mode_enabled(),
            )
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockDelayCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockDelayCtrl {{ ccdly: {}, pwth: {}, hash_clock_enabled: {}, multi_midstate_enabled: {}, sweep_frequency_mode_enabled: {} }}",
            self.ccdly(),
            self.pwth(),
            self.hash_clock_enabled(),
            self.multi_midstate_enabled(),
            self.sweep_frequency_mode_enabled(),
        );
    }
}
