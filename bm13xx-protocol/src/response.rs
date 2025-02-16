//! BM13xx Protocol Responses.

use crate::crc::{crc5, crc5_bits};
use crate::{Error, Result};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct RegisterResponse {
    pub chip_addr: u8,
    pub reg_addr: u8,
    pub reg_value: u32,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct JobResponse {
    pub nonce: u32,
    pub job_id: usize,
    pub midstate_id: usize,
    pub small_core_id: usize,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct JobVersionResponse {
    pub nonce: u32,
    pub unknown: u8,
    pub job_id: usize,
    pub small_core_id: usize,
    pub version_bit: u32,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum ResponseType {
    Reg(RegisterResponse),
    Job(JobResponse),
    JobVer(JobVersionResponse),
}

pub const FRAME_SIZE: usize = 9;
pub const FRAME_SIZE_VER: usize = 11;

#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
    /// - `Err(Error::UnsupportedCoreSmallCoreCnt)` if core_small_core_cnt is not 4, 8 or 16.
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
    /// let resp = Response::parse(&[0x00,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x06], 4);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse(&[0xAA,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x06], 4);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse(&[0x00,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x06], 4);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// // Error::UnsupportedCoreSmallCoreCnt
    /// let resp = Response::parse(&[0xAA,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x06], 5);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::UnsupportedCoreSmallCoreCnt);
    ///
    /// // Error::InvalidCrc
    /// let resp = Response::parse(&[0xAA,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x00], 4); // should be 0x06
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidCrc { expected: 0x06, actual: 0x00 });
    ///
    /// // ChipIdentification == 0x13971800
    /// let resp = Response::parse(&[0xAA,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x06], 4);
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
    /// let resp = Response::parse(&[0xAA,0x55,0x97,0xC3,0x28,0xB6,0x01,0x63,0x9C], 4);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::Job(j) => {
    ///         assert_eq!(j.nonce, 0xB628_C397);
    ///         assert_eq!(j.midstate_id, 1);
    ///         assert_eq!(j.job_id, 24);
    ///         assert_eq!(j.small_core_id, 3);
    ///     },
    ///     _ => panic!(),
    /// };
    /// ```
    pub fn parse(data: &[u8; FRAME_SIZE], core_small_core_cnt: usize) -> Result<ResponseType> {
        if data[0] != 0xAA || data[1] != 0x55 {
            return Err(Error::InvalidPreamble);
        }
        if core_small_core_cnt != 4 && core_small_core_cnt != 8 && core_small_core_cnt != 16 {
            return Err(Error::UnsupportedCoreSmallCoreCnt);
        }
        if crc5(&data[2..9]) != 0x00 {
            return Err(Error::InvalidCrc {
                expected: crc5_bits(&data[2..9]),
                actual: data[8] & 0x1f,
            });
        }
        if data[8] & 0x80 == 0x80 {
            let small_core_mask = core_small_core_cnt - 1;
            let small_core_bits = small_core_mask.count_ones();
            return Ok(ResponseType::Job(JobResponse {
                nonce: u32::from_le_bytes(data[2..6].try_into().unwrap()),
                midstate_id: data[6] as usize,
                job_id: ((data[7] as usize) >> small_core_bits) & 0b1_1111,
                small_core_id: (data[7] as usize) & small_core_mask,
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
    /// - `Err(Error::UnsupportedCoreSmallCoreCnt)` if core_small_core_cnt is not 8 or 16.
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
    /// let resp = Response::parse_version(&[0x00,0x55,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06], 8);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse_version(&[0xAA,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06], 8);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// let resp = Response::parse_version(&[0x00,0x00,0x13,0x97,0x18,0x00,0x00,0x00,0x00,0x00,0x06], 8);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidPreamble);
    ///
    /// // Error::UnsupportedCoreSmallCoreCnt
    /// let resp = Response::parse_version(&[0xAA,0x55,0x2F,0xD5,0x96,0xCE,0x02,0x93,0x94,0xFB,0x86], 9);
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::UnsupportedCoreSmallCoreCnt);
    ///
    /// // Error::InvalidCrc
    /// let resp = Response::parse_version(&[0xAA,0x55,0x13,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00], 8); // should be 0x05
    /// assert!(resp.is_err());
    /// assert_eq!(resp.unwrap_err(), Error::InvalidCrc { expected: 0x05, actual: 0x00 });
    ///
    /// // ChipIdentification == 0x13660000
    /// let resp = Response::parse_version(&[0xAA,0x55,0x13,0x62,0x03,0x00,0x00,0x00,0x00,0x00,0x1E], 8);
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
    /// let resp = Response::parse_version(&[0xAA,0x55,0x2F,0xD5,0x96,0xCE,0x02,0x93,0x94,0xFB,0x86], 8);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::JobVer(j) => {
    ///         assert_eq!(j.nonce, 0xCE96_D52F);
    ///         assert_eq!(j.unknown, 2);
    ///         assert_eq!(j.job_id, 18);
    ///         assert_eq!(j.small_core_id, 3);
    ///         assert_eq!(j.version_bit, 0x129F_6000);
    ///     },
    ///     _ => panic!(),
    /// };
    ///
    /// let resp = Response::parse_version(&[0xAA,0x55,0x07,0x35,0xCD,0xCF,0x02,0x5E,0x00,0x2E,0x96], 16);
    /// assert!(resp.is_ok());
    /// match resp.unwrap() {
    ///     ResponseType::JobVer(j) => {
    ///         assert_eq!(j.nonce, 0xCFCD_3507);
    ///         assert_eq!(j.unknown, 1);
    ///         assert_eq!(j.job_id, 5);
    ///         assert_eq!(j.small_core_id, 14);
    ///         assert_eq!(j.version_bit, 0x0005_C000);
    ///     },
    ///     _ => panic!(),
    /// };
    /// ```
    pub fn parse_version(
        data: &[u8; FRAME_SIZE_VER],
        core_small_core_cnt: usize,
    ) -> Result<ResponseType> {
        if data[0] != 0xAA || data[1] != 0x55 {
            return Err(Error::InvalidPreamble);
        }
        if core_small_core_cnt != 8 && core_small_core_cnt != 16 {
            return Err(Error::UnsupportedCoreSmallCoreCnt);
        }
        if crc5(&data[2..11]) != 0x00 {
            return Err(Error::InvalidCrc {
                expected: crc5_bits(&data[2..11]),
                actual: data[10] & 0x1f,
            });
        }
        if data[10] & 0x80 == 0x80 {
            let small_core_mask = core_small_core_cnt - 1;
            let small_core_bits = small_core_mask.count_ones();
            let chunk = ((data[6] as u16) << 8) | data[7] as u16;
            return Ok(ResponseType::JobVer(JobVersionResponse {
                nonce: u32::from_le_bytes(data[2..6].try_into().unwrap()),
                unknown: (chunk >> (small_core_bits + 5)) as u8,
                job_id: ((chunk >> small_core_bits) as usize) & 0b1_1111,
                small_core_id: (data[7] as usize) & small_core_mask,
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
