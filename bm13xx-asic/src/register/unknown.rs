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

unknown!(Reg24, 0x24);
unknown!(Reg30, 0x30);
unknown!(Reg34, 0x34);
unknown!(RegA8, 0xA8);
unknown!(RegAC, 0xAC);
unknown!(RegB0, 0xB0);
unknown!(RegB4, 0xB4);
unknown!(RegB8, 0xB8);
unknown!(RegBC, 0xBC);
unknown!(RegC0, 0xC0);
unknown!(RegC4, 0xC4);
unknown!(RegC8, 0xC8);
unknown!(RegCC, 0xCC);
unknown!(RegD0, 0xD0);
unknown!(RegD4, 0xD4);
unknown!(RegD8, 0xD8);
unknown!(RegDC, 0xDC);
unknown!(RegE0, 0xE0);
unknown!(RegE4, 0xE4);
unknown!(RegE8, 0xE8);
unknown!(RegEC, 0xEC);
unknown!(RegF0, 0xF0);
unknown!(RegF4, 0xF4);
unknown!(RegF8, 0xF8);
unknown!(RegFC, 0xFC);
