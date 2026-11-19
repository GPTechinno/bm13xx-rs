//! Die temperature conversions.

/// ## Temperature sensor `DOUT` to °C
///
/// For the temperature sensor data registers ([`TempSensorData`](crate::register::TempSensorData),
/// [`TempSensorData0`](crate::register::TempSensorData0)).
///
/// ### Example
///
/// ```
/// use bm13xx_asic::temperature::ts_dout_to_celsius;
///
/// assert!((ts_dout_to_celsius(1884) - 25.09).abs() < 0.01);
/// assert!((ts_dout_to_celsius(2333) - 99.99).abs() < 0.01);
/// ```
pub fn ts_dout_to_celsius(dout: u16) -> f32 {
    dout as f32 * 683.254 / 4096.0 - 289.18
}

/// ## ADC die temperature to °C
///
/// From two series of ADC conversions read in `ADC_VOLTAGE_DATAOUT` (`0xBD`): `dv0` holds the
/// `DV0` values (bits 14:0) of the first series, `dv1` the `DV1` values (bits 30:16) of the second
/// one. The first value of each series is discarded and the others are averaged.
///
/// ### Example
///
/// ```
/// use bm13xx_asic::temperature::adc_to_celsius;
///
/// // a = 8000, b = 8100: code = 8000 + 16384 - 8050 = 16334
/// let t = adc_to_celsius(&[0, 8000, 8000], &[0, 8100, 8100]).unwrap();
/// assert!((t - (16334.0 * 0.0309 - 769.25)).abs() < 0.001);
/// assert_eq!(adc_to_celsius(&[0], &[0, 8100]), None);
/// ```
pub fn adc_to_celsius(dv0: &[u16], dv1: &[u16]) -> Option<f32> {
    if dv0.len() < 2 || dv1.len() < 2 {
        return None;
    }
    let a = dv0[1..].iter().map(|&v| v as u32).sum::<u32>() / (dv0.len() as u32 - 1);
    let b = dv1[1..].iter().map(|&v| v as u32).sum::<u32>() / (dv1.len() as u32 - 1);
    let code = a + 16384 - ((a + b) >> 1);
    Some(code as f32 * 0.0309 - 769.25)
}
