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

impl core::fmt::Display for OrderedClockMonitor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrderedClockMonitor").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for OrderedClockMonitor {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "OrderedClockMonitor {{  }}",);
    }
}

/// Clock Select.
///
/// This is used by [`ClockOrderControl0::clock`], [`ClockOrderControl0::set_clock`],
/// [`ClockOrderControl1::clock`] and [`ClockOrderControl1::set_clock`] method
///
/// [`ClockOrderControl0::clock`]: crate::register::ClockOrderControl0::clock
// [`ClockOrderControl0::set_clock`]: crate::register::ClockOrderControl0::set_clock
/// [`ClockOrderControl1::clock`]: crate::register::ClockOrderControl1::clock
// [`ClockOrderControl1::set_clock`]: crate::register::ClockOrderControl1::set_clock
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum ClockSelect {
    CLK0,
    CLK1,
    CLK2,
    CLK3,
    CLK4,
    CLK5,
    CLK6,
    CLK7,
    CLK8,
    CLK9,
    CLK10,
    CLK11,
    CLK12,
    CLK13,
    CLK14,
    CLK15,
}

/// # Clock Order Control 0 register
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockOrderControl0(pub u32);
impl_boilerplate_for!(ClockOrderControl0);

impl ClockOrderControl0 {
    pub const ADDR: u8 = 0x80;

    const CLK_SEL0_OFFSET: u8 = 0;
    const CLK_SEL1_OFFSET: u8 = 4;
    const CLK_SEL2_OFFSET: u8 = 8;
    const CLK_SEL3_OFFSET: u8 = 12;
    const CLK_SEL4_OFFSET: u8 = 16;
    const CLK_SEL5_OFFSET: u8 = 20;
    const CLK_SEL6_OFFSET: u8 = 24;
    const CLK_SEL7_OFFSET: u8 = 28;

    const CLK_SEL_MASK: u32 = 0xF;

    /// ## Handle the CLK_SELx field.
    ///
    /// Get and get the CLK_SELx value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl0};
    ///
    /// let mut clk_ord_ctrl = ClockOrderControl0(0xD95C_8410); // BM1397 default value
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK0), Some(0));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK1), Some(1));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK2), Some(4));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK3), Some(8));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK4), Some(12));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK5), Some(5));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK6), Some(9));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK7), Some(13));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK8), None);
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK1, 0).clock(ClockSelect::CLK1), Some(0)); // min value
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK1, 15).clock(ClockSelect::CLK1), Some(15)); // max value
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK1, 16).clock(ClockSelect::CLK1), Some(0)); // out of bound value
    /// ```
    pub const fn clock(&self, clk: ClockSelect) -> Option<u8> {
        let offset = match clk {
            ClockSelect::CLK0 => Self::CLK_SEL0_OFFSET,
            ClockSelect::CLK1 => Self::CLK_SEL1_OFFSET,
            ClockSelect::CLK2 => Self::CLK_SEL2_OFFSET,
            ClockSelect::CLK3 => Self::CLK_SEL3_OFFSET,
            ClockSelect::CLK4 => Self::CLK_SEL4_OFFSET,
            ClockSelect::CLK5 => Self::CLK_SEL5_OFFSET,
            ClockSelect::CLK6 => Self::CLK_SEL6_OFFSET,
            ClockSelect::CLK7 => Self::CLK_SEL7_OFFSET,
            _ => return None,
        };
        Some(((self.0 >> (offset)) & Self::CLK_SEL_MASK) as u8)
    }
    pub fn set_clock(&mut self, clk: ClockSelect, value: u8) -> &mut Self {
        let offset = match clk {
            ClockSelect::CLK0 => Self::CLK_SEL0_OFFSET,
            ClockSelect::CLK1 => Self::CLK_SEL1_OFFSET,
            ClockSelect::CLK2 => Self::CLK_SEL2_OFFSET,
            ClockSelect::CLK3 => Self::CLK_SEL3_OFFSET,
            ClockSelect::CLK4 => Self::CLK_SEL4_OFFSET,
            ClockSelect::CLK5 => Self::CLK_SEL5_OFFSET,
            ClockSelect::CLK6 => Self::CLK_SEL6_OFFSET,
            ClockSelect::CLK7 => Self::CLK_SEL7_OFFSET,
            _ => return self,
        };
        self.0 &= !(Self::CLK_SEL_MASK << offset);
        self.0 |= ((value as u32) & Self::CLK_SEL_MASK) << offset;
        self
    }
}

impl core::fmt::Display for ClockOrderControl0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClockOrderControl0")
            .field("CLK_SEL0", &self.clock(ClockSelect::CLK0).unwrap())
            .field("CLK_SEL1", &self.clock(ClockSelect::CLK1).unwrap())
            .field("CLK_SEL2", &self.clock(ClockSelect::CLK2).unwrap())
            .field("CLK_SEL3", &self.clock(ClockSelect::CLK3).unwrap())
            .field("CLK_SEL4", &self.clock(ClockSelect::CLK4).unwrap())
            .field("CLK_SEL5", &self.clock(ClockSelect::CLK5).unwrap())
            .field("CLK_SEL6", &self.clock(ClockSelect::CLK6).unwrap())
            .field("CLK_SEL7", &self.clock(ClockSelect::CLK7).unwrap())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ClockOrderControl0 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockOrderControl0 {{ CLK_SEL0: {}, CLK_SEL1: {}, CLK_SEL2: {}, CLK_SEL3: {}, CLK_SEL4: {}, CLK_SEL5: {}, CLK_SEL6: {}, CLK_SEL7: {} }}",
            self.clock(ClockSelect::CLK0).unwrap(),
            self.clock(ClockSelect::CLK1).unwrap(),
            self.clock(ClockSelect::CLK2).unwrap(),
            self.clock(ClockSelect::CLK3).unwrap(),
            self.clock(ClockSelect::CLK4).unwrap(),
            self.clock(ClockSelect::CLK5).unwrap(),
            self.clock(ClockSelect::CLK6).unwrap(),
            self.clock(ClockSelect::CLK7).unwrap(),
        );
    }
}

/// # Clock Order Control 1 register
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockOrderControl1(pub u32);
impl_boilerplate_for!(ClockOrderControl1);

impl ClockOrderControl1 {
    pub const ADDR: u8 = 0x84;

    const CLK_SEL8_OFFSET: u8 = 0;
    const CLK_SEL9_OFFSET: u8 = 4;
    const CLK_SEL10_OFFSET: u8 = 8;
    const CLK_SEL11_OFFSET: u8 = 12;
    const CLK_SEL12_OFFSET: u8 = 16;
    const CLK_SEL13_OFFSET: u8 = 20;
    const CLK_SEL14_OFFSET: u8 = 24;
    const CLK_SEL15_OFFSET: u8 = 28;

    const CLK_SEL_MASK: u32 = 0xF;

    /// ## Handle the CLK_SELx field.
    ///
    /// Get and get the CLK_SELx value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, ClockOrderControl1};
    ///
    /// let mut clk_ord_ctrl = ClockOrderControl1(0xFB73_EA62); // BM1397 default value
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK8), Some(2));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK9), Some(6));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK10), Some(10));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK11), Some(14));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK12), Some(3));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK13), Some(7));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK14), Some(11));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK15), Some(15));
    /// assert_eq!(clk_ord_ctrl.clock(ClockSelect::CLK0), None);
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK8, 0).clock(ClockSelect::CLK8), Some(0)); // min value
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK8, 15).clock(ClockSelect::CLK8), Some(15)); // max value
    /// assert_eq!(clk_ord_ctrl.set_clock(ClockSelect::CLK8, 16).clock(ClockSelect::CLK8), Some(0)); // out of bound value
    /// ```
    pub const fn clock(&self, clk: ClockSelect) -> Option<u8> {
        let offset = match clk {
            ClockSelect::CLK8 => Self::CLK_SEL8_OFFSET,
            ClockSelect::CLK9 => Self::CLK_SEL9_OFFSET,
            ClockSelect::CLK10 => Self::CLK_SEL10_OFFSET,
            ClockSelect::CLK11 => Self::CLK_SEL11_OFFSET,
            ClockSelect::CLK12 => Self::CLK_SEL12_OFFSET,
            ClockSelect::CLK13 => Self::CLK_SEL13_OFFSET,
            ClockSelect::CLK14 => Self::CLK_SEL14_OFFSET,
            ClockSelect::CLK15 => Self::CLK_SEL15_OFFSET,
            _ => return None,
        };
        Some(((self.0 >> (offset)) & Self::CLK_SEL_MASK) as u8)
    }
    pub fn set_clock(&mut self, clk: ClockSelect, value: u8) -> &mut Self {
        let offset = match clk {
            ClockSelect::CLK8 => Self::CLK_SEL8_OFFSET,
            ClockSelect::CLK9 => Self::CLK_SEL9_OFFSET,
            ClockSelect::CLK10 => Self::CLK_SEL10_OFFSET,
            ClockSelect::CLK11 => Self::CLK_SEL11_OFFSET,
            ClockSelect::CLK12 => Self::CLK_SEL12_OFFSET,
            ClockSelect::CLK13 => Self::CLK_SEL13_OFFSET,
            ClockSelect::CLK14 => Self::CLK_SEL14_OFFSET,
            ClockSelect::CLK15 => Self::CLK_SEL15_OFFSET,
            _ => return self,
        };
        self.0 &= !(Self::CLK_SEL_MASK << offset);
        self.0 |= ((value as u32) & Self::CLK_SEL_MASK) << offset;
        self
    }
}

impl core::fmt::Display for ClockOrderControl1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClockOrderControl1")
            .field("CLK_SEL8", &self.clock(ClockSelect::CLK8).unwrap())
            .field("CLK_SEL9", &self.clock(ClockSelect::CLK9).unwrap())
            .field("CLK_SEL10", &self.clock(ClockSelect::CLK10).unwrap())
            .field("CLK_SEL11", &self.clock(ClockSelect::CLK11).unwrap())
            .field("CLK_SEL12", &self.clock(ClockSelect::CLK12).unwrap())
            .field("CLK_SEL13", &self.clock(ClockSelect::CLK13).unwrap())
            .field("CLK_SEL14", &self.clock(ClockSelect::CLK14).unwrap())
            .field("CLK_SEL15", &self.clock(ClockSelect::CLK15).unwrap())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ClockOrderControl1 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ClockOrderControl1 {{ CLK_SEL8: {}, CLK_SEL9: {}, CLK_SEL10: {}, CLK_SEL11: {}, CLK_SEL12: {}, CLK_SEL13: {}, CLK_SEL14: {}, CLK_SEL15: {} }}",
            self.clock(ClockSelect::CLK8).unwrap(),
            self.clock(ClockSelect::CLK9).unwrap(),
            self.clock(ClockSelect::CLK10).unwrap(),
            self.clock(ClockSelect::CLK11).unwrap(),
            self.clock(ClockSelect::CLK12).unwrap(),
            self.clock(ClockSelect::CLK13).unwrap(),
            self.clock(ClockSelect::CLK14).unwrap(),
            self.clock(ClockSelect::CLK15).unwrap(),
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

impl core::fmt::Display for ClockOrderStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClockOrderStatus").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for ClockOrderStatus {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ClockOrderStatus {{  }}",);
    }
}

/// # Ordered Clock Enable register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OrderedClockEnable(pub u32);
impl_boilerplate_for!(OrderedClockEnable);

impl OrderedClockEnable {
    pub const ADDR: u8 = 0x20;

    const EN_MASK: u32 = 0b1;

    /// ## Handle the enabled fields.
    ///
    /// Get and set the enabled state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{ClockSelect, OrderedClockEnable};
    ///
    /// let mut clk_ord_en = OrderedClockEnable(0x0000_FFFF); // BM1397 default value
    /// assert!(clk_ord_en.enabled(ClockSelect::CLK0));
    /// assert!(clk_ord_en.enabled(ClockSelect::CLK8));
    /// assert!(!clk_ord_en.disable(ClockSelect::CLK0).enabled(ClockSelect::CLK0));
    /// assert!(clk_ord_en.enable(ClockSelect::CLK0).enabled(ClockSelect::CLK0));
    /// assert!(!clk_ord_en.disable_all().enabled(ClockSelect::CLK0));
    /// assert!(clk_ord_en.enable_all().enabled(ClockSelect::CLK0));
    /// ```
    pub const fn enabled(&self, clk: ClockSelect) -> bool {
        (self.0 >> (clk as usize)) & Self::EN_MASK == Self::EN_MASK
    }
    pub fn enable(&mut self, clk: ClockSelect) -> &mut Self {
        self.0 |= Self::EN_MASK << (clk as usize);
        self
    }
    pub fn disable(&mut self, clk: ClockSelect) -> &mut Self {
        self.0 &= !(Self::EN_MASK << (clk as usize));
        self
    }
    pub fn enable_all(&mut self) -> &mut Self {
        self.0 = 0xffff_ffff;
        self
    }
    pub fn disable_all(&mut self) -> &mut Self {
        self.0 = 0x0000_0000;
        self
    }
}

impl core::fmt::Display for OrderedClockEnable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrderedClockEnable")
            .field("CLK_EN0", &self.enabled(ClockSelect::CLK0))
            .field("CLK_EN1", &self.enabled(ClockSelect::CLK1))
            .field("CLK_EN2", &self.enabled(ClockSelect::CLK2))
            .field("CLK_EN3", &self.enabled(ClockSelect::CLK3))
            .field("CLK_EN4", &self.enabled(ClockSelect::CLK4))
            .field("CLK_EN5", &self.enabled(ClockSelect::CLK5))
            .field("CLK_EN6", &self.enabled(ClockSelect::CLK6))
            .field("CLK_EN7", &self.enabled(ClockSelect::CLK7))
            .field("CLK_EN8", &self.enabled(ClockSelect::CLK8))
            .field("CLK_EN9", &self.enabled(ClockSelect::CLK9))
            .field("CLK_EN10", &self.enabled(ClockSelect::CLK10))
            .field("CLK_EN11", &self.enabled(ClockSelect::CLK11))
            .field("CLK_EN12", &self.enabled(ClockSelect::CLK12))
            .field("CLK_EN13", &self.enabled(ClockSelect::CLK13))
            .field("CLK_EN14", &self.enabled(ClockSelect::CLK14))
            .field("CLK_EN15", &self.enabled(ClockSelect::CLK15))
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for OrderedClockEnable {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "OrderedClockEnable {{ CLK_EN0: {}, CLK_EN1: {}, CLK_EN2: {}, CLK_EN3: {}, CLK_EN4: {}, CLK_EN5: {}, CLK_EN6: {}, CLK_EN7: {}, CLK_EN8: {}, CLK_EN9: {}, CLK_EN10: {}, CLK_EN11: {}, CLK_EN12: {}, CLK_EN13: {}, CLK_EN14: {}, CLK_EN15: {} }}",
            self.enabled(ClockSelect::CLK0),
            self.enabled(ClockSelect::CLK1),
            self.enabled(ClockSelect::CLK2),
            self.enabled(ClockSelect::CLK3),
            self.enabled(ClockSelect::CLK4),
            self.enabled(ClockSelect::CLK5),
            self.enabled(ClockSelect::CLK6),
            self.enabled(ClockSelect::CLK7),
            self.enabled(ClockSelect::CLK8),
            self.enabled(ClockSelect::CLK9),
            self.enabled(ClockSelect::CLK10),
            self.enabled(ClockSelect::CLK11),
            self.enabled(ClockSelect::CLK12),
            self.enabled(ClockSelect::CLK13),
            self.enabled(ClockSelect::CLK14),
            self.enabled(ClockSelect::CLK15),
        );
    }
}
