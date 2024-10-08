use crc::{Algorithm, Crc};

const CRC5: Crc<u8> = Crc::<u8>::new(&Algorithm {
    width: 5,
    poly: 0x05,
    init: 0x1f,
    refin: false,
    refout: false,
    xorout: 0x00,
    check: 0x00,
    residue: 0x00,
});

const CRC16: Crc<u16> = Crc::<u16>::new(&Algorithm {
    width: 16,
    poly: 0x1021,
    init: 0xffff,
    refin: false,
    refout: false,
    xorout: 0x0000,
    check: 0x29b1,
    residue: 0x0000,
});

pub const fn crc5(data: &[u8]) -> u8 {
    CRC5.checksum(data)
}

pub const fn crc5_bits(data: &[u8]) -> u8 {
    let mut var1;
    let mut var2 = true;
    let mut var3 = true;
    let mut var4;
    let mut var5 = true;
    let mut crc5 = 0x80u8;
    let mut bit_cnt = 0usize;
    let mut total_bit_cnt = 0usize;
    let data_bit_len = if data.is_empty() {
        0
    } else {
        data.len() * 8 - 5
    };
    let mut data_cnt = 0usize;
    let mut set_bit_0 = true;
    let mut set_bit_1 = true;
    let mut set_bit_2 = true;

    if data_bit_len == 0 {
        crc5 = 0x10;
    } else {
        loop {
            var4 = set_bit_2;
            set_bit_1 = set_bit_0;
            var1 = var5;
            bit_cnt += 1;
            set_bit_0 = var2;
            if (data[data_cnt] & crc5) != 0 {
                set_bit_0 = var2 ^ true;
            }
            total_bit_cnt += 1;
            crc5 >>= 1;
            if bit_cnt == 8 {
                data_cnt += 1;
            }
            set_bit_2 = var3 ^ set_bit_0;
            if bit_cnt == 8 {
                bit_cnt = 0;
                crc5 = 0x80;
            }
            var5 = var4;
            var2 = var1;
            var3 = set_bit_1;
            if total_bit_cnt >= data_bit_len {
                break;
            }
        }
        if var1 {
            crc5 = 0x10;
        } else {
            crc5 = 0;
        }
        if !var4 {
            if set_bit_2 {
                crc5 |= 4;
            }
            if set_bit_1 {
                crc5 |= 2;
            }
            if set_bit_0 {
                crc5 |= 1;
            }
            return crc5;
        }
    }
    crc5 |= 8;
    if set_bit_2 {
        crc5 |= 4;
    }
    if set_bit_1 {
        crc5 |= 2;
    }
    if set_bit_0 {
        crc5 |= 1;
    }
    crc5
}

pub const fn crc16(data: &[u8]) -> u16 {
    CRC16.checksum(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the byte-aligned CRC5.
    #[test]
    fn crc5_byte_aligned() {
        // Chain inactive
        assert_eq!(crc5(&[0x53, 0x05, 0x00, 0x00]), 0x03);
        // Read Register ChipIddentification
        assert_eq!(crc5(&[0x52, 0x05, 0x00, 0x00]), 0x0A);
        // Write Register ClockOrderControl0
        assert_eq!(
            crc5(&[0x51, 0x09, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00]),
            0x1C
        );
        // Response Register - check full frame only
        assert_eq!(
            crc5(&[0x13, 0x62, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1E]),
            0x00
        );
        // Response Nonce - check full frame only
        assert_eq!(
            crc5(&[0x2F, 0xD5, 0x96, 0xCE, 0x02, 0x93, 0x94, 0xFB, 0x86]),
            0x00
        );
    }

    /// Test the bit-aligned CRC5.
    #[test]
    fn crc5_bit_aligned() {
        // on an empty slice - equivalent to byte-aligned crc5
        assert_eq!(crc5(&[]), crc5_bits(&[]));

        // Response Register - compute
        assert_eq!(
            crc5_bits(&[0x13, 0x62, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            0x1E
        );

        // Response Nonce - compute
        assert_eq!(
            crc5_bits(&[0x2F, 0xD5, 0x96, 0xCE, 0x02, 0x93, 0x94, 0xFB, 0x80]),
            0x06
        );
    }

    /// Test the byte-aligned CRC16.
    #[test]
    fn crc16_byte_aligned() {
        // Job example - compute
        assert_eq!(
            crc16(&[
                0x21, 0x96, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x15, 0x9E, 0x07, 0x17, 0x75, 0x32,
                0x8E, 0x63, 0xA2, 0xB3, 0x6A, 0x70, 0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30, 0x1D, 0xE1,
                0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1, 0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14, 0x15, 0x03,
                0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F, 0x55, 0x74, 0xD4, 0xBA, 0xAE, 0x2F, 0x3F, 0xC6,
                0x02, 0xD9, 0xCD, 0x3B, 0x9E, 0x39, 0xAD, 0x97, 0x9C, 0xFD, 0xFF, 0x3A, 0x40, 0x49,
                0x4D, 0xB6, 0xD7, 0x8D, 0xA4, 0x51, 0x34, 0x99, 0x29, 0xD1, 0xAD, 0x36, 0x66, 0x1D,
                0xDF, 0xFF, 0xC1, 0xCC, 0x89, 0x33, 0xEA, 0xF3, 0xE8, 0x3A, 0x91, 0x58, 0xA6, 0xD6,
                0xFA, 0x02, 0x0D, 0xCF, 0x60, 0xF8, 0xC1, 0x0E, 0x99, 0x36, 0xDE, 0x71, 0xDB, 0xD3,
                0xF7, 0xD2, 0x86, 0xAF, 0xAD, 0x62, 0x59, 0x3A, 0x8D, 0xA3, 0x28, 0xAF, 0xEC, 0x09,
                0x6D, 0x86, 0xB9, 0x8E, 0x30, 0xE5, 0x79, 0xAE, 0xA4, 0x35, 0xE1, 0x4B, 0xB5, 0xD7,
                0x09, 0xCC, 0xE1, 0x74, 0x04, 0x3A, 0x7C, 0x2D
            ]),
            0x1B5C
        );
        // Job example - check full frame
        assert_eq!(
            crc16(&[
                0x21, 0x96, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x15, 0x9E, 0x07, 0x17, 0x75, 0x32,
                0x8E, 0x63, 0xA2, 0xB3, 0x6A, 0x70, 0xDE, 0x60, 0x4A, 0x09, 0xE9, 0x30, 0x1D, 0xE1,
                0x25, 0x6D, 0x7E, 0xB8, 0x0E, 0xA1, 0xE6, 0x43, 0x82, 0xDF, 0x61, 0x14, 0x15, 0x03,
                0x96, 0x6C, 0x18, 0x5F, 0x50, 0x2F, 0x55, 0x74, 0xD4, 0xBA, 0xAE, 0x2F, 0x3F, 0xC6,
                0x02, 0xD9, 0xCD, 0x3B, 0x9E, 0x39, 0xAD, 0x97, 0x9C, 0xFD, 0xFF, 0x3A, 0x40, 0x49,
                0x4D, 0xB6, 0xD7, 0x8D, 0xA4, 0x51, 0x34, 0x99, 0x29, 0xD1, 0xAD, 0x36, 0x66, 0x1D,
                0xDF, 0xFF, 0xC1, 0xCC, 0x89, 0x33, 0xEA, 0xF3, 0xE8, 0x3A, 0x91, 0x58, 0xA6, 0xD6,
                0xFA, 0x02, 0x0D, 0xCF, 0x60, 0xF8, 0xC1, 0x0E, 0x99, 0x36, 0xDE, 0x71, 0xDB, 0xD3,
                0xF7, 0xD2, 0x86, 0xAF, 0xAD, 0x62, 0x59, 0x3A, 0x8D, 0xA3, 0x28, 0xAF, 0xEC, 0x09,
                0x6D, 0x86, 0xB9, 0x8E, 0x30, 0xE5, 0x79, 0xAE, 0xA4, 0x35, 0xE1, 0x4B, 0xB5, 0xD7,
                0x09, 0xCC, 0xE1, 0x74, 0x04, 0x3A, 0x7C, 0x2D, 0x1B, 0x5C
            ]),
            0
        );
    }
}
