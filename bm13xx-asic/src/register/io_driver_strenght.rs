use crate::register::Register;

/// Driver Select.
///
/// This is used by [`IoDriverStrenghtConfiguration::strenght`], [`IoDriverStrenghtConfiguration::set_strenght`] method
///
/// [`IoDriverStrenghtConfiguration::strenght`]: crate::register::IoDriverStrenghtConfiguration::strenght
// [`IoDriverStrenghtConfiguration::set_strenght`]: crate::register::IoDriverStrenghtConfiguration::set_strenght
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum DriverSelect {
    RF,
    RO,
    CLKO,
    NRSTO,
    BO,
    CO,
}

/// Driver Select.
///
/// This is used by [`IoDriverStrenghtConfiguration::strenght`], [`IoDriverStrenghtConfiguration::set_strenght`] method
///
/// [`IoDriverStrenghtConfiguration::strenght`]: crate::register::IoDriverStrenghtConfiguration::strenght
// [`IoDriverStrenghtConfiguration::set_strenght`]: crate::register::IoDriverStrenghtConfiguration::set_strenght
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum DriverRSelect {
    D0R,
    D1R,
    D2R,
    D3R,
}

/// # Io Driver Strenght Configuration register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct IoDriverStrenghtConfiguration(pub u32);
impl_boilerplate_for!(IoDriverStrenghtConfiguration);

impl IoDriverStrenghtConfiguration {
    pub const ADDR: u8 = 0x58;

    const RF_DS_OFFSET: u8 = 24;
    const D3RS_EN_OFFSET: u8 = 23;
    const D2RS_EN_OFFSET: u8 = 22;
    const D1RS_EN_OFFSET: u8 = 21;
    const D0RS_EN_OFFSET: u8 = 20;
    const RO_DS_OFFSET: u8 = 16;
    const CLKO_DS_OFFSET: u8 = 12;
    const NRSTO_DS_OFFSET: u8 = 8;
    const BO_DS_OFFSET: u8 = 4;
    const CO_DS_OFFSET: u8 = 0;

    const RF_DS_MASK: u32 = 0b1111;
    const D3RS_EN_MASK: u32 = 0b1;
    const D2RS_EN_MASK: u32 = 0b1;
    const D1RS_EN_MASK: u32 = 0b1;
    const D0RS_EN_MASK: u32 = 0b1;
    const RO_DS_MASK: u32 = 0b1111;
    const CLKO_DS_MASK: u32 = 0b1111;
    const NRSTO_DS_MASK: u32 = 0b1111;
    const BO_DS_MASK: u32 = 0b1111;
    const CO_DS_MASK: u32 = 0b1111;

    /// ## Handle the xx_DS field.
    ///
    /// Get and set the xx_DS value.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{DriverSelect, IoDriverStrenghtConfiguration};
    ///
    /// let mut io_conf = IoDriverStrenghtConfiguration(0x0001_2111); // BM1366 default value
    /// assert_eq!(io_conf.strenght(DriverSelect::RF), 0);
    /// assert_eq!(io_conf.strenght(DriverSelect::RO), 1);
    /// assert_eq!(io_conf.strenght(DriverSelect::CLKO), 2);
    /// assert_eq!(io_conf.strenght(DriverSelect::NRSTO), 1);
    /// assert_eq!(io_conf.strenght(DriverSelect::BO), 1);
    /// assert_eq!(io_conf.strenght(DriverSelect::CO), 1);
    /// assert_eq!(io_conf.set_strenght(DriverSelect::RF, 2).strenght(DriverSelect::RF), 2); // BM1366 init() value
    /// assert_eq!(io_conf.set_strenght(DriverSelect::RF, 0xf).strenght(DriverSelect::RF), 0xf); // max value
    /// assert_eq!(io_conf.set_strenght(DriverSelect::RF, 0x10).strenght(DriverSelect::RF), 0); // out of bound value
    /// ```
    pub const fn strenght(&self, drv: DriverSelect) -> u8 {
        ((self.0
            >> match drv {
                DriverSelect::RF => Self::RF_DS_OFFSET,
                DriverSelect::RO => Self::RO_DS_OFFSET,
                DriverSelect::CLKO => Self::CLKO_DS_OFFSET,
                DriverSelect::NRSTO => Self::NRSTO_DS_OFFSET,
                DriverSelect::BO => Self::BO_DS_OFFSET,
                DriverSelect::CO => Self::CO_DS_OFFSET,
            })
            & match drv {
                DriverSelect::RF => Self::RF_DS_MASK,
                DriverSelect::RO => Self::RO_DS_MASK,
                DriverSelect::CLKO => Self::CLKO_DS_MASK,
                DriverSelect::NRSTO => Self::NRSTO_DS_MASK,
                DriverSelect::BO => Self::BO_DS_MASK,
                DriverSelect::CO => Self::CO_DS_MASK,
            }) as u8
    }
    pub fn set_strenght(&mut self, drv: DriverSelect, strenght: u8) -> &mut Self {
        let offset = match drv {
            DriverSelect::RF => Self::RF_DS_OFFSET,
            DriverSelect::RO => Self::RO_DS_OFFSET,
            DriverSelect::CLKO => Self::CLKO_DS_OFFSET,
            DriverSelect::NRSTO => Self::NRSTO_DS_OFFSET,
            DriverSelect::BO => Self::BO_DS_OFFSET,
            DriverSelect::CO => Self::CO_DS_OFFSET,
        };
        let mask = match drv {
            DriverSelect::RF => Self::RF_DS_MASK,
            DriverSelect::RO => Self::RO_DS_MASK,
            DriverSelect::CLKO => Self::CLKO_DS_MASK,
            DriverSelect::NRSTO => Self::NRSTO_DS_MASK,
            DriverSelect::BO => Self::BO_DS_MASK,
            DriverSelect::CO => Self::CO_DS_MASK,
        };
        self.0 &= !(mask << offset);
        self.0 |= ((strenght as u32) & mask) << offset;
        self
    }

    /// ## Handle the DxRS_EN field.
    ///
    /// Get and set the DxRS_EN state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::register::{DriverRSelect, IoDriverStrenghtConfiguration};
    ///
    /// let mut io_conf = IoDriverStrenghtConfiguration(0x0001_2111); // BM1366 default value
    /// assert!(!io_conf.enabled(DriverRSelect::D0R));
    /// assert!(!io_conf.enabled(DriverRSelect::D1R));
    /// assert!(!io_conf.enabled(DriverRSelect::D2R));
    /// assert!(!io_conf.enabled(DriverRSelect::D3R));
    /// assert!(io_conf.enable(DriverRSelect::D0R).enabled(DriverRSelect::D0R));
    /// assert!(!io_conf.disable(DriverRSelect::D0R).enabled(DriverRSelect::D0R));
    /// ```
    pub const fn enabled(&self, drv: DriverRSelect) -> bool {
        let offset = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_OFFSET,
            DriverRSelect::D1R => Self::D1RS_EN_OFFSET,
            DriverRSelect::D2R => Self::D2RS_EN_OFFSET,
            DriverRSelect::D3R => Self::D3RS_EN_OFFSET,
        };
        let mask = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_MASK,
            DriverRSelect::D1R => Self::D1RS_EN_MASK,
            DriverRSelect::D2R => Self::D2RS_EN_MASK,
            DriverRSelect::D3R => Self::D3RS_EN_MASK,
        };
        (self.0 >> offset) & mask == mask
    }
    pub fn enable(&mut self, drv: DriverRSelect) -> &mut Self {
        let offset = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_OFFSET,
            DriverRSelect::D1R => Self::D1RS_EN_OFFSET,
            DriverRSelect::D2R => Self::D2RS_EN_OFFSET,
            DriverRSelect::D3R => Self::D3RS_EN_OFFSET,
        };
        let mask = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_MASK,
            DriverRSelect::D1R => Self::D1RS_EN_MASK,
            DriverRSelect::D2R => Self::D2RS_EN_MASK,
            DriverRSelect::D3R => Self::D3RS_EN_MASK,
        };
        self.0 |= mask << offset;
        self
    }
    pub fn disable(&mut self, drv: DriverRSelect) -> &mut Self {
        let offset = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_OFFSET,
            DriverRSelect::D1R => Self::D1RS_EN_OFFSET,
            DriverRSelect::D2R => Self::D2RS_EN_OFFSET,
            DriverRSelect::D3R => Self::D3RS_EN_OFFSET,
        };
        let mask = match drv {
            DriverRSelect::D0R => Self::D0RS_EN_MASK,
            DriverRSelect::D1R => Self::D1RS_EN_MASK,
            DriverRSelect::D2R => Self::D2RS_EN_MASK,
            DriverRSelect::D3R => Self::D3RS_EN_MASK,
        };
        self.0 &= !(mask << offset);
        self
    }
}

impl core::fmt::Display for IoDriverStrenghtConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IoDriverStrenghtConfiguration")
            .field("RF_DS", &self.strenght(DriverSelect::RF))
            .field("RO_DS", &self.strenght(DriverSelect::RO))
            .field("CLKO_DS", &self.strenght(DriverSelect::CLKO))
            .field("NRSTO_DS", &self.strenght(DriverSelect::NRSTO))
            .field("BO_DS", &self.strenght(DriverSelect::BO))
            .field("CO_DS", &self.strenght(DriverSelect::CO))
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for IoDriverStrenghtConfiguration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "IoDriverStrenghtConfiguration {{ RF_DS: {}, RO_DS: {}, CLKO_DS: {}, NRSTO_DS: {}, BO_DS: {}, CO_DS: {} }}",
            self.strenght(DriverSelect::RF),
            self.strenght(DriverSelect::RO),
            self.strenght(DriverSelect::CLKO),
            self.strenght(DriverSelect::NRSTO),
            self.strenght(DriverSelect::BO),
            self.strenght(DriverSelect::CO),
        );
    }
}
