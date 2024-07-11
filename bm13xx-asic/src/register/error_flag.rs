use crate::register::Register;

/// # Error Flag register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ErrorFlag(pub u32);
impl_boilerplate_for!(ErrorFlag);

impl ErrorFlag {
    pub const ADDR: u8 = 0x48;

    // const CMD_ERR_CNT_OFFSET: u8 = 24;
    // const WORK_ERR_CNT_OFFSET: u8 = 16;
    // const CORE_RESP_ERR_OFFSET: u8 = 0;

    // const CMD_ERR_CNT_MASK: u32 = 0xff;
    // const WORK_ERR_CNT_MASK: u32 = 0xff;
    // const CORE_RESP_ERR_MASK: u32 = 0xff;
}

impl core::fmt::Display for ErrorFlag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ErrorFlag").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ErrorFlag {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ErrorFlag {{  }}",);
    }
}
