//! BM13xx Protocol Responses.

use crate::crc::{crc5, crc5_bits};
use crate::{Error, Result};

#[derive(Debug, PartialEq)]
pub struct RegisterResponse {
    pub chip_addr: u8,
    pub reg_addr: u8,
    pub reg_value: u32,
}

#[derive(Debug)]
pub struct JobResponse {
    pub nonce: u32,
    pub job_id: u8,
    pub midstate_id: u8,
}

#[derive(Debug)]
pub struct JobVersionResponse {
    pub nonce: u32,
    pub job_id: u8,
    pub midstate_id: u8,
    pub version_bit: u32,
}

#[derive(Debug)]
pub enum ResponseType {
    Reg(RegisterResponse),
    Job(JobResponse),
    JobVer(JobVersionResponse),
}

pub struct Response;

impl Response {
    /// # Parse Response
    ///
    /// Parse raw bytes from RO signal of BM13xx with Version Rolling disabled.
    ///
    /// The packet must have a lenght of 9 bytes.
    ///
    /// ## Return
    /// - `Err(Error::InvalidPreamble)` if it first 2 bytes are not `[0xAA, 0x55]`.
    /// - `Err(Error::InvalidCrc)` if the CRC5 is not valid.
    /// - `Ok(ResponseType::Reg(r))` with the `RegisterResponse`.
    /// - `Ok(ResponseType::Job(j))` with the `JobResponse`.
    ///
    /// ## Example
    ///
    /// ```
    /// use bm13xx_protocol::Error;
    /// use bm13xx_protocol::response::{Response, ResponseType};
    ///
    /// // Error::InvalidPreamble
    /// let resp = Response::parse(&[0x00,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse(&[0xAA,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse(&[0x00,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// // Error::InvalidCrc
    /// let resp = Response::parse(&[0xAA,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x00]); // should be 0x06
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidCrc { expected: 0x06, actual: 0x00 });
    ///
    /// // ChipIdentification == 0x13971800
    /// let resp = Response::parse(&[0xAA,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::Reg(r) => {
    ///         assert_eq!(r.chip_addr, 0);
    ///         assert_eq!(r.reg_addr, 0x00);
    ///         assert_eq!(r.reg_value, 0x1397_1800);
    ///     },
    ///     _ => panic!(),
    /// };
    ///
    /// let resp = Response::parse(&[0xAA,0x55,0x97,0xC3,0x28,0xB6,0x01,0x63,0x9C]);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::Job(j) => {
    ///         assert_eq!(j.nonce, 0x97C3_28B6);
    ///         assert_eq!(j.midstate_id, 1);
    ///         assert_eq!(j.job_id, 0x63);
    ///     },
    ///     _ => panic!(),
    /// };
    /// ```
    pub fn parse(data: &[u8; 9]) -> Result<ResponseType> {
        if data[0] != 0xAA || data[1] != 0x55 {
            return Err(Error::InvalidPreamble);
        }
        if crc5(&data[2..9]) != 0x00 {
            return Err(Error::InvalidCrc {
                expected: crc5_bits(&data[2..9]),
                actual: data[8] & 0x1f,
            });
        }
        if data[8] & 0x80 == 0x80 {
            return Ok(ResponseType::Job(JobResponse {
                nonce: u32::from_be_bytes(data[2..6].try_into().unwrap()),
                midstate_id: data[6],
                job_id: data[7],
            }));
        }
        Ok(ResponseType::Reg(RegisterResponse {
            chip_addr: data[6],
            reg_addr: data[7],
            reg_value: u32::from_be_bytes(data[2..6].try_into().unwrap()),
        }))
    }

    /// # Parse Version Response
    ///
    /// Parse raw bytes from RO signal of BM13xx with Version Rolling enabled.
    ///
    /// The packet must have a lenght of 11 bytes.
    ///
    /// ## Return
    /// - `Err(Error::InvalidPreamble)` if it first 2 bytes are not `[0xAA, 0x55]`.
    /// - `Err(Error::InvalidCrc)` if the CRC5 is not valid.
    /// - `Ok(ResponseType::Reg(r))` with the `RegisterResponse`.
    /// - `Ok(ResponseType::JobVer(j))` with the `JobVersionResponse`.
    ///
    /// ## Example
    ///
    /// ```
    /// use bm13xx_protocol::Error;
    /// use bm13xx_protocol::response::{Response, ResponseType};
    ///
    /// // Error::InvalidPreamble
    /// let resp = Response::parse_version(&[0x00,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse_version(&[0xAA,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse_version(&[0x00,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06]);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// // Error::InvalidCrc
    /// let resp = Response::parse_version(&[0xAA,0x55,0x13,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00]); // should be 0x05
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidCrc { expected: 0x05, actual: 0x00 });
    ///
    /// // ChipIdentification == 0x13660000
    /// let resp = Response::parse_version(&[0xAA,0x55,0x13,0x62,0x03,0x00,0x00,0x00,0x00,0x00,0x1E]);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::Reg(r) => {
    ///         assert_eq!(r.chip_addr, 0);
    ///         assert_eq!(r.reg_addr, 0x00);
    ///         assert_eq!(r.reg_value, 0x1362_0300);
    ///     },
    ///     _ => panic!(),
    /// };
    ///
    /// let resp = Response::parse_version(&[0xAA,0x55,0x2F,0xD5,0x96,0xCE,0x02,0x93,0x94,0xFB,0x86]);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::JobVer(j) => {
    ///         assert_eq!(j.nonce, 0x2FD5_96CE);
    ///         assert_eq!(j.midstate_id, 2);
    ///         assert_eq!(j.job_id, 0x93);
    ///         assert_eq!(j.version_bit, 0x129F_6000);
    ///     },
    ///     _ => panic!(),
    /// };
    /// ```
    pub fn parse_version(data: &[u8; 11]) -> Result<ResponseType> {
        if data[0] != 0xAA || data[1] != 0x55 {
            return Err(Error::InvalidPreamble);
        }
        if crc5(&data[2..11]) != 0x00 {
            return Err(Error::InvalidCrc {
                expected: crc5_bits(&data[2..11]),
                actual: data[10] & 0x1f,
            });
        }
        if data[10] & 0x80 == 0x80 {
            return Ok(ResponseType::JobVer(JobVersionResponse {
                nonce: u32::from_be_bytes(data[2..6].try_into().unwrap()),
                midstate_id: data[6],
                job_id: data[7],
                version_bit: (u16::from_be_bytes(data[8..10].try_into().unwrap()) as u32) << 13,
            }));
        }
        Ok(ResponseType::Reg(RegisterResponse {
            chip_addr: data[6],
            reg_addr: data[7],
            reg_value: u32::from_be_bytes(data[2..6].try_into().unwrap()),
        }))
    }
}
