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
