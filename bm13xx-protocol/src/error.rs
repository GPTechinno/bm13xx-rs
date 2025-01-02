use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error {
    // -- response
    InvalidPreamble,
    InvalidCrc { expected: u8, actual: u8 },
}

#[rustversion::since(1.81)]
impl core::error::Error for Error {}

#[rustversion::since(1.81)]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
