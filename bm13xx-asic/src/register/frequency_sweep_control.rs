use crate::register::Register;

/// # Frequency Sweep Control 1 register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FrequencySweepControl1(pub u32);
impl_boilerplate_for!(FrequencySweepControl1);

impl FrequencySweepControl1 {
    pub const ADDR: u8 = 0x90;

    // const SWEEP_STATE_OFFSET: u8 = 24;

    // const SWEEP_STATE_MASK: u32 = 0b111;
}

impl core::fmt::Display for FrequencySweepControl1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrequencySweepControl1").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for FrequencySweepControl1 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "FrequencySweepControl1 {{  }}",);
    }
}
