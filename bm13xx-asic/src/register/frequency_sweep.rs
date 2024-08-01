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

#[cfg(feature = "defmt-03")]
impl defmt::Format for FrequencySweepControl1 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "FrequencySweepControl1 {{  }}",);
    }
}

/// # Golden Nonce For Sweep Return register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GoldenNonceForSweepReturn(pub u32);
impl_boilerplate_for!(GoldenNonceForSweepReturn);

impl GoldenNonceForSweepReturn {
    pub const ADDR: u8 = 0x94;

    // const GNOSWR_OFFSET: u8 = 0;

    // const GNOSWR_MASK: u32 = 0xffff_ffff;
}

impl core::fmt::Display for GoldenNonceForSweepReturn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GoldenNonceForSweepReturn").finish()
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for GoldenNonceForSweepReturn {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "GoldenNonceForSweepReturn {{  }}",);
    }
}
