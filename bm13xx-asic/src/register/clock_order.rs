use crate::register::Register;

/// # Ordered Clock Monitor register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OrderedClockMonitor(pub u32);
impl_boilerplate_for!(OrderedClockMonitor);

impl OrderedClockMonitor {
    pub const ADDR: u8 = 0x6C;

    // const START_OFFSET: u8 = 31;
    // const CLK_SEL_OFFSET: u8 = 24;
    // const CLK_COUNT_OFFSET: u8 = 0;

    // const START_MASK: u32 = 0b1;
    // const CLK_SEL_MASK: u32 = 0b1111;
    // const CLK_COUNT_MASK: u32 = 0xffff;
}

impl ::core::fmt::Display for OrderedClockMonitor {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("OrderedClockMonitor").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for OrderedClockMonitor {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "OrderedClockMonitor {{  }}",);
    }
}

/// Clock Select.
///
/// This is used by [`ClockOrderControl0::clock_select`], [`ClockOrderControl0::set_clock_select`],
/// [`ClockOrderControl1::clock_select`] and [`ClockOrderControl1::set_clock_select`] method
///
/// [`ClockOrderControl0::clock_select`]: crate::register::ClockOrderControl0::clock_select
/// [`ClockOrderControl0::set_clock_select`]: crate::register::ClockOrderControl0::set_clock_select
/// [`ClockOrderControl1::clock_select`]: crate::register::ClockOrderControl1::clock_select
/// [`ClockOrderControl1::set_clock_select`]: crate::register::ClockOrderControl1::set_clock_select
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ClockSelect {
    /// Default.
    Default = 0b0000,
}

impl ClockSelect {
    /// Convert a raw `u8` to an `ClockSelect`.
    ///
    /// Bit values that do not correspond to a ClockSelect will be returned in the
    /// `Err` variant of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use bm13xx_asic::register::ClockSelect;
    ///
    /// assert_eq!(ClockSelect::from_raw(0b0000), Ok(ClockSelect::Default));
    /// assert_eq!(ClockSelect::from_raw(0b0101), Err(0b0101));
    /// ```
    pub const fn from_raw(val: u8) -> Result<Self, u8> {
        match val {
            x if x == ClockSelect::Default as u8 => Ok(ClockSelect::Default),
            _ => Err(val),
        }
    }
}

impl From<ClockSelect> for u8 {
    fn from(val: ClockSelect) -> u8 {
        val as u8
    }
}

impl Default for ClockSelect {
    fn default() -> Self {
        Self::Default
    }
}

impl TryFrom<u8> for ClockSelect {
    type Error = u8;
    fn try_from(val: u8) -> Result<Self, u8> {
        Self::from_raw(val)
    }
}

/// # Clock Order Control 0 register
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockOrderControl0(pub u32);
impl_boilerplate_for!(ClockOrderControl0);

impl ClockOrderControl0 {
    pub const ADDR: u8 = 0x80;

    const CLKN_SEL_LENGTH: u8 = 4;

    const CLKN_SEL_MASK: u32 = 0xF;

    /// ## Get the clock select.
    ///
    /// This returns an `Err(u8)` with the clock select bits if the clock select bits
    /// do not match a valid clock select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl0};
    ///
    /// let clk_ord_ctrl = ClockOrderControl0(0xD95C_8410);
    /// assert_eq!(clk_ord_ctrl.clock_select(0), Ok(ClockSelect::Default));
    /// ```
    pub const fn clock_select(&self, clock: u8) -> Result<ClockSelect, u8> {
        if clock > 7 {
            return Err(clock);
        }
        ClockSelect::from_raw(
            ((self.0 >> (clock * Self::CLKN_SEL_LENGTH)) & Self::CLKN_SEL_MASK) as u8,
        )
    }

    /* TODO
    /// ## Set the clock select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl0};
    ///
    /// const CLK_ORD_CTRL: ClockOrderControl0 = ClockOrderControl0::DEFAULT.set_clock_select(1, ClockSelect::Default);
    /// assert_eq!(CLK_ORD_CTRL.clock_select(1), Ok(ClockSelect::Default));
    /// ```
    pub const fn set_clock_select(mut self, clock: u8, clock_select: ClockSelect) -> Self {
        if clock < 8 {
            self.0 = (self.0 & !(Self::CLKN_SEL_MASK << (clock * Self::CLKN_SEL_LENGTH)))
                | ((((clock_select as u8) & 0xF) as u32) << (clock * Self::CLKN_SEL_LENGTH));
        }
        self
    }*/
}

impl core::fmt::Display for ClockOrderControl0 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockOrderControl0")
            .field("clock0_select", &self.clock_select(0))
            .field("clock1_select", &self.clock_select(1))
            .field("clock2_select", &self.clock_select(2))
            .field("clock3_select", &self.clock_select(3))
            .field("clock4_select", &self.clock_select(4))
            .field("clock5_select", &self.clock_select(5))
            .field("clock6_select", &self.clock_select(6))
            .field("clock7_select", &self.clock_select(7))
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockOrderControl0 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockOrderControl0 {{ clock0_select: {}, clock1_select: {}, clock2_select: {}, clock3_select: {}, clock4_select: {}, clock5_select: {}, clock6_select: {}, clock7_select: {} }}",
            self.clock_select(0),
            self.clock_select(1),
            self.clock_select(2),
            self.clock_select(3),
            self.clock_select(4),
            self.clock_select(5),
            self.clock_select(6),
            self.clock_select(7),
        );
    }
}

/// # Clock Order Control 1 register
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockOrderControl1(pub u32);
impl_boilerplate_for!(ClockOrderControl1);

impl ClockOrderControl1 {
    pub const ADDR: u8 = 0x84;

    const CLKN_SEL_LENGTH: u8 = 4;

    const CLKN_SEL_MASK: u32 = 0xF;

    /// ## Get the clock select.
    ///
    /// This returns an `Err(u8)` with the clock select bits if the clock select bits
    /// do not match a valid clock select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl1};
    ///
    /// let clk_ord_ctrl = ClockOrderControl1(0xFB73EA62);
    /// assert_eq!(clk_ord_ctrl.clock_select(0), ClockSelect::from_raw(0x2));
    /// ```
    pub const fn clock_select(&self, clock: u8) -> Result<ClockSelect, u8> {
        if clock > 7 {
            return Err(clock);
        }
        ClockSelect::from_raw(
            ((self.0 >> (clock * Self::CLKN_SEL_LENGTH)) & Self::CLKN_SEL_MASK) as u8,
        )
    }

    /* TODO
    /// ## Set the clock select.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl1};
    ///
    /// const CLK_ORD_CTRL: ClockOrderControl1 = ClockOrderControl1::DEFAULT.set_clock_select(1, ClockSelect::Default);
    /// assert_eq!(CLK_ORD_CTRL.clock_select(1), Ok(ClockSelect::Default));
    /// ```
    pub const fn set_clock_select(mut self, clock: u8, clock_select: ClockSelect) -> Self {
        if clock < 8 {
            self.0 = (self.0 & !(Self::CLKN_SEL_MASK << (clock * Self::CLKN_SEL_LENGTH)))
                | ((((clock_select as u8) & 0xF) as u32) << (clock * Self::CLKN_SEL_LENGTH));
        }
        self
    }*/
}

impl core::fmt::Display for ClockOrderControl1 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockOrderControl1")
            .field("clock8_select", &self.clock_select(0))
            .field("clock9_select", &self.clock_select(1))
            .field("clock10_select", &self.clock_select(2))
            .field("clock11_select", &self.clock_select(3))
            .field("clock12_select", &self.clock_select(4))
            .field("clock13_select", &self.clock_select(5))
            .field("clock14_select", &self.clock_select(6))
            .field("clock15_select", &self.clock_select(7))
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockOrderControl1 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockOrderControl1 {{ clock8_select: {}, clock9_select: {}, clock10_select: {}, clock11_select: {}, clock12_select: {}, clock13_select: {}, clock14_select: {}, clock15_select: {} }}",
            self.clock_select(0),
            self.clock_select(1),
            self.clock_select(2),
            self.clock_select(3),
            self.clock_select(4),
            self.clock_select(5),
            self.clock_select(6),
            self.clock_select(7),
        );
    }
}

/// # Clock Order Status register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClockOrderStatus(pub u32);
impl_boilerplate_for!(ClockOrderStatus);

impl ClockOrderStatus {
    pub const ADDR: u8 = 0x8C;

    // const CLOK_ORDER_STATUS_OFFSET: u8 = 0;

    // const CLOK_ORDER_STATUS_MASK: u32 = 0xffff_ffff;
}

impl ::core::fmt::Display for ClockOrderStatus {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ClockOrderStatus").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockOrderStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ClockOrderStatus {{  }}",);
    }
}
