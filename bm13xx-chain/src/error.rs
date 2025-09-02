use bm13xx_asic::register::ChipIdentification;
use bm13xx_protocol::response::{RegisterResponse, ResponseType};
use derive_more::From;

pub type Result<T, IO, B, R> = core::result::Result<T, Error<IO, B, R>>;

#[derive(PartialEq, From)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<IO, B, R> {
    /// We received a response from ASIC which does not correspond to the command sent
    UnexpectedResponse { resp: ResponseType },
    /// We received a register response which does not correspond to the register read command
    BadRegisterResponse { reg_resp: RegisterResponse },
    /// We enumerated an ASIC which does not correspond to the chip we are looking for
    UnexpectedAsic { chip_ident: ChipIdentification },
    /// We enumerated an empty chain
    EmptyChain,
    /// The BM13xx protocol returned an error
    #[from]
    Protocol(bm13xx_protocol::Error),
    /// The serial interface returned an error
    Io(IO),
    /// The gpio interface returned an error on Busy signal
    Busy(B),
    /// The gpio interface returned an error on Reset signal
    Reset(R),
    /// The serial interface returned an error while setting baudrate
    SetBaudrate,
}

#[rustversion::since(1.81)]
impl<IO: core::fmt::Debug, B: core::fmt::Debug, R: core::fmt::Debug> core::error::Error
    for Error<IO, B, R>
{
}

#[rustversion::since(1.81)]
impl<IO: core::fmt::Debug, B: core::fmt::Debug, R: core::fmt::Debug> core::fmt::Display
    for Error<IO, B, R>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<IO: core::fmt::Debug, B: core::fmt::Debug, R: core::fmt::Debug> core::fmt::Debug
    for Error<IO, B, R>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::UnexpectedResponse { resp } => f
                .debug_struct("UnexpectedResponse")
                .field("resp", &format_args!("{:x?}", resp))
                .finish(),
            Error::BadRegisterResponse { reg_resp } => f
                .debug_struct("BadRegisterResponse")
                .field("reg_resp", &format_args!("{:x?}", reg_resp))
                .finish(),
            Error::UnexpectedAsic { chip_ident } => f
                .debug_struct("UnexpectedAsic")
                .field("chip_ident", &format_args!("{:x?}", chip_ident))
                .finish(),
            Error::EmptyChain => f.debug_struct("EmptyChain").finish(),
            Error::Protocol(protocol_err) => f.debug_tuple("Protocol").field(protocol_err).finish(),
            Error::Io(io_err) => f.debug_tuple("Io").field(io_err).finish(),
            Error::Busy(gpio_err) => f.debug_tuple("Busy").field(gpio_err).finish(),
            Error::Reset(gpio_err) => f.debug_tuple("Reset").field(gpio_err).finish(),
            Error::SetBaudrate => f.debug_struct("SetBaudrate").finish(),
        }
    }
}
