//! Hash counting number.
//!
//! The hash counting number sets how long a core works on its share of the chip's nonce range,
//! in cycles of the 25 MHz input clock. The chip's nonce range is `asic_addr_interval << 24` when
//! the chips are `asic_addr_interval` apart in the 8-bit address space (twice the interval for a
//! chip addressed on 7 bits), or `2^32 / chain_asic_num` when the chips split the whole range.

use fugit::HertzU64;

/// ## Hash counting number keeping `percent` of a core's half nonce range time
///
/// `⌊⌊⌊chip_nonce_range / next_pow2(core_cnt)⌋ × 25 / f⌋ × percent / 200⌋`, with `f` the hash
/// frequency in whole MHz. Each step rounds down.
///
/// ### Example
///
/// ```
/// use bm13xx_asic::hash_counting::hash_counting_number;
/// use fugit::HertzU64;
///
/// // address interval 2, 80 %
/// assert_eq!(hash_counting_number(2 << 24, 65, HertzU64::MHz(525), 80), 0x1381); // 126 × BM1362
/// assert_eq!(hash_counting_number(2 << 24, 65, HertzU64::MHz(675), 80), 0x0F2B); // 88 × BM1362
/// assert_eq!(hash_counting_number(2 << 24, 112, HertzU64::MHz(485), 80), 0x151C); // 110 × BM1366
/// assert_eq!(hash_counting_number(2 << 24, 112, HertzU64::MHz(590), 80), 0x115A); // 77 × BM1366
/// // the whole range split between the chips, 80 %, minus 800
/// let s21 = hash_counting_number((1 << 32) / 108, 80, HertzU64::MHz(490), 80);
/// assert_eq!(s21 - 800, 0x15A4); // 108 × BM1368
/// let s21_xp = hash_counting_number((1 << 32) / 91, 128, HertzU64::MHz(490), 80);
/// assert_eq!(s21_xp - 800, 0x1A44); // 91 × BM1370
/// let s21_pro = hash_counting_number((1 << 32) / 65, 128, HertzU64::MHz(596), 80);
/// assert_eq!(s21_pro - 800, 0x1EB5); // 65 × BM1370
/// ```
pub fn hash_counting_number(
    chip_nonce_range: u64,
    core_cnt: usize,
    hash_freq: HertzU64,
    percent: u32,
) -> u32 {
    let core_range = chip_nonce_range / core_cnt.next_power_of_two() as u64;
    let cycles = core_range * 25 / hash_freq.to_MHz().max(1);
    (cycles * percent as u64 / 200).min(u32::MAX as u64) as u32
}
