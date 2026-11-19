//! Per-chip hash frequency tuning.
//!
//! [`FreqSweep`] decides, round after round, the hash frequency of each chip of a chain from the
//! nonces each chip returned at its current frequency. It does no I/O. A round is:
//! 1. apply [`FreqSweep::targets`] with [`Chain::set_chip_hash_freqs`](crate::Chain::set_chip_hash_freqs),
//! 2. clear the counts ([`Chain::clear_chip_nonce_counts`](crate::Chain::clear_chip_nonce_counts)),
//!    send work for a while,
//! 3. give [`FreqSweep::round`] the nonces each chip returned
//!    ([`Chain::chip_nonce_counts`](crate::Chain::chip_nonce_counts)) and the nonces expected in that
//!    time ([`Chain::expected_chip_nonces`](crate::Chain::expected_chip_nonces)).
//!
//! The first round sets the limits from the chain's nonce rate at the starting frequencies. Each
//! later round raises by one step the chips that kept their nonce rate above the chip limit; a chip
//! that fell below goes back `back_off_steps` steps and stays there. The sweep ends when the chain's
//! rate falls below the chain limit (every chip back to its last good frequency), when the average
//! frequencies of the voltage domains spread too far apart, when every chip is done, when the
//! average frequency reaches the target, or when the rate-weighted average frequency stops rising.
//!
//! [`DomainGuard`] watches the voltage domains while hashing and slows down one that falls behind
//! the others.
//!
//! A nonce rate needs enough nonces to be meaningful: a lower ticket mask difficulty during the
//! sweep gives more of them (about 10 000 nonces per chip for a rate within 1 %).
//!
//! ### Example
//!
//! ```
//! use bm13xx_chain::tuning::{FreqSweep, SweepConfig, SweepState};
//! use fugit::HertzU64;
//!
//! // 4 chips in 2 voltage domains, each chip loses nonces above its own limit
//! let limits = [475, 500, 525, 550];
//! let mut sweep = FreqSweep::<4>::new(
//!     SweepConfig::new(HertzU64::MHz(600)),
//!     &[HertzU64::MHz(450); 4],
//!     2,
//! );
//! loop {
//!     // hash at sweep.targets() for a while, then count the nonces of each chip
//!     let nonces: Vec<u32> = sweep
//!         .targets()
//!         .iter()
//!         .zip(limits)
//!         .map(|(f, l)| if f.to_MHz() <= l { 10_000 } else { 9_700 })
//!         .collect();
//!     if sweep.round(&nonces, &[10_000.0; 4]).unwrap() == SweepState::Done {
//!         break;
//!     }
//! }
//! assert!(sweep.targets().iter().zip(limits).all(|(f, l)| f.to_MHz() <= l));
//! ```

use fugit::HertzU64;
use heapless::Vec;

/// # Settings of a frequency sweep
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SweepConfig {
    /// Frequency step of a chip between two rounds.
    pub step: HertzU64,
    /// Steps a chip goes back when its nonce rate falls below the chip limit.
    pub back_off_steps: u32,
    /// Chip limit, in points of percent from the chain's nonce rate in the first round.
    pub chip_rate_deviation: f32,
    /// Chain limit, in points of percent from the chain's nonce rate in the first round.
    pub chain_rate_deviation: f32,
    /// Largest spread allowed between the average frequencies of the voltage domains, in percent.
    pub domain_spread: f32,
    /// Lowest chain nonce rate, in percent, accepted in the first round.
    pub min_start_rate: f32,
    /// Rounds without a gain of the rate-weighted average frequency that end the sweep.
    pub max_flat_rounds: u32,
    /// Average chip frequency that ends the sweep.
    pub target_avg_freq: HertzU64,
    /// Highest frequency a chip gets.
    pub max_freq: HertzU64,
}

impl SweepConfig {
    /// ## Settings for a target average frequency
    ///
    /// Steps of 6.25 MHz, 2 steps back, chip limit 0.8 point and chain limit 1 point below the
    /// chain's first rate, domains within 20 %, a first rate of 80 % at least, 2 rounds without
    /// gain, no chip above the target plus 25 %.
    ///
    /// ### Example
    ///
    /// ```
    /// use bm13xx_chain::tuning::SweepConfig;
    /// use fugit::HertzU64;
    ///
    /// let cfg = SweepConfig::new(HertzU64::MHz(500));
    /// assert_eq!(cfg.step, HertzU64::kHz(6250));
    /// assert_eq!(cfg.max_freq, HertzU64::MHz(625));
    /// ```
    pub fn new(target_avg_freq: HertzU64) -> Self {
        Self {
            step: HertzU64::kHz(6250),
            back_off_steps: 2,
            chip_rate_deviation: -0.8,
            chain_rate_deviation: -1.0,
            domain_spread: 20.0,
            min_start_rate: 80.0,
            max_flat_rounds: 2,
            target_avg_freq,
            max_freq: HertzU64::from_raw(target_avg_freq.to_raw() / 4 * 5),
        }
    }
}

/// # Error of a frequency sweep round
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SweepError {
    /// The counts given are not one per chip.
    SizeMismatch,
    /// The chain's nonce rate in the first round, in percent, is below `min_start_rate`.
    StartRateTooLow { rate: f32 },
    /// The sweep has already ended.
    Finished,
}

/// # State of a frequency sweep after a round
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SweepState {
    /// Apply the targets and run another round.
    Running,
    /// Apply the targets: they are the final frequencies.
    Done,
}

/// # Frequency sweep of the chips of a chain, up to `N` chips
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FreqSweep<const N: usize> {
    cfg: SweepConfig,
    domain_cnt: usize,
    targets: Vec<HertzU64, N>,
    last_good: Vec<HertzU64, N>,
    done: Vec<bool, N>,
    chip_limit: f32,
    chain_limit: f32,
    started: bool,
    finished: bool,
    prev_avg: f32,
    flat_rounds: u32,
}

impl<const N: usize> FreqSweep<N> {
    /// ## Start a sweep from the current frequency of each chip
    ///
    /// The chips are split into `domain_cnt` equal groups along the chain, the voltage domains.
    /// Only the first `N` frequencies are kept.
    pub fn new(cfg: SweepConfig, freqs: &[HertzU64], domain_cnt: usize) -> Self {
        let targets: Vec<HertzU64, N> = freqs.iter().copied().take(N).collect();
        let n = targets.len();
        Self {
            cfg,
            domain_cnt: domain_cnt.clamp(1, n.max(1)),
            last_good: targets.clone(),
            targets,
            done: (0..n).map(|_| false).collect(),
            chip_limit: 0.0,
            chain_limit: 0.0,
            started: false,
            finished: false,
            prev_avg: 0.0,
            flat_rounds: 0,
        }
    }

    /// ## Frequencies to apply, one per chip
    pub fn targets(&self) -> &[HertzU64] {
        &self.targets
    }

    /// ## Chips that no longer move
    pub fn chip_done(&self) -> &[bool] {
        &self.done
    }

    /// ## Whether the sweep has ended
    pub fn is_done(&self) -> bool {
        self.finished
    }

    /// ## Run a round from the nonces each chip returned at the current targets
    ///
    /// `nonces` and `expected` hold, for each chip, the nonces received and the nonces expected
    /// over the same time. Each chip's nonce rate is `min(nonces, expected) / expected`, the
    /// chain's the same over all chips.
    ///
    /// ### Errors
    ///
    /// - SizeMismatch: not one count per chip
    /// - StartRateTooLow: the chain's rate in the first round is below `min_start_rate`
    /// - Finished: the sweep has already ended
    pub fn round(&mut self, nonces: &[u32], expected: &[f32]) -> Result<SweepState, SweepError> {
        if self.finished {
            return Err(SweepError::Finished);
        }
        let n = self.targets.len();
        if nonces.len() != n || expected.len() != n {
            return Err(SweepError::SizeMismatch);
        }
        let got = |i: usize| (nonces[i] as f32).min(expected[i].max(0.0));
        let rate = |i: usize| {
            if expected[i] > 0.0 {
                100.0 * got(i) / expected[i]
            } else {
                0.0
            }
        };
        let expected_sum: f32 = expected.iter().map(|e| e.max(0.0)).sum();
        let chain_rate = if expected_sum > 0.0 {
            100.0 * (0..n).map(got).sum::<f32>() / expected_sum
        } else {
            0.0
        };

        if !self.started {
            if chain_rate < self.cfg.min_start_rate {
                self.finished = true;
                return Err(SweepError::StartRateTooLow { rate: chain_rate });
            }
            self.started = true;
            self.chip_limit = chain_rate + self.cfg.chip_rate_deviation;
            self.chain_limit = chain_rate + self.cfg.chain_rate_deviation;
            for i in 0..n {
                self.last_good[i] = self.targets[i];
                self.targets[i] = self.raised(self.targets[i]);
            }
            return Ok(SweepState::Running);
        }

        if chain_rate < self.chain_limit || self.domains_spread() {
            return Ok(self.finish());
        }
        let back_off = self.cfg.step.to_raw() * self.cfg.back_off_steps as u64;
        for i in 0..n {
            if self.done[i] {
                continue;
            }
            if rate(i) < self.chip_limit {
                self.done[i] = true;
                let freq = self.targets[i]
                    .to_raw()
                    .saturating_sub(back_off)
                    .max(self.cfg.step.to_raw());
                self.targets[i] = HertzU64::from_raw(freq);
                self.last_good[i] = self.targets[i];
            } else {
                self.last_good[i] = self.targets[i];
                if self.targets[i] >= self.cfg.max_freq {
                    self.done[i] = true;
                } else {
                    self.targets[i] = self.raised(self.targets[i]);
                }
            }
        }

        let avg = mean_mhz(&self.last_good);
        let weighted_avg = avg * chain_rate / 100.0;
        if weighted_avg <= self.prev_avg {
            self.flat_rounds += 1;
        }
        self.prev_avg = weighted_avg;
        if self.done.iter().all(|d| *d)
            || self.flat_rounds >= self.cfg.max_flat_rounds
            || avg >= self.cfg.target_avg_freq.to_raw() as f32 / 1_000_000.0
        {
            return Ok(self.finish());
        }
        Ok(SweepState::Running)
    }

    fn raised(&self, freq: HertzU64) -> HertzU64 {
        let freq = freq + self.cfg.step;
        if freq > self.cfg.max_freq {
            self.cfg.max_freq
        } else {
            freq
        }
    }

    /// Whether the average frequencies of the domains spread more than `domain_spread`.
    fn domains_spread(&self) -> bool {
        let domain_asic_cnt = self.targets.len().div_ceil(self.domain_cnt).max(1);
        let mut min = f32::MAX;
        let mut max = 0.0f32;
        for domain in self.targets.chunks(domain_asic_cnt) {
            let avg = mean_mhz(domain);
            min = min.min(avg);
            max = max.max(avg);
        }
        max > min * (100.0 + self.cfg.domain_spread) / 100.0
    }

    /// End the sweep with every chip at its last good frequency.
    fn finish(&mut self) -> SweepState {
        self.targets.clone_from(&self.last_good);
        self.finished = true;
        SweepState::Done
    }
}

/// # Settings of a domain guard
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DomainGuardConfig {
    /// Fewest nonces a domain must be expected to return before it is judged.
    pub min_domain_nonces: f32,
    /// A domain whose nonce rate is below this fraction of the chain's is lagging.
    pub rate_ratio: f32,
    /// With voltages given, a lagging domain is slowed down only if its voltage is below this
    /// fraction of the expected one.
    pub voltage_ratio: f32,
    /// Frequency taken off the chips of a lagging domain.
    pub step: HertzU64,
    /// Most slow-downs per chain.
    pub max_slow_downs: u32,
}

impl Default for DomainGuardConfig {
    /// At least 2000 nonces expected per domain, rate below 95 % of the chain's, voltage below
    /// 97 % of the expected one, 15 MHz off, twice at most.
    fn default() -> Self {
        Self {
            min_domain_nonces: 2000.0,
            rate_ratio: 0.95,
            voltage_ratio: 0.97,
            step: HertzU64::MHz(15),
            max_slow_downs: 2,
        }
    }
}

/// # Guard slowing down the voltage domains that fall behind while hashing
///
/// Call [`DomainGuard::check`] from time to time with the nonces each chip returned and the nonces
/// expected over the same time; when it names a domain, lower its chips with
/// [`DomainGuard::slow_down`] and apply the frequencies with
/// [`Chain::set_chip_hash_freqs`](crate::Chain::set_chip_hash_freqs).
///
/// ### Example
///
/// ```
/// use bm13xx_chain::tuning::{DomainGuard, DomainGuardConfig};
/// use fugit::HertzU64;
///
/// let mut guard = DomainGuard::new(DomainGuardConfig::default(), 2);
/// let expected = [1500.0; 4];
/// // the second domain (chips 2 and 3) returns 10 % fewer nonces
/// let nonces = [1500, 1500, 1350, 1350];
/// assert_eq!(guard.check(&nonces, &expected, None), Some(1));
/// let mut freqs = [HertzU64::MHz(500); 4];
/// guard.slow_down(&mut freqs, 1);
/// assert_eq!(freqs[2..], [HertzU64::MHz(485); 2]);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DomainGuard {
    cfg: DomainGuardConfig,
    domain_cnt: usize,
    slow_downs: u32,
}

impl DomainGuard {
    /// ## Guard a chain whose chips are split into `domain_cnt` equal voltage domains
    pub fn new(cfg: DomainGuardConfig, domain_cnt: usize) -> Self {
        Self {
            cfg,
            domain_cnt: domain_cnt.max(1),
            slow_downs: 0,
        }
    }

    /// ## Slow-downs done so far
    pub fn slow_downs(&self) -> u32 {
        self.slow_downs
    }

    /// ## Find a domain to slow down
    ///
    /// `nonces` and `expected` hold, for each chip, the nonces received and the nonces expected
    /// over the same time. `voltages`, when given, holds each domain's voltage and the voltage
    /// expected for a domain. Returns the lagging domain with the lowest nonce rate, if any, and
    /// counts it as a slow-down; `None` once `max_slow_downs` is reached.
    pub fn check(
        &mut self,
        nonces: &[u32],
        expected: &[f32],
        voltages: Option<(&[f32], f32)>,
    ) -> Option<usize> {
        if self.slow_downs >= self.cfg.max_slow_downs || nonces.len() != expected.len() {
            return None;
        }
        let n = nonces.len();
        let domain_asic_cnt = n.div_ceil(self.domain_cnt).max(1);
        let total_expected: f32 = expected.iter().sum();
        if total_expected <= 0.0 {
            return None;
        }
        let chain_rate = nonces.iter().map(|&c| c as f32).sum::<f32>() / total_expected;
        let mut lagging: Option<(usize, f32)> = None;
        for (domain, (counts, exp)) in nonces
            .chunks(domain_asic_cnt)
            .zip(expected.chunks(domain_asic_cnt))
            .enumerate()
        {
            let domain_expected: f32 = exp.iter().sum();
            if domain_expected < self.cfg.min_domain_nonces {
                continue;
            }
            let rate = counts.iter().map(|&c| c as f32).sum::<f32>() / domain_expected;
            if rate >= chain_rate * self.cfg.rate_ratio {
                continue;
            }
            if let Some((volts, expected_volts)) = voltages {
                match volts.get(domain) {
                    Some(v) if *v < expected_volts * self.cfg.voltage_ratio => {}
                    _ => continue,
                }
            }
            if lagging.is_none_or(|(_, r)| rate < r) {
                lagging = Some((domain, rate));
            }
        }
        let (domain, _) = lagging?;
        self.slow_downs += 1;
        Some(domain)
    }

    /// ## Lower the frequency of the chips of `domain` by `step`
    pub fn slow_down(&self, freqs: &mut [HertzU64], domain: usize) {
        let domain_asic_cnt = freqs.len().div_ceil(self.domain_cnt).max(1);
        let step = self.cfg.step.to_raw();
        for f in freqs
            .iter_mut()
            .skip(domain * domain_asic_cnt)
            .take(domain_asic_cnt)
        {
            *f = HertzU64::from_raw(f.to_raw().saturating_sub(step).max(step));
        }
    }
}

fn mean_mhz(freqs: &[HertzU64]) -> f32 {
    if freqs.is_empty() {
        return 0.0;
    }
    freqs.iter().map(|f| f.to_raw() as f32).sum::<f32>() / freqs.len() as f32 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::vec::Vec as StdVec;

    /// Nonces of chips that lose `loss` of their nonces above their own limit (MHz).
    fn nonces(sweep: &FreqSweep<8>, limits: &[u32], loss: u32) -> StdVec<u32> {
        sweep
            .targets()
            .iter()
            .zip(limits)
            .map(|(f, l)| {
                if f.to_MHz() as u32 <= *l {
                    10_000
                } else {
                    10_000 - loss
                }
            })
            .collect()
    }

    fn run(sweep: &mut FreqSweep<8>, limits: &[u32], loss: u32) -> usize {
        let expected = [10_000.0; 8];
        let mut rounds = 0;
        loop {
            rounds += 1;
            let n = nonces(sweep, limits, loss);
            if sweep.round(&n, &expected[..limits.len()]).unwrap() == SweepState::Done {
                return rounds;
            }
        }
    }

    #[test]
    fn each_chip_stops_two_steps_above_its_limit_at_most() {
        let limits = [480, 500, 520, 540];
        let mut cfg = SweepConfig::new(HertzU64::MHz(600));
        cfg.max_flat_rounds = 10;
        let mut sweep = FreqSweep::<8>::new(cfg, &[HertzU64::MHz(450); 4], 1);
        run(&mut sweep, &limits, 300);
        for (f, l) in sweep.targets().iter().zip(limits) {
            // the first failing step is above the limit, two steps back is below it
            assert!(f.to_MHz() as u32 <= l, "{f} above {l}");
            assert!(f.to_MHz() as u32 + 13 > l, "{f} far below {l}");
        }
        assert!(sweep.chip_done().iter().all(|d| *d));
        assert!(sweep.is_done());
    }

    #[test]
    fn stops_at_the_target_average() {
        let mut sweep = FreqSweep::<8>::new(
            SweepConfig::new(HertzU64::MHz(500)),
            &[HertzU64::MHz(450); 4],
            1,
        );
        run(&mut sweep, &[700; 4], 300);
        assert!(sweep.targets().iter().all(|f| *f == HertzU64::MHz(500)));
    }

    #[test]
    fn chain_rate_drop_puts_every_chip_back() {
        let mut sweep = FreqSweep::<8>::new(
            SweepConfig::new(HertzU64::MHz(600)),
            &[HertzU64::MHz(450); 4],
            1,
        );
        // one chip losing a quarter of its nonces drops the chain rate by more than one point
        run(&mut sweep, &[500, 700, 700, 700], 2_500);
        let f = sweep.targets()[0];
        assert!(sweep.targets().iter().all(|t| *t == f));
        assert!(f.to_MHz() <= 500);
    }

    #[test]
    fn domain_spread_ends_the_sweep() {
        let mut cfg = SweepConfig::new(HertzU64::MHz(700));
        cfg.domain_spread = 5.0;
        cfg.max_flat_rounds = 10;
        let mut sweep = FreqSweep::<8>::new(cfg, &[HertzU64::MHz(450); 8], 2);
        // the chips of the first domain stop early, the second domain goes on
        run(&mut sweep, &[460, 460, 460, 460, 700, 700, 700, 700], 150);
        assert!(sweep.chip_done()[..4].iter().all(|d| *d));
        let domain = |d: usize| mean_mhz(&sweep.targets()[4 * d..4 * d + 4]);
        assert!(domain(1) > domain(0));
        assert!(domain(1) <= domain(0) * 1.05 + 6.25);
    }

    #[test]
    fn rounds_without_gain_end_the_sweep() {
        // every round where a chip fails lowers the rate-weighted average a little
        let mut sweep = FreqSweep::<8>::new(
            SweepConfig::new(HertzU64::MHz(600)),
            &[HertzU64::MHz(450); 4],
            1,
        );
        run(&mut sweep, &[480, 500, 520, 540], 300);
        assert_eq!(sweep.chip_done(), &[true, true, false, false]);
    }

    #[test]
    fn domain_guard_needs_enough_nonces_and_a_low_voltage() {
        let mut guard = DomainGuard::new(DomainGuardConfig::default(), 2);
        let nonces = [1500, 1500, 1350, 1350];
        // not enough nonces yet
        assert_eq!(guard.check(&nonces, &[900.0; 4], None), None);
        // the lagging domain's voltage is normal
        let voltages = [3.3, 3.3];
        assert_eq!(
            guard.check(&nonces, &[1500.0; 4], Some((&voltages, 3.3))),
            None
        );
        // and low
        let voltages = [3.3, 3.1];
        assert_eq!(
            guard.check(&nonces, &[1500.0; 4], Some((&voltages, 3.3))),
            Some(1)
        );
        assert_eq!(guard.check(&nonces, &[1500.0; 4], None), Some(1));
        // two slow-downs at most
        assert_eq!(guard.check(&nonces, &[1500.0; 4], None), None);
        assert_eq!(guard.slow_downs(), 2);
    }

    #[test]
    fn domain_guard_picks_the_slowest_domain() {
        let mut guard = DomainGuard::new(DomainGuardConfig::default(), 3);
        let nonces = [3000, 3000, 2600, 2600, 2400, 2400];
        assert_eq!(guard.check(&nonces, &[3000.0; 6], None), Some(2));
    }

    #[test]
    fn low_start_rate_is_refused() {
        let mut sweep = FreqSweep::<8>::new(
            SweepConfig::new(HertzU64::MHz(600)),
            &[HertzU64::MHz(450); 2],
            1,
        );
        assert_eq!(
            sweep.round(&[7_000, 7_000], &[10_000.0, 10_000.0]),
            Err(SweepError::StartRateTooLow { rate: 70.0 })
        );
        assert_eq!(
            sweep.round(&[10_000, 10_000], &[10_000.0, 10_000.0]),
            Err(SweepError::Finished)
        );
    }

    #[test]
    fn counts_must_match_the_chips() {
        let mut sweep = FreqSweep::<8>::new(
            SweepConfig::new(HertzU64::MHz(600)),
            &[HertzU64::MHz(450); 2],
            1,
        );
        assert_eq!(
            sweep.round(&[10_000], &[10_000.0]),
            Err(SweepError::SizeMismatch)
        );
    }
}
