use crate::register::Register;

/// # Analog Mux Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnalogMuxControl(pub u32);
impl_boilerplate_for!(AnalogMuxControl);

impl AnalogMuxControl {
    pub const ADDR: u8 = 0x54;

    const DIODE_VDD_MUX_SEL_OFFSET: u8 = 0;

    const DIODE_VDD_MUX_SEL_MASK: u32 = 0b111;

    /// ## Handle the `DIODE_VDD_MUX_SEL` field.
    ///
    /// Get and set the `DIODE_VDD_MUX_SEL` value.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::register::AnalogMuxControl;
    ///
    /// let mut ana_mux = AnalogMuxControl(0x0000_0000); // BM1397 default value
    /// assert_eq!(ana_mux.diode_vdd_mux_sel(), 0);
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(3).diode_vdd_mux_sel(), 3);
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(0x7).diode_vdd_mux_sel(), 0x7); // max value
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(0x8).diode_vdd_mux_sel(), 0); // out of bound value
    /// ```
    pub const fn diode_vdd_mux_sel(&self) -> u8 {
        ((self.0 >> Self::DIODE_VDD_MUX_SEL_OFFSET) & Self::DIODE_VDD_MUX_SEL_MASK) as u8
    }
    pub fn set_diode_vdd_mux_sel(&mut self, mux_sel: u8) -> &mut Self {
        self.0 &= !(Self::DIODE_VDD_MUX_SEL_MASK << Self::DIODE_VDD_MUX_SEL_OFFSET);
        self.0 |=
            ((mux_sel as u32) & Self::DIODE_VDD_MUX_SEL_MASK) << Self::DIODE_VDD_MUX_SEL_OFFSET;
        self
    }
}

impl core::fmt::Display for AnalogMuxControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnalogMuxControl")
            .field("diode_vdd_mux_sel", &self.diode_vdd_mux_sel())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for AnalogMuxControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "AnalogMuxControlV2 {{ diode_vdd_mux_sel: {} }}",
            self.diode_vdd_mux_sel(),
        );
    }
}

/// # Analog Mux Control register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnalogMuxControlV2(pub u32);
impl_boilerplate_for!(AnalogMuxControlV2);

impl AnalogMuxControlV2 {
    pub const ADDR: u8 = 0x54;

    const DIODE_VDD_MUX_SEL_OFFSET: u8 = 0;

    const DIODE_VDD_MUX_SEL_MASK: u32 = 0b1111;

    /// ## Handle the `DIODE_VDD_MUX_SEL` field.
    ///
    /// Get and set the `DIODE_VDD_MUX_SEL` value.
    ///
    /// ### Example
    /// ```
    /// use bm13xx_asic::register::AnalogMuxControlV2;
    ///
    /// let mut ana_mux = AnalogMuxControlV2(0x0000_0000); // BM1366 default value
    /// assert_eq!(ana_mux.diode_vdd_mux_sel(), 0);
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(3).diode_vdd_mux_sel(), 3); // BM1366 init() value
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(0xf).diode_vdd_mux_sel(), 0xf); // max value
    /// assert_eq!(ana_mux.set_diode_vdd_mux_sel(0x10).diode_vdd_mux_sel(), 0); // out of bound value
    /// ```
    pub const fn diode_vdd_mux_sel(&self) -> u8 {
        ((self.0 >> Self::DIODE_VDD_MUX_SEL_OFFSET) & Self::DIODE_VDD_MUX_SEL_MASK) as u8
    }
    pub fn set_diode_vdd_mux_sel(&mut self, mux_sel: u8) -> &mut Self {
        self.0 &= !(Self::DIODE_VDD_MUX_SEL_MASK << Self::DIODE_VDD_MUX_SEL_OFFSET);
        self.0 |=
            ((mux_sel as u32) & Self::DIODE_VDD_MUX_SEL_MASK) << Self::DIODE_VDD_MUX_SEL_OFFSET;
        self
    }
}

impl core::fmt::Display for AnalogMuxControlV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnalogMuxControlV2")
            .field("diode_vdd_mux_sel", &self.diode_vdd_mux_sel())
            .finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for AnalogMuxControlV2 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "AnalogMuxControlV2 {{ diode_vdd_mux_sel: {} }}",
            self.diode_vdd_mux_sel(),
        );
    }
}
