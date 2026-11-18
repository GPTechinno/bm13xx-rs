use crate::register::Register;

macro_rules! unknown {
    ($REG:ident, $ADDR:expr) => {
        /// # $REG register
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $REG(pub u32);
        impl_boilerplate_for!($REG);

        impl $REG {
            pub const ADDR: u8 = $ADDR;
        }

        impl core::fmt::Display for $REG {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("$REG").finish()
            }
        }

        #[cfg(feature = "defmt")]
        impl defmt::Format for $REG {
            fn format(&self, fmt: defmt::Formatter) {
                defmt::write!(fmt, "$REG {{  }}",);
            }
        }
    };
}

// Registers declared without fields: `RegXX` ones have no known meaning, the others are named but
// their fields are not implemented yet. The counters `0x88`-`0x8C`, `0x90` and `0x94` are those of the
// BM1368 and later (`0x8C` is `ClockOrderStatus` on the BM1397).
unknown!(Time1sCounter, 0x04);
unknown!(TopProcessMonitor, 0x24);
unknown!(CoreNumber, 0x30);
unknown!(Reg34, 0x34);
unknown!(Domain0GotNonceCounter, 0x88);
unknown!(Domain1GotNonceCounter, 0x89);
unknown!(Domain2GotNonceCounter, 0x8A);
unknown!(Domain3GotNonceCounter, 0x8B);
unknown!(GotNonceCounter, 0x8C);
unknown!(NtickNonceCounter, 0x90);
unknown!(TickNonceCounter, 0x94);
unknown!(RegAC, 0xAC);
unknown!(RegB0, 0xB0);
unknown!(RegB4, 0xB4);
unknown!(AdcCtrl0, 0xB8);
unknown!(AdcCtrl1, 0xB9);
unknown!(Pll0ChargePumpProgramming, 0xBC);
unknown!(Pll0OffsetCalibration, 0xC0);
unknown!(AutoWorkCfg, 0xC4);
unknown!(RegC8, 0xC8);
unknown!(RegCC, 0xCC);
unknown!(FrequencySweepControl, 0xD0);
unknown!(RegD4, 0xD4);
unknown!(RegD8, 0xD8);
unknown!(SweepNonceRetTimeout, 0xDC);
unknown!(RegE0, 0xE0);
unknown!(RegE4, 0xE4);
unknown!(RegE8, 0xE8);
unknown!(RegEC, 0xEC);
unknown!(RegF0, 0xF0);
unknown!(RegF4, 0xF4);
unknown!(RegF8, 0xF8);
unknown!(RegFC, 0xFC);
