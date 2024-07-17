use crate::core_register::CoreRegister;

/// # Clock Delay Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClockDelayCtrl(pub u8);
impl_boilerplate_for_core_reg!(ClockDelayCtrl);

impl ClockDelayCtrl {
    pub const ID: u8 = 0;

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

    /// ## Handle the CCdly field.
    ///
    /// Get and set the CCdly value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// let mut cdc = ClockDelayCtrl(0x00); // BM1397 default value
    /// assert_eq!(cdc.ccdly(), 0);
    /// assert_eq!(cdc.set_ccdly(1).ccdly(), 1); // BM1397 init() value
    /// assert_eq!(cdc.set_ccdly(3).ccdly(), 3); // max value
    /// assert_eq!(cdc.set_ccdly(4).ccdly(), 0); // out of bound value
    /// ```
    pub const fn ccdly(&self) -> u8 {
        (self.0 >> Self::CCDLY_SEL_OFFSET) & Self::CCDLY_SEL_MASK
    }
    pub fn set_ccdly(&mut self, ccdly: u8) -> &mut Self {
        self.0 &= !Self::CCDLY_SEL_MASK << Self::CCDLY_SEL_OFFSET;
        self.0 |= (ccdly & Self::CCDLY_SEL_MASK) << Self::CCDLY_SEL_OFFSET;
        self
    }

    /// ## Handle the PWth field.
    ///
    /// Get and set the PWth value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// let mut cdc = ClockDelayCtrl(0x00); // BM1397 default value
    /// assert_eq!(cdc.pwth(), 0);
    /// assert_eq!(cdc.set_pwth(3).pwth(), 3); // BM1397 init() value
    /// assert_eq!(cdc.set_pwth(4).pwth(), 0); // out of bound value
    /// ```
    pub const fn pwth(&self) -> u8 {
        (self.0 >> Self::PWTH_SEL_OFFSET) & Self::PWTH_SEL_MASK
    }
    pub fn set_pwth(&mut self, pwth: u8) -> &mut Self {
        self.0 &= !Self::PWTH_SEL_MASK << Self::PWTH_SEL_OFFSET;
        self.0 |= (pwth & Self::PWTH_SEL_MASK) << Self::PWTH_SEL_OFFSET;
        self
    }

    /// ## Handle the Hash Clock field.
    ///
    /// Get and set the Hash Clock state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// let mut cdc = ClockDelayCtrl(0x00); // BM1397 default value
    /// assert!(!cdc.hash_clock_enabled());
    /// assert!(cdc.enable_hash_clock().hash_clock_enabled());
    /// assert!(!cdc.disable_hash_clock().hash_clock_enabled()); // BM1397 init() value
    /// ```
    pub const fn hash_clock_enabled(&self) -> bool {
        (self.0 >> Self::HASH_CLKEN_OFFSET) & Self::HASH_CLKEN_MASK == Self::HASH_CLKEN_MASK
    }
    pub fn enable_hash_clock(&mut self) -> &mut Self {
        self.0 |= 1 << Self::HASH_CLKEN_OFFSET;
        self
    }
    pub fn disable_hash_clock(&mut self) -> &mut Self {
        self.0 &= !(1 << Self::HASH_CLKEN_OFFSET);
        self
    }

    /// ## Handle the Multi Midstate field.
    ///
    /// Get and set the Multi Midstate state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// let mut cdc = ClockDelayCtrl(0x00); // BM1397 default value
    /// assert!(!cdc.multi_midstate_enabled());
    /// assert!(cdc.enable_multi_midstate().multi_midstate_enabled()); // BM1397 init() value
    /// assert!(!cdc.disable_multi_midstate().multi_midstate_enabled());
    /// ```
    pub const fn multi_midstate_enabled(&self) -> bool {
        (self.0 >> Self::MMEN_OFFSET) & Self::MMEN_MASK == Self::MMEN_MASK
    }
    pub fn enable_multi_midstate(&mut self) -> &mut Self {
        self.0 |= 1 << Self::MMEN_OFFSET;
        self
    }
    pub fn disable_multi_midstate(&mut self) -> &mut Self {
        self.0 &= !(1 << Self::MMEN_OFFSET);
        self
    }

    /// ## Handle the Sweep Frequency Mode field.
    ///
    /// Get and set the Sweep Frequency Mode state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrl;
    ///
    /// let mut cdc = ClockDelayCtrl(0x00); // BM1397 default value
    /// assert!(!cdc.sweep_frequency_mode_enabled());
    /// assert!(cdc.enable_sweep_frequency_mode().sweep_frequency_mode_enabled());
    /// assert!(!cdc.disable_sweep_frequency_mode().sweep_frequency_mode_enabled()); // BM1397 init() value
    /// ```
    pub const fn sweep_frequency_mode_enabled(&self) -> bool {
        (self.0 >> Self::SWPF_MODE_OFFSET) & Self::SWPF_MODE_MASK == Self::SWPF_MODE_MASK
    }
    pub fn enable_sweep_frequency_mode(&mut self) -> &mut Self {
        self.0 |= 1 << Self::SWPF_MODE_OFFSET;
        self
    }
    pub fn disable_sweep_frequency_mode(&mut self) -> &mut Self {
        self.0 &= !(1 << Self::SWPF_MODE_OFFSET);
        self
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

/// # Clock Delay Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClockDelayCtrlV2(pub u8);
impl_boilerplate_for_core_reg!(ClockDelayCtrlV2);

impl ClockDelayCtrlV2 {
    pub const ID: u8 = 0;

    const CCDLY_SEL_OFFSET: u8 = 6;
    const PWTH_SEL_OFFSET: u8 = 3;
    const SWPF_MODE_OFFSET: u8 = 0;

    const CCDLY_SEL_MASK: u8 = 0b11;
    const PWTH_SEL_MASK: u8 = 0b111;
    const SWPF_MODE_MASK: u8 = 0b1;

    /// ## Handle the CCdly field.
    ///
    /// Get and set the CCdly value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrlV2;
    ///
    /// let mut cdc = ClockDelayCtrlV2(0x98); // BM1366 default value
    /// assert_eq!(cdc.ccdly(), 2);
    /// assert_eq!(cdc.set_ccdly(0).ccdly(), 0); // BM1366 init() value
    /// assert_eq!(cdc.set_ccdly(3).ccdly(), 3); // max value
    /// assert_eq!(cdc.set_ccdly(4).ccdly(), 0); // out of bound value
    /// ```
    pub const fn ccdly(&self) -> u8 {
        (self.0 >> Self::CCDLY_SEL_OFFSET) & Self::CCDLY_SEL_MASK
    }
    pub fn set_ccdly(&mut self, ccdly: u8) -> &mut Self {
        self.0 &= !Self::CCDLY_SEL_MASK << Self::CCDLY_SEL_OFFSET;
        self.0 |= (ccdly & Self::CCDLY_SEL_MASK) << Self::CCDLY_SEL_OFFSET;
        self
    }

    /// ## Handle the PWth field.
    ///
    /// Get and set the PWth value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrlV2;
    ///
    /// let mut cdc = ClockDelayCtrlV2(0x98); // BM1366 default value
    /// assert_eq!(cdc.pwth(), 3);
    /// assert_eq!(cdc.set_pwth(4).pwth(), 4); // BM1366 init() value
    /// assert_eq!(cdc.set_pwth(7).pwth(), 7); // max value
    /// assert_eq!(cdc.set_pwth(8).pwth(), 0); // out of bound value
    /// ```
    pub const fn pwth(&self) -> u8 {
        (self.0 >> Self::PWTH_SEL_OFFSET) & Self::PWTH_SEL_MASK
    }
    pub fn set_pwth(&mut self, pwth: u8) -> &mut Self {
        self.0 &= !Self::PWTH_SEL_MASK << Self::PWTH_SEL_OFFSET;
        self.0 |= (pwth & Self::PWTH_SEL_MASK) << Self::PWTH_SEL_OFFSET;
        self
    }

    /// ## Handle the Sweep Frequency Mode field.
    ///
    /// Get and set the Sweep Frequency Mode state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ClockDelayCtrlV2;
    ///
    /// let mut cdc = ClockDelayCtrlV2(0x98); // BM1366 default value
    /// assert!(!cdc.sweep_frequency_mode_enabled());
    /// assert!(cdc.enable_sweep_frequency_mode().sweep_frequency_mode_enabled());
    /// assert!(!cdc.disable_sweep_frequency_mode().sweep_frequency_mode_enabled()); // BM1366 init() value
    /// ```
    pub const fn sweep_frequency_mode_enabled(&self) -> bool {
        (self.0 >> Self::SWPF_MODE_OFFSET) & Self::SWPF_MODE_MASK == Self::SWPF_MODE_MASK
    }
    pub fn enable_sweep_frequency_mode(&mut self) -> &mut Self {
        self.0 |= 1 << Self::SWPF_MODE_OFFSET;
        self
    }
    pub fn disable_sweep_frequency_mode(&mut self) -> &mut Self {
        self.0 &= !(1 << Self::SWPF_MODE_OFFSET);
        self
    }
}

impl ::core::fmt::Display for ClockDelayCtrlV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockDelayCtrlV2")
            .field("ccdly", &self.ccdly())
            .field("pwth", &self.pwth())
            .field(
                "sweep_frequency_mode_enabled",
                &self.sweep_frequency_mode_enabled(),
            )
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockDelayCtrlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockDelayCtrlV2 {{ ccdly: {}, pwth: {}, sweep_frequency_mode_enabled: {} }}",
            self.ccdly(),
            self.pwth(),
            self.sweep_frequency_mode_enabled(),
        );
    }
}
