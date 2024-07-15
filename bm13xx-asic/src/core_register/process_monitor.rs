use crate::core_register::CoreRegister;

/// Process Monitor SELect.
///
/// This is used by [`ProcessMonitorCtrl::pm_sel`] and [`ProcessMonitorCtrl::start`] method.
///
/// [`ProcessMonitorCtrl::pm_sel`]: crate::core_register::ProcessMonitorCtrl::pm_sel
/// [`ProcessMonitorCtrl::start`]: crate::core_register::ProcessMonitorCtrl::start
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ProcessMonitorSelect {
    /// Process Monitor on LVT delay chain.
    LVTDelayChain = 0,
    /// Process Monitor on SVT delay chain.
    SVTDelayChain = 1,
    /// Process Monitor on HVT delay chain.
    HVTDelayChain = 2,
    /// Process Monitor on Critical path chain.
    CriticalPathChain = 3,
}

impl From<ProcessMonitorSelect> for u8 {
    /// Get the register value from a buffer size.
    ///
    /// # Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ProcessMonitorSelect;
    ///
    /// assert_eq!(u8::from(ProcessMonitorSelect::LVTDelayChain), 0);
    /// assert_eq!(u8::from(ProcessMonitorSelect::SVTDelayChain), 1);
    /// assert_eq!(u8::from(ProcessMonitorSelect::HVTDelayChain), 2);
    /// assert_eq!(u8::from(ProcessMonitorSelect::CriticalPathChain), 3);
    /// ```
    fn from(val: ProcessMonitorSelect) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for ProcessMonitorSelect {
    type Error = u8;

    /// Get the buffer size given the register value.
    ///
    /// # Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ProcessMonitorSelect;
    ///
    /// assert_eq!(ProcessMonitorSelect::try_from(0), Ok(ProcessMonitorSelect::LVTDelayChain));
    /// assert_eq!(ProcessMonitorSelect::try_from(1), Ok(ProcessMonitorSelect::SVTDelayChain));
    /// assert_eq!(ProcessMonitorSelect::try_from(2), Ok(ProcessMonitorSelect::HVTDelayChain));
    /// assert_eq!(ProcessMonitorSelect::try_from(3), Ok(ProcessMonitorSelect::CriticalPathChain));
    /// assert_eq!(ProcessMonitorSelect::try_from(4), Err(4));
    /// ```
    fn try_from(val: u8) -> Result<ProcessMonitorSelect, u8> {
        match val {
            x if x == ProcessMonitorSelect::LVTDelayChain as u8 => {
                Ok(ProcessMonitorSelect::LVTDelayChain)
            }
            x if x == ProcessMonitorSelect::SVTDelayChain as u8 => {
                Ok(ProcessMonitorSelect::SVTDelayChain)
            }
            x if x == ProcessMonitorSelect::HVTDelayChain as u8 => {
                Ok(ProcessMonitorSelect::HVTDelayChain)
            }
            x if x == ProcessMonitorSelect::CriticalPathChain as u8 => {
                Ok(ProcessMonitorSelect::CriticalPathChain)
            }
            _ => Err(val),
        }
    }
}

/// # Process Monitor Ctrl core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ProcessMonitorCtrl(pub u8);
impl_boilerplate_for_core_reg!(ProcessMonitorCtrl);

impl ProcessMonitorCtrl {
    pub const ID: u8 = 0x01;

    const PM_START_OFFSET: u8 = 2;
    const PM_SEL_OFFSET: u8 = 0;

    const PM_START_MASK: u8 = 0b1;
    const PM_SEL_MASK: u8 = 0b11;

    /// ## Get the Started state.
    ///
    /// This returns an `bool` with the Started state.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::{ProcessMonitorCtrl, ProcessMonitorSelect};
    ///
    /// let pmc = ProcessMonitorCtrl(ProcessMonitorCtrl::start(ProcessMonitorSelect::HVTDelayChain));
    /// assert!(pmc.started());
    /// assert_eq!(pmc.pm_sel(), ProcessMonitorSelect::HVTDelayChain);
    /// ```
    pub const fn started(&self) -> bool {
        (self.0 >> Self::PM_START_OFFSET) & Self::PM_START_MASK == Self::PM_START_MASK
    }
    pub fn pm_sel(&self) -> ProcessMonitorSelect {
        ProcessMonitorSelect::try_from((self.0 & Self::PM_SEL_MASK) >> Self::PM_SEL_OFFSET).unwrap()
    }
    /// ## Start Process Monitor on pm_sel.
    pub const fn start(pm_sel: ProcessMonitorSelect) -> u8 {
        (Self::PM_START_MASK << Self::PM_START_OFFSET) | ((pm_sel as u8) << Self::PM_SEL_OFFSET)
    }
}

impl ::core::fmt::Display for ProcessMonitorCtrl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ProcessMonitorCtrl")
            .field("started", &self.started())
            .field("pm_sel", &self.pm_sel())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ProcessMonitorCtrl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ProcessMonitorCtrl {{ started: {}, pm_sel: {} }}",
            self.started(),
            self.pm_sel()
        );
    }
}

/// # Process Monitor Data core register
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ProcessMonitorData(pub u8);
impl_boilerplate_for_core_reg!(ProcessMonitorData);

impl ProcessMonitorData {
    pub const ID: u8 = 0x02;

    const DATA_OFFSET: u8 = 0;

    const DATA_MASK: u8 = 0xff;

    /// ## Get the Data.
    ///
    /// This returns an `u8` with the Data.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::core_register::ProcessMonitorData;
    ///
    /// assert_eq!(ProcessMonitorData(0x00).data(), 0x00);
    /// ```
    pub const fn data(&self) -> u8 {
        (self.0 >> Self::DATA_OFFSET) & Self::DATA_MASK
    }
}

impl ::core::fmt::Display for ProcessMonitorData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ProcessMonitorData")
            .field("data", &self.data())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ProcessMonitorData {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ProcessMonitorData {{ data: {} }}", self.data());
    }
}
