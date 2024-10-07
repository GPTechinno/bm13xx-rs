use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error {
    // -- register
    UnknownRegister { reg_addr: u8 },
}

#[cfg(feature = "core-error")]
impl core::error::Error for Error {}

#[cfg(feature = "core-error")]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
