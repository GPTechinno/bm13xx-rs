//! Supply voltages through the on-chip ADC.
//!
//! The ADC is set up through `ADC_CTRL0` to `ADC_CTRL3` (`0xB8`–`0xBB`) and gives each conversion in
//! `ADC_VOLTAGE_DATAOUT` (`0xBD`): bit 31 data ready, DV0 in bits 14:0. Four sense channels are
//! multiplexed (selectors 1, 2, 4 and 5); which rail each one senses is not known.

use crate::{
    register::{AdcCtrl0, AdcCtrl1},
    CmdDelay, SensorStep,
};
use bm13xx_protocol::command::{Command, Destination};

const ADC_CTRL2: u8 = 0xBA;
const ADC_CTRL3: u8 = 0xBB;
const ADC_VOLTAGE_DATAOUT: u8 = 0xBD;
/// Mux selector of each sense channel.
const SELECTORS: [u32; 4] = [1, 2, 4, 5];
/// Conversions of a reading; the first one is discarded.
pub const SUPPLY_CONVERSIONS: usize = 10;
const SETUP_WRITES: usize = 4;

/// # Supply voltage reading settings of a chip
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SupplyAdc {
    /// `ADC_CTRL0` set-up value, without the selector (bits 15:13).
    pub ctrl0_setup: u32,
    /// `ADC_CTRL1` value.
    pub ctrl1: u32,
    /// `ADC_CTRL0` value starting a conversion, without the selector.
    pub ctrl0_start: u32,
    /// Millivolts per unit of `DV0 / 16384 - 1`, doubled on channels 2 and 3.
    pub scale_mv: f32,
}

impl SupplyAdc {
    /// ## Step `step` of the reading of `channel` (0 to 3) for chip `dest`
    ///
    /// Four set-up writes, then [`SUPPLY_CONVERSIONS`] times: start a conversion, wait 60 ms, read
    /// `ADC_VOLTAGE_DATAOUT`. `None` after the last step or for a channel above 3.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::{adc::{SupplyAdc, SUPPLY_CONVERSIONS}, SensorStep};
    /// use bm13xx_protocol::command::Destination;
    ///
    /// let adc = SupplyAdc { ctrl0_setup: 0x230C_030D, ctrl1: 0x3F01_4381, ctrl0_start: 0x232C_030D, scale_mv: 1000.0 };
    /// let dest = Destination::Chip(0);
    /// let steps: Vec<SensorStep> = (0..).map_while(|i| adc.step(i, dest, 2)).collect();
    /// assert_eq!(steps.len(), 4 + 2 * SUPPLY_CONVERSIONS);
    /// // channel 2 is selector 4: ADC_CTRL0 = 0x230C_030D | 4 << 13
    /// let SensorStep::Write(setup) = &steps[0] else { panic!() };
    /// assert_eq!(setup.cmd[5..10], [0xB8, 0x23, 0x0C, 0x83, 0x0D]);
    /// assert_eq!(steps[5], SensorStep::Read { reg_addr: 0xBD, delay_ms: 0 });
    /// assert_eq!(adc.step(0, dest, 4), None);
    /// ```
    pub fn step(&self, step: usize, dest: Destination, channel: u8) -> Option<SensorStep> {
        let selector = *SELECTORS.get(channel as usize)? << 13;
        let write = |addr: u8, value: u32, delay_ms: u32| {
            Some(SensorStep::Write(CmdDelay {
                cmd: Command::write_reg(addr, value, dest),
                delay_ms,
            }))
        };
        match step {
            0 => write(AdcCtrl0::ADDR, self.ctrl0_setup | selector, 0),
            1 => write(AdcCtrl1::ADDR, self.ctrl1, 0),
            2 => write(ADC_CTRL2, 0x0004_0010, 0),
            3 => write(ADC_CTRL3, 0x0334_0E80, 0),
            s if s < SETUP_WRITES + 2 * SUPPLY_CONVERSIONS => {
                if (s - SETUP_WRITES).is_multiple_of(2) {
                    write(AdcCtrl0::ADDR, self.ctrl0_start | selector, 60)
                } else {
                    Some(SensorStep::Read {
                        reg_addr: ADC_VOLTAGE_DATAOUT,
                        delay_ms: 0,
                    })
                }
            }
            _ => None,
        }
    }

    /// ## Supply voltage of `channel` in mV from the values read, in order
    ///
    /// `k × scale_mv × (mean / 16384 - 1)`, `mean` the DV0 average of all conversions but the first,
    /// `k` 1 on channels 0 and 1, 2 on channels 2 and 3. `None` if a conversion was not ready or the
    /// count is wrong.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_asic::adc::{SupplyAdc, SUPPLY_CONVERSIONS};
    ///
    /// let adc = SupplyAdc { ctrl0_setup: 0x230C_030D, ctrl1: 0x3F01_4381, ctrl0_start: 0x232C_030D, scale_mv: 1000.0 };
    /// let samples = [0x8000_6000u32; SUPPLY_CONVERSIONS]; // DV0 = 24576
    /// assert_eq!(adc.millivolts(0, &samples), Some(500.0));
    /// assert_eq!(adc.millivolts(2, &samples), Some(1000.0));
    /// assert_eq!(adc.millivolts(0, &[0x0000_6000; SUPPLY_CONVERSIONS]), None); // not ready
    /// assert_eq!(adc.millivolts(0, &samples[1..]), None);
    /// ```
    pub fn millivolts(&self, channel: u8, samples: &[u32]) -> Option<f32> {
        if samples.len() != SUPPLY_CONVERSIONS || channel as usize >= SELECTORS.len() {
            return None;
        }
        let used = &samples[1..];
        if used.iter().any(|s| s & 0x8000_0000 == 0) {
            return None;
        }
        let mean = used.iter().map(|s| (s & 0x7FFF) as f32).sum::<f32>() / used.len() as f32;
        let k = if channel < 2 { 1.0 } else { 2.0 };
        Some(k * self.scale_mv * (mean / 16384.0 - 1.0))
    }
}
