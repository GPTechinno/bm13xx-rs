use crate::register::Register;

/// # PLL Parameter registers
///
/// Used to set PLL parameters.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL0Parameter(pub u32);
impl_boilerplate_for!(PLL0Parameter);

impl PLL0Parameter {
    pub const ADDR: u8 = 0x08;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL1Parameter(pub u32);
impl_boilerplate_for!(PLL1Parameter);

impl PLL1Parameter {
    pub const ADDR: u8 = 0x60;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL2Parameter(pub u32);
impl_boilerplate_for!(PLL2Parameter);

impl PLL2Parameter {
    pub const ADDR: u8 = 0x64;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PLL3Parameter(pub u32);
impl_boilerplate_for!(PLL3Parameter);

impl PLL3Parameter {
    pub const ADDR: u8 = 0x68;
}

pub trait PLLParameterRegister: Register {}

impl dyn PLLParameterRegister {
    /// ## Bit offset for the `LOCKED` field.
    const LOCKED_OFFSET: u8 = 31;
    /// ## Bit offset for the `PLLEN` field.
    const PLLEN_OFFSET: u8 = 30;
    /// ## Bit offset for the `FBDIV` field.
    const FBDIV_OFFSET: u8 = 16;
    /// ## Bit offset for the `REFDIV` field.
    const REFDIV_OFFSET: u8 = 8;
    /// ## Bit offset for the `POSTDIV1` field.
    const POSTDIV1_OFFSET: u8 = 4;
    /// ## Bit offset for the `POSTDIV2` field.
    const POSTDIV2_OFFSET: u8 = 0;

    /// ## Bit mask for the `LOCKED` field.
    const LOCKED_MASK: u32 = 0x1;
    /// ## Bit mask for the `PLLEN` field.
    const PLLEN_MASK: u32 = 0x1;
    /// ## Bit mask for the `FBDIV` field.
    const FBDIV_MASK: u32 = 0xfff;
    /// ## Bit mask for the `REFDIV` field.
    const REFDIV_MASK: u32 = 0x3f;
    /// ## Bit mask for the `POSTDIV1` field.
    const POSTDIV1_MASK: u32 = 0x7;
    /// ## Bit mask for the `POSTDIV2` field.
    const POSTDIV2_MASK: u32 = 0x7;

    /// ## Get the PLL locked state.
    ///
    /// This returns an `bool` with the locked state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert!(pll0.locked());
    /// ```
    pub fn locked(&self) -> bool {
        (self.val() >> Self::LOCKED_OFFSET) & Self::LOCKED_MASK == Self::LOCKED_MASK
    }

    /// ## Get the PLL enabled state.
    ///
    /// This returns an `bool` with the PLL enabled state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert!(pll0.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        (self.val() >> Self::PLLEN_OFFSET) & Self::PLLEN_MASK == Self::PLLEN_MASK
    }

    /// ## Get the PLL FB Divider.
    ///
    /// This returns an `u16` with the PLL FB Divider.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert_eq!(pll0.fbdiv(), 0x0060);
    /// ```
    pub fn fbdiv(&self) -> u16 {
        ((self.val() >> Self::FBDIV_OFFSET) & Self::FBDIV_MASK) as u16
    }

    /// ## Get the PLL REF Divider.
    ///
    /// This returns an `u8` with the PLL REF Divider.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert_eq!(pll0.refdiv(), 0x01);
    /// ```
    pub fn refdiv(&self) -> u8 {
        ((self.val() >> Self::REFDIV_OFFSET) & Self::REFDIV_MASK) as u8
    }

    /// ## Get the PLL POST Divider 1.
    ///
    /// This returns an `u8` with the PLL POST Divider 1.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert_eq!(pll0.postdiv1(), 0x06);
    /// ```
    pub fn postdiv1(&self) -> u8 {
        ((self.val() >> Self::POSTDIV1_OFFSET) & Self::POSTDIV1_MASK) as u8
    }

    /// ## Get the PLL POST Divider 2.
    ///
    /// This returns an `u8` with the PLL POST Divider 2.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{PLL0Parameter, PLLParameterRegister};
    ///
    /// let pll0 = PLL0Parameter(0xC060_0161);
    // assert_eq!(pll0.postdiv2(), 0x01);
    /// ```
    pub fn postdiv2(&self) -> u8 {
        ((self.val() >> Self::POSTDIV2_OFFSET) & Self::POSTDIV2_MASK) as u8
    }
}

impl core::fmt::Display for dyn PLLParameterRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PLLParameter")
            .field("locked", &self.locked())
            .field("enabled", &self.enabled())
            .field("fbdiv", &self.fbdiv())
            .field("refdiv", &self.refdiv())
            .field("postdiv1", &self.postdiv1())
            .field("postdiv2", &self.postdiv2())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for dyn PLLParameterRegister {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PLLParameter {{ locked: {}, enabled: {}, fbdiv: {}, refdiv: {}, postdiv1: {}, postdiv2: {} }}",
            self.locked(),
            self.enabled(),
            self.fbdiv(),
            self.refdiv(),
            self.postdiv1(),
            self.postdiv2(),
        );
    }
}

impl PLLParameterRegister for PLL0Parameter {}
impl PLLParameterRegister for PLL1Parameter {}
impl PLLParameterRegister for PLL2Parameter {}
impl PLLParameterRegister for PLL3Parameter {}
