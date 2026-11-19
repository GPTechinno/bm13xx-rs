//! Hash counting number.
//!
//! The hash counting number sets how long a core works on its share of the nonce range. It is a
//! percentage of the time a core takes to cover that share:
//! `(asic_addr_interval << 24) / next_pow2(core_cnt) * 25 MHz / (2 * hash_freq)`, with the
//! interval between chip addresses in the 8-bit address space. A chip whose nonce range is split
//! over fewer address bits uses an interval scaled up accordingly (twice the interval for 7 bits).

use fugit::HertzU64;

/// ## Hash counting number keeping `percent` of the core nonce range time
///
/// ### Example
///
/// ```
/// use bm13xx_asic::hash_counting::hash_counting_number;
/// use fugit::HertzU64;
///
/// // 110 chips, interval 2, 112 cores, 485 MHz, 80 % minus 1
/// assert_eq!(hash_counting_number(2, 112, HertzU64::MHz(485), 80) - 1, 0x151C);
/// // 77 chips, interval 2, 112 cores, 590 MHz, 80 % minus 1
/// assert_eq!(hash_counting_number(2, 112, HertzU64::MHz(590), 80) - 1, 0x115A);
/// // 91 chips, interval 2, 128 cores, 448 MHz, 85 %
/// assert_eq!(hash_counting_number(2, 128, HertzU64::MHz(448), 85), 0x1849);
/// // a single chip, 128 cores, 525 MHz, 85 %
/// assert_eq!(hash_counting_number(256, 128, HertzU64::MHz(525), 85), 679_077);
/// ```
pub fn hash_counting_number(
    asic_addr_interval: usize,
    core_cnt: usize,
    hash_freq: HertzU64,
    percent: u32,
) -> u32 {
    let core_share = ((asic_addr_interval as u64) << 24) / core_cnt.next_power_of_two() as u64;
    // 25 MHz / 2 / 100 % = 125 000
    (core_share * 125_000 * percent as u64 / hash_freq.to_raw().max(1)).min(u32::MAX as u64) as u32
}
