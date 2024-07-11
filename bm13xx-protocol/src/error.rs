use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, From)]
pub enum Error {
    // -- response
    InvalidPreamble,
    InvalidCrc { expected: u8, actual: u8 },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "{self:?}")
    }
}
