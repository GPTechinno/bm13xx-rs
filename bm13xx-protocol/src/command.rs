//! BM13xx Protocol Commands.

use crate::crc::{crc16, crc5};

/// Some command can be send to All chip in the chain or to a specific one
pub enum Destination {
    All,
    Chip(u8),
}

pub struct Command;

impl Command {
    const CMD_ALL_CHIP: u8 = 0x10;
    const CMD_SEND_JOB: u8 = 0x21;
    const CMD_SET_CHIP_ADDR: u8 = 0x40;
    const CMD_WRITE_REGISTER: u8 = 0x41;
    const CMD_READ_REGISTER: u8 = 0x42;
    const CMD_CHAIN_INACTIVE: u8 = 0x43;

    /// # Chain Inactive Command
    ///
    /// This disable the relay ability of every chip on the chain (CI signal is no more relayed to CO pin).
    /// Usually done before setting each chip address individually using `Command::set_chip_addr`.
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::Command;
    ///
    /// let cmd = Command::chain_inactive();
    /// assert_eq!(cmd, [0x55, 0xAA, 0x53, 0x05, 0x00, 0x00, 0x03]);
    /// ```
    pub fn chain_inactive() -> [u8; 7] {
        let mut data: [u8; 7] = [
            0x55,
            0xAA,
            Self::CMD_CHAIN_INACTIVE + Self::CMD_ALL_CHIP,
            5,
            0,
            0,
            0,
        ];
        data[6] = crc5(&data[2..6]);
        data
    }

    /// # Set Chip Address Command
    ///
    /// Give a logical `ChipAddress` to the chip on the chain that does not have one yet.
    /// Usually done after sending `Command::chain_inactive` so only the first chip in the
    /// chain will receive it's chip address, then it will start relaying CI signal to
    /// CO pin and subsequent `Command::set_chip_addr` will be received by the next chip only.
    /// This way all chips will be addressed one by one and will received differents `ChipAddress`.
    ///
    /// This logical `ChipAddress` have 2 utilities :
    /// - sending command to a specific chip on the chain, using `Destination::Chip(ChipAddress)`.
    /// - when mining, the nonce space (u32) will be divided evenly according to `ChipAddress` :
    /// each chip will add it's own `ChipAddress` near by the MSB of the starting nonce for a job.
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::Command;
    ///
    /// let cmd = Command::set_chip_addr(0x00);
    /// assert_eq!(cmd, [0x55, 0xAA, 0x40, 0x05, 0x00, 0x00, 0x1C]);
    ///
    /// let cmd = Command::set_chip_addr(0x08);
    /// assert_eq!(cmd, [0x55, 0xAA, 0x40, 0x05, 0x08, 0x00, 0x07]);
    /// ```
    pub fn set_chip_addr(addr: u8) -> [u8; 7] {
        let mut data: [u8; 7] = [0x55, 0xAA, Self::CMD_SET_CHIP_ADDR, 5, addr, 0, 0];
        data[6] = crc5(&data[2..6]);
        data
    }

    /// # Read Register Command
    ///
    /// Used to send a Read Register command on the chain.
    ///
    /// All chips on the chain or only a specific one can be addressed by this command
    /// using the `dest` parameter.
    ///
    /// Usually the first command sent by a driver to the chain is the Read Register
    /// command of `ChipIdentification` to all chips on the chain, this is usefull to
    /// enumerate all chips on the chain.
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::{Command, Destination};
    ///
    /// // Enumerate the chain
    /// let cmd = Command::read_reg(0x00, Destination::All);
    /// assert_eq!(cmd, [0x55, 0xAA, 0x52, 0x05, 0x00, 0x00, 0x0A]);
    ///
    /// // Read I2CControl on chip with ChipAddress@64
    /// let cmd = Command::read_reg(0x1C, Destination::Chip(64));
    /// assert_eq!(cmd, [0x55, 0xAA, 0x42, 0x05, 0x40, 0x1C, 0x0B]);
    /// ```
    pub fn read_reg(reg_addr: u8, dest: Destination) -> [u8; 7] {
        let mut data: [u8; 7] = [0x55, 0xAA, Self::CMD_READ_REGISTER, 5, 0, reg_addr, 0];
        match dest {
            Destination::All => data[2] += Self::CMD_ALL_CHIP,
            Destination::Chip(c) => data[4] = c,
        }
        data[6] = crc5(&data[2..6]);
        data
    }

    /// # Write Register Command
    ///
    /// Used to send a Write Register command on the chain.
    ///
    /// All chips on the chain or only a specific one can be addressed by this command
    /// using the `dest` parameter.
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::{Command, Destination};
    ///
    /// // Write ClockOrderControl0 value 0x0000_0000 on All chip of the chain
    /// let cmd = Command::write_reg(0x80, 0x0000_0000, Destination::All);
    /// assert_eq!(cmd, [0x55, 0xAA, 0x51, 0x09, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x1C]);
    ///
    /// // Write MiscControl value 0x0000_7A31 on chip with ChipAddress@64
    /// let cmd = Command::write_reg(0x18, 0x0000_7A31, Destination::Chip(64));
    /// assert_eq!(cmd, [0x55, 0xAA, 0x41, 0x09, 0x40, 0x18, 0x00, 0x00, 0x7A, 0x31, 0x11]);
    /// ```
    pub fn write_reg(reg_addr: u8, reg_val: u32, dest: Destination) -> [u8; 11] {
        let mut data: [u8; 11] = [
            0x55,
            0xAA,
            Self::CMD_WRITE_REGISTER,
            9,
            0,
            reg_addr,
            ((reg_val >> 24) & 0xff) as u8,
            ((reg_val >> 16) & 0xff) as u8,
            ((reg_val >> 8) & 0xff) as u8,
            (reg_val & 0xff) as u8,
            0,
        ];
        match dest {
            Destination::All => data[2] += Self::CMD_ALL_CHIP,
            Destination::Chip(c) => data[4] = c,
        }
        data[10] = crc5(&data[2..10]);
        data
    }

    /// # Job with 1 Midstate Command
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::Command;
    ///
    /// let midstates = [
    ///     [
    ///         0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30, 0x1D, 0xE1, 0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1,
    ///         0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14, 0x15, 0x03, 0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F,
    ///         0x55, 0x74, 0xD4, 0xBA,
    ///     ],
    /// ];
    /// let cmd = Command::job_1_midstate(0, 0x1707_9E15, 0x638E_3275, 0x706A_B3A2, midstates);
    /// assert_eq!(
    ///     cmd,
    ///     [
    ///         0x55, 0xAA, 0x21, 0x36, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x15, 0x9E, 0x07, 0x17,
    ///         0x75, 0x32, 0x8E, 0x63, 0xA2, 0xB3, 0x6A, 0x70, 0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30,
    ///         0x1D, 0xE1, 0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1, 0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14,
    ///         0x15, 0x03, 0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F, 0x55, 0x74, 0xD4, 0xBA, 0xD3, 0xDC
    ///     ]
    /// );
    /// ```
    pub fn job_1_midstate(
        job_id: u8,
        n_bits: u32,
        n_time: u32,
        merkle_root: u32,
        midstates: [[u8; 32]; 1],
    ) -> [u8; 56] {
        let mut data = [0; 56];
        data[0] = 0x55;
        data[1] = 0xAA;
        data[2] = Self::CMD_SEND_JOB;
        // data[3] = 22 + (midstates.len() * 32) as u8;
        data[3] = data.len() as u8 - 2;
        data[4] = job_id;
        data[5] = midstates.len() as u8;
        // data[6..].clone_from_slice(&0u32.to_le_bytes()); // starting_nonce ?
        data[10..14].clone_from_slice(&n_bits.to_le_bytes());
        data[14..18].clone_from_slice(&n_time.to_le_bytes());
        data[18..22].clone_from_slice(&merkle_root.to_le_bytes());
        let mut offset = 22;
        for ms in midstates.iter() {
            data[offset..offset + ms.len()].clone_from_slice(ms);
            offset += ms.len();
        }
        let crc = crc16(&data[2..offset]);
        data[offset..offset + 2].clone_from_slice(&crc.to_be_bytes());
        data
    }

    /// # Job with 4 Midstate Command
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::Command;
    ///
    /// let midstates = [
    ///     [
    ///         0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30, 0x1D, 0xE1, 0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1,
    ///         0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14, 0x15, 0x03, 0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F,
    ///         0x55, 0x74, 0xD4, 0xBA,
    ///     ],
    ///     [
    ///         0xAE, 0x2F, 0x3F, 0xC6, 0x02, 0xD9, 0xCD, 0x3B, 0x9E, 0x39, 0xAD, 0x97, 0x9C, 0xFD,
    ///         0xFF, 0x3A, 0x40, 0x49, 0x4D, 0xB6, 0xD7, 0x8D, 0xA4, 0x51, 0x34, 0x99, 0x29, 0xD1,
    ///         0xAD, 0x36, 0x66, 0x1D,
    ///     ],
    ///     [
    ///         0xDF, 0xFF, 0xC1, 0xCC, 0x89, 0x33, 0xEA, 0xF3, 0xE8, 0x3A, 0x91, 0x58, 0xA6, 0xD6,
    ///         0xFA, 0x02, 0x0D, 0xCF, 0x60, 0xF8, 0xC1, 0x0E, 0x99, 0x36, 0xDE, 0x71, 0xDB, 0xD3,
    ///         0xF7, 0xD2, 0x86, 0xAF,
    ///     ],
    ///     [
    ///         0xAD, 0x62, 0x59, 0x3A, 0x8D, 0xA3, 0x28, 0xAF, 0xEC, 0x09, 0x6D, 0x86, 0xB9, 0x8E,
    ///         0x30, 0xE5, 0x79, 0xAE, 0xA4, 0x35, 0xE1, 0x4B, 0xB5, 0xD7, 0x09, 0xCC, 0xE1, 0x74,
    ///         0x04, 0x3A, 0x7C, 0x2D,
    ///     ],
    /// ];
    /// let cmd = Command::job_4_midstate(0, 0x1707_9E15, 0x638E_3275, 0x706A_B3A2, midstates);
    /// assert_eq!(
    ///     cmd,
    ///     [
    ///         0x55, 0xAA, 0x21, 0x96, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x15, 0x9E, 0x07, 0x17,
    ///         0x75, 0x32, 0x8E, 0x63, 0xA2, 0xB3, 0x6A, 0x70, 0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30,
    ///         0x1D, 0xE1, 0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1, 0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14,
    ///         0x15, 0x03, 0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F, 0x55, 0x74, 0xD4, 0xBA, 0xAE, 0x2F,
    ///         0x3F, 0xC6, 0x02, 0xD9, 0xCD, 0x3B, 0x9E, 0x39, 0xAD, 0x97, 0x9C, 0xFD, 0xFF, 0x3A,
    ///         0x40, 0x49, 0x4D, 0xB6, 0xD7, 0x8D, 0xA4, 0x51, 0x34, 0x99, 0x29, 0xD1, 0xAD, 0x36,
    ///         0x66, 0x1D, 0xDF, 0xFF, 0xC1, 0xCC, 0x89, 0x33, 0xEA, 0xF3, 0xE8, 0x3A, 0x91, 0x58,
    ///         0xA6, 0xD6, 0xFA, 0x02, 0x0D, 0xCF, 0x60, 0xF8, 0xC1, 0x0E, 0x99, 0x36, 0xDE, 0x71,
    ///         0xDB, 0xD3, 0xF7, 0xD2, 0x86, 0xAF, 0xAD, 0x62, 0x59, 0x3A, 0x8D, 0xA3, 0x28, 0xAF,
    ///         0xEC, 0x09, 0x6D, 0x86, 0xB9, 0x8E, 0x30, 0xE5, 0x79, 0xAE, 0xA4, 0x35, 0xE1, 0x4B,
    ///         0xB5, 0xD7, 0x09, 0xCC, 0xE1, 0x74, 0x04, 0x3A, 0x7C, 0x2D, 0x1B, 0x5C
    ///     ]
    /// );
    /// ```
    pub fn job_4_midstate(
        job_id: u8,
        n_bits: u32,
        n_time: u32,
        merkle_root: u32,
        midstates: [[u8; 32]; 4],
    ) -> [u8; 152] {
        let mut data = [0; 152];
        data[0] = 0x55;
        data[1] = 0xAA;
        data[2] = Self::CMD_SEND_JOB;
        // data[3] = 22 + (midstates.len() * 32) as u8;
        data[3] = data.len() as u8 - 2;
        data[4] = job_id;
        data[5] = midstates.len() as u8;
        // data[6..].clone_from_slice(&0u32.to_le_bytes()); // starting_nonce ?
        data[10..14].clone_from_slice(&n_bits.to_le_bytes());
        data[14..18].clone_from_slice(&n_time.to_le_bytes());
        data[18..22].clone_from_slice(&merkle_root.to_le_bytes());
        let mut offset = 22;
        for ms in midstates.iter() {
            data[offset..offset + ms.len()].clone_from_slice(ms);
            offset += ms.len();
        }
        let crc = crc16(&data[2..offset]);
        data[offset..offset + 2].clone_from_slice(&crc.to_be_bytes());
        data
    }

    /// # Job with Header (for Hardware Version Rolling) Command
    ///
    /// ## Example
    /// ```
    /// use bm13xx_protocol::command::Command;
    ///
    /// let merkle_root = [0x2d, 0x19, 0x75, 0x74, 0x66, 0x63, 0x21, 0x46, 0xb8, 0x71, 0x7a, 0x7e,
    ///         0xfe, 0x83, 0xec, 0x35, 0xc0, 0x96, 0xf3, 0xa4, 0xc0, 0xd8, 0x86, 0xda, 0xa8, 0x0e,
    ///         0x70, 0x2e, 0xed, 0xe9, 0x96, 0x71];
    /// let prev_hash = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x86, 0x02, 0x00,
    ///         0x5b, 0xa4, 0xa5, 0x0e, 0x55, 0xd3, 0x00, 0xfc, 0xae, 0x0e, 0xd5, 0x56, 0xd7, 0x76,
    ///         0xd8, 0x1a, 0x38, 0xe1, 0x99, 0x1f];
    /// let header = Command::job_header(168, 0x1704_2450, 0x6570_de83, merkle_root, prev_hash, 0x2000_0000);
    /// assert_eq!(
    ///     header,
    ///     [
    ///         0x55, 0xaa, 0x21, 0x36, 0xa8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x50, 0x24, 0x04, 0x17,
    ///         0x83, 0xde, 0x70, 0x65, 0x2d, 0x19, 0x75, 0x74, 0x66, 0x63, 0x21, 0x46, 0xb8, 0x71,
    ///         0x7a, 0x7e, 0xfe, 0x83, 0xec, 0x35, 0xc0, 0x96, 0xf3, 0xa4, 0xc0, 0xd8, 0x86, 0xda,
    ///         0xa8, 0x0e, 0x70, 0x2e, 0xed, 0xe9, 0x96, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ///         0x00, 0x00, 0xff, 0x86, 0x02, 0x00, 0x5b, 0xa4, 0xa5, 0x0e, 0x55, 0xd3, 0x00, 0xfc,
    ///         0xae, 0x0e, 0xd5, 0x56, 0xd7, 0x76, 0xd8, 0x1a, 0x38, 0xe1, 0x99, 0x1f, 0x00, 0x00,
    ///         0x00, 0x20, 0x30, 0xb9,
    ///     ]);
    /// ```
    pub fn job_header(
        job_id: u8,
        n_bits: u32,
        n_time: u32,
        full_merkle_root: [u8; 32],
        prev_block_header_hash: [u8; 32],
        version: u32,
    ) -> [u8; 88] {
        let mut data = [0; 88];
        data[0] = 0x55;
        data[1] = 0xAA;
        data[2] = Self::CMD_SEND_JOB;
        // data[3] = 54;
        data[3] = data.len() as u8 - 32 - 2;
        data[4] = job_id;
        data[5] = 1;
        // data[6..].clone_from_slice(&0u32.to_le_bytes()); // starting_nonce ?
        data[10..14].clone_from_slice(&n_bits.to_le_bytes());
        data[14..18].clone_from_slice(&n_time.to_le_bytes());
        data[18..50].clone_from_slice(&full_merkle_root);
        data[50..82].clone_from_slice(&prev_block_header_hash);
        data[82..86].clone_from_slice(&version.to_le_bytes());
        let crc = crc16(&data[2..86]);
        data[86..88].clone_from_slice(&crc.to_be_bytes());
        data
    }
}
