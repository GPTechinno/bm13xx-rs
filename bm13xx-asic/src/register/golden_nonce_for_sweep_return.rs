use crate::register::Register;

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

#[cfg(feature = "defmt")]
impl defmt::Format for GoldenNonceForSweepReturn {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "GoldenNonceForSweepReturn {{  }}",);
    }
}
