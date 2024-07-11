use crate::register::Register;

/// # Io Driver Strenght Configuration register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct IoDriverStrenghtConfiguration(pub u32);
impl_boilerplate_for!(IoDriverStrenghtConfiguration);

impl IoDriverStrenghtConfiguration {
    pub const ADDR: u8 = 0x58;

    // const RF_DS_OFFSET: u8 = 24;
    // const D3RS_EN_OFFSET: u8 = 23;
    // const D2RS_EN_OFFSET: u8 = 22;
    // const D1RS_EN_OFFSET: u8 = 21;
    // const D0RS_EN_OFFSET: u8 = 20;
    // const RO_DS_OFFSET: u8 = 16;
    // const CLKO_DS_OFFSET: u8 = 12;
    // const NRSTO_DS_OFFSET: u8 = 8;
    // const BO_DS_OFFSET: u8 = 4;
    // const CO_DS_OFFSET: u8 = 0;

    // const RF_DS_MASK: u32 = 0b1111;
    // const D3RS_EN_MASK: u32 = 0b1;
    // const D2RS_EN_MASK: u32 = 0b1;
    // const D1RS_EN_MASK: u32 = 0b1;
    // const D0RS_EN_MASK: u32 = 0b1;
    // const RO_DS_MASK: u32 = 0b1111;
    // const CLKO_DS_MASK: u32 = 0b1111;
    // const NRSTO_DS_MASK: u32 = 0b1111;
    // const BO_DS_MASK: u32 = 0b1111;
    // const CO_DS_MASK: u32 = 0b1111;
}

impl ::core::fmt::Display for IoDriverStrenghtConfiguration {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("IoDriverStrenghtConfiguration").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for IoDriverStrenghtConfiguration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "IoDriverStrenghtConfiguration {{  }}",);
    }
}
