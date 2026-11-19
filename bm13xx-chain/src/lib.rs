//! BM13xx Chain representation based on
//! [`embedded-hal`](https://github.com/rust-embedded/embedded-hal).
//! Thanks to this abstraction layer, it can be used on full-fledged operating
//! systems as well as embedded devices.
//!
//! By default, this library exposes an async API.
//!
//! # Examples
//! The crate ships with a CLI example that utilize the library from your host computer:
//! * [`bm13xx-cli.rs`](examples/cli.rs) uses the asynchronous interface (embedded-io-async).
//!
//! The example below demonstrates how to use it with an ESP32S3,
//! showcasing the strength of the embedded-hal abstractions.
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use embassy_executor::Spawner;
//! use embassy_time::{Delay, Duration, Timer};
//! use esp_backtrace as _;
//! use esp_hal::{
//!     gpio::Output,
//!     timer::timg::TimerGroup,
//!     uart::{Config, Uart},
//! };
//! use esp_println::println;
//! use bm1366::BM1366;
//! use bm13xx_chain::{Baud, Chain};
//!
//! #[esp_hal_embassy::main]
//! async fn main(_s: Spawner) {
//!     println!("Init!");
//!     let peripherals = esp_hal::init(esp_hal::Config::default());
//!
//!     let timg0 = TimerGroup::new(peripherals.TIMG0);
//!     esp_hal_embassy::init(timg0.timer0);
//!
//!     let (tx_pin, rx_pin) = (peripherals.GPIO43, peripherals.GPIO44);
//!
//!     let config = Config::default()
//!         .baudrate(115_200)
//!         .with_rx_fifo_full_threshold(bm13xx_protocol::READ_BUF_SIZE as u16);
//!     let mut uart0 = Uart::new(peripherals.UART0, config)
//!         .unwrap()
//!         .with_tx(tx_pin)
//!         .with_rx(rx_pin)
//!         .into_async();
//!
//!     let busy = Output::new(peripherals.GPIO0, Level::High);
//!     let reset = Output::new(peripherals.GPIO2, Level::Low);
//!
//!     let bm1366 = BM1366::default();
//!     let mut chain = Chain::enumerate(bm1366,&mut uart0, busy, reset, Delay).await.unwrap();
//!     println!("Enumerated {} asics", chain.asic_cnt);
//!     println!("Interval: {}", chain.asic_addr_interval);
//!     chain.init(256).await.unwrap();
//!     chain.set_baudrate(1_000_000).await.unwrap();
//!
//!     loop {
//!         Timer::after(Duration::from_millis(30)).await;
//!     }
//! }
//! ```

#![no_std]
pub(crate) mod fmt;

mod error;
pub mod tuning;

use core::time::Duration;

pub use self::error::{Error, Result};

use bm13xx_asic::{
    register::{ChipIdentification, OrderedClockMonitor, PLL0Parameter},
    Asic, CmdDelay, SensorStep,
};
use bm13xx_protocol::{
    command::{Command, Destination},
    response::{Response, ResponseType, FRAME_SIZE, FRAME_SIZE_VER},
};

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, ReadReady, Write};
use fugit::HertzU64;
use heapless::Vec;

pub trait Baud {
    fn set_baudrate(&mut self, baudrate: u32);
}

const RX_BUF_SIZE: usize = 256;

/// Enumeration stops once no ChipIdentification reply has come for this long.
const ENUM_QUIET_MS: u32 = 100;
/// Time to wait for the answer to a register read.
const READ_TIMEOUT_MS: u32 = 100;
/// Values read by one sensor reading sequence.
const SENSOR_SAMPLES: usize = 64;

/// Largest number of chips in a chain: a chip address is one byte.
pub const MAX_ASIC_CNT: usize = 256;

const NONCE_BITS: u32 = u32::BITS;
const CHIP_ADDR_BITS: u32 = u8::BITS;

/// A job sent with version rolling, kept to check the nonces found for it.
#[cfg(feature = "nonce-check")]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct SentJob {
    version: u32,
    prev_block_header_hash: [u8; 32],
    merkle_root: [u8; 32],
    n_bits: u32,
    n_time: u32,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Chain<A, U, OB, OR, D> {
    pub asic_cnt: usize,
    asic: A,
    pub asic_addr_interval: usize,
    domain_cnt: usize,
    uart: U,
    rx_buf: [u8; RX_BUF_SIZE],
    rx_free_pos: usize,
    rx_dropped: usize,
    busy: OB,
    reset: OR,
    delay: D,
    job_id: u8,
    version_rolling_mask: Option<u32>,
    chip_nonce_space: usize,
    /// Hash frequency of each chip.
    chip_freqs: Vec<HertzU64, MAX_ASIC_CNT>,
    /// Nonces received from each chip since the last clear.
    chip_nonces: Vec<u32, MAX_ASIC_CNT>,
    /// Difficulty of the ticket mask the chips use.
    ticket_difficulty: u32,
    /// Jobs sent, by job ID.
    #[cfg(feature = "nonce-check")]
    jobs: [Option<SentJob>; 32],
    /// Nonces from each chip that do not meet the ticket mask since the last clear.
    #[cfg(feature = "nonce-check")]
    chip_hw_errors: Vec<u32, MAX_ASIC_CNT>,
}

impl<A: Asic, U: Read + ReadReady + Write + Baud, OB: OutputPin, OR: OutputPin, D: DelayNs>
    Chain<A, U, OB, OR, D>
{
    async fn send(&mut self, step: CmdDelay) -> Result<(), U::Error, OB::Error, OR::Error> {
        self.uart.write_all(&step.cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(step.delay_ms).await;
        Ok(())
    }

    /// ## Get the rolling duration
    ///
    /// Total time to roll the Nonce space and Version space (if HW version rolling is enabled) for the full chain at current Hash frequency.
    /// A new job should be sent every `rolling_duration`.
    pub fn rolling_duration(&self) -> Duration {
        let space = self.chip_nonce_space as f32
            * if let Some(mask) = self.version_rolling_mask {
                (mask.count_ones() - 1) as f32
            } else {
                1.0
            };
        Duration::from_secs_f32(space / (self.asic.hash_freq().to_raw() as f32))
    }

    /// ## Get the theoretical Hashrate in GH/s
    pub fn theoretical_hashrate_ghs(&self) -> f32 {
        (self.asic.hash_freq().to_raw() as f32 * self.asic.small_core_count() as f32
            / 1_000_000_000.0)
            * self.asic_cnt as f32
    }

    /// ## Poll for a response
    ///
    /// Read data from the UART, and store them in the internal rx buffer.
    /// If only a partial frame or nothing has been received, this function will return None.
    /// If at least a complete frame has been received, this function will return the parsed response.
    /// If more than the first frame has been received, this function will keep the extra data in the internal rx buffer.
    /// In case of receiving a Frame with bad CRC or bad Preamble, this function will drop bytes up to the next
    /// possible Preamble, count them (see [`Chain::rx_dropped_bytes`]) and return None.
    pub async fn poll_response(
        &mut self,
    ) -> Result<Option<ResponseType>, U::Error, OB::Error, OR::Error> {
        let mut resp = None;
        let expected_frame_size = if self.version_rolling_mask.is_some() {
            FRAME_SIZE_VER
        } else {
            FRAME_SIZE
        };
        if self.rx_free_pos >= expected_frame_size {
            let frame = &self.rx_buf[..expected_frame_size];
            let (used, dropped) = match if self.version_rolling_mask.is_some() {
                Response::parse_version(
                    frame.try_into().unwrap(),
                    self.asic.core_small_core_count(),
                )
            } else {
                Response::parse(frame.try_into().unwrap(), self.asic.core_small_core_count())
            } {
                Ok(r) => {
                    resp = Some(r);
                    (expected_frame_size, 0)
                }
                Err(bm13xx_protocol::Error::InvalidCrc { expected, actual }) => {
                    let offset = self.next_preamble_offset();
                    error!(
                        "Resync Frame {:x} because bad CRC {:02x}!={:02x}, dropping first {} bytes",
                        frame, expected, actual, offset
                    );
                    (offset, offset)
                }
                Err(bm13xx_protocol::Error::InvalidPreamble) => {
                    let offset = self.next_preamble_offset();
                    error!(
                        "Resync Frame {:x} because bad preamble, dropping first {} bytes",
                        frame, offset
                    );
                    (offset, offset)
                }
                Err(bm13xx_protocol::Error::UnsupportedCoreSmallCoreCnt) => {
                    error!(
                        "Ignoring Frame {:x} because bad CoreSmallCoreCnt {}",
                        frame,
                        self.asic.core_small_core_count()
                    );
                    (expected_frame_size, 0)
                }
            };
            self.rx_dropped = self.rx_dropped.wrapping_add(dropped);
            if self.rx_free_pos > used {
                debug!("copy reminder {} bytes @0", self.rx_free_pos - used);
                self.rx_buf.copy_within(used..self.rx_free_pos, 0);
            }
            self.rx_free_pos -= used;
        }
        if self.uart.read_ready().map_err(Error::Io)? {
            let n = self
                .uart
                .read(self.rx_buf[self.rx_free_pos..].as_mut())
                .await
                .map_err(Error::Io)?;
            debug!("read {} bytes @{}", n, self.rx_free_pos);
            trace!("{:?}", &self.rx_buf[self.rx_free_pos..self.rx_free_pos + n]);
            self.rx_free_pos += n;
        }
        if let Some(r) = &resp {
            self.count_nonce(r);
        }
        Ok(resp)
    }

    /// Count a nonce response for the chip that found it. With the nonce check, a nonce that does
    /// not meet the ticket mask counts as a hardware error of that chip instead.
    fn count_nonce(&mut self, resp: &ResponseType) {
        let (nonce, version_rolling) = match resp {
            ResponseType::Job(j) => (j.nonce, false),
            ResponseType::JobVer(j) => (j.nonce, true),
            ResponseType::Reg(_) => return,
        };
        let Some(i) = self.asic.nonce_chip_index(
            nonce,
            version_rolling,
            self.asic_cnt,
            self.asic_addr_interval,
        ) else {
            return;
        };
        #[cfg(feature = "nonce-check")]
        if let Some(zero_bits) = self.share_zero_bits(resp) {
            if zero_bits < 32 + self.ticket_difficulty.trailing_zeros() {
                if let Some(errors) = self.chip_hw_errors.get_mut(i) {
                    *errors = errors.wrapping_add(1);
                }
                return;
            }
        }
        if let Some(count) = self.chip_nonces.get_mut(i) {
            *count = count.wrapping_add(1);
        }
    }

    /// ## Count the leading zero bits of a share's hash
    ///
    /// Rebuilds the block header of a nonce response with version rolling from the job sent with
    /// its job ID, and double-hashes it: a nonce meets the ticket mask when its hash has at least
    /// `32 + log2(ticket mask difficulty)` leading zero bits. `None` for other responses or an
    /// unknown job.
    #[cfg(feature = "nonce-check")]
    pub fn share_zero_bits(&self, resp: &ResponseType) -> Option<u32> {
        use sha2::{Digest, Sha256};

        let ResponseType::JobVer(j) = resp else {
            return None;
        };
        let job = self.jobs.get(j.job_id)?.as_ref()?;
        let mut header = [0u8; 80];
        header[0..4].copy_from_slice(&(job.version | j.version_bit).to_le_bytes());
        header[4..36].copy_from_slice(&job.prev_block_header_hash);
        header[4..36].reverse();
        header[36..68].copy_from_slice(&job.merkle_root);
        header[36..68].reverse();
        header[68..72].copy_from_slice(&job.n_time.to_le_bytes());
        header[72..76].copy_from_slice(&job.n_bits.to_le_bytes());
        header[76..80].copy_from_slice(&j.nonce.to_le_bytes());
        let hash = Sha256::digest(Sha256::digest(header));
        let mut zero_bits = 0;
        for b in hash.iter().rev() {
            zero_bits += b.leading_zeros();
            if *b != 0 {
                break;
            }
        }
        Some(zero_bits)
    }

    /// Number of bytes to drop from the rx buffer to reach the next possible Preamble after the first byte.
    fn next_preamble_offset(&self) -> usize {
        let data = &self.rx_buf[..self.rx_free_pos];
        match data[1..].windows(2).position(|w| w == [0xAA, 0x55]) {
            Some(pos) => pos + 1,
            // keep a trailing 0xAA, it can be the start of the next Preamble
            None if data.last() == Some(&0xAA) => data.len() - 1,
            None => data.len(),
        }
    }

    /// ## Get the number of bytes dropped while resyncing on the response stream
    ///
    /// Bytes are dropped when a Frame has a bad Preamble or a bad CRC.
    /// A growing count is a sign of a noisy serial line or a wrong baudrate.
    pub fn rx_dropped_bytes(&self) -> usize {
        self.rx_dropped
    }

    /// Wait for the answer of chip `chip_addr` to a read of `reg_addr`. Other responses received
    /// meanwhile, nonces included, are dropped.
    async fn wait_register(
        &mut self,
        chip_addr: u8,
        reg_addr: u8,
    ) -> Result<Option<u32>, U::Error, OB::Error, OR::Error> {
        let mut waited_ms = 0;
        while waited_ms < READ_TIMEOUT_MS {
            match self.poll_response().await? {
                Some(ResponseType::Reg(r))
                    if r.chip_addr == chip_addr && r.reg_addr == reg_addr =>
                {
                    return Ok(Some(r.reg_value));
                }
                Some(_) => {}
                None => {
                    self.delay.delay_ms(1).await;
                    waited_ms += 1;
                }
            }
        }
        Ok(None)
    }

    /// ## Read a register of one asic
    ///
    /// Returns `None` if the asic did not answer. Other responses received meanwhile, nonces
    /// included, are dropped.
    pub async fn read_register(
        &mut self,
        asic_index: usize,
        reg_addr: u8,
    ) -> Result<Option<u32>, U::Error, OB::Error, OR::Error> {
        let chip_addr = (asic_index * self.asic_addr_interval) as u8;
        let cmd = Command::read_reg(reg_addr, Destination::Chip(chip_addr));
        self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        self.wait_register(chip_addr, reg_addr).await
    }

    /// ## Read the hash clock of one asic
    ///
    /// The chip counts its hash clock cycles during 256 cycles of its input clock: the frequency
    /// it really runs at. `None` if the asic did not answer or has no count ready.
    pub async fn read_chip_hash_clock(
        &mut self,
        asic_index: usize,
    ) -> Result<Option<HertzU64>, U::Error, OB::Error, OR::Error> {
        let input_clock_freq = self.asic.input_clock_freq();
        Ok(self
            .read_register(asic_index, OrderedClockMonitor::ADDR)
            .await?
            .map(OrderedClockMonitor)
            .filter(|m| m.data_ready())
            .map(|m| m.hash_freq(input_clock_freq)))
    }

    /// ## Read whether the hash PLL of one asic is locked
    ///
    /// The chip sets `PLL0Parameter` bit 31 once its PLL has locked. `None` if the asic did not
    /// answer.
    pub async fn read_chip_pll_locked(
        &mut self,
        asic_index: usize,
    ) -> Result<Option<bool>, U::Error, OB::Error, OR::Error> {
        Ok(self
            .read_register(asic_index, PLL0Parameter::ADDR)
            .await?
            .map(|v| v & 0x8000_0000 != 0))
    }

    /// Run a sensor reading sequence for one asic, from `next` (`next(asic, dest)` gives the steps),
    /// and collect the values read. `None` if the asic did not answer every read.
    async fn read_sensor(
        &mut self,
        asic_index: usize,
        mut next: impl FnMut(&mut A, Destination) -> Option<SensorStep>,
    ) -> Result<Option<Vec<u32, SENSOR_SAMPLES>>, U::Error, OB::Error, OR::Error> {
        let chip_addr = (asic_index * self.asic_addr_interval) as u8;
        let dest = Destination::Chip(chip_addr);
        let mut samples = Vec::<u32, SENSOR_SAMPLES>::new();
        let mut answered = true;
        while let Some(step) = next(&mut self.asic, dest) {
            match step {
                SensorStep::Write(cmd) => self.send(cmd).await?,
                SensorStep::Read { reg_addr, delay_ms } => {
                    let cmd = Command::read_reg(reg_addr, dest);
                    self.uart.write_all(&cmd).await.map_err(Error::Io)?;
                    self.delay.delay_ms(delay_ms).await;
                    match self.wait_register(chip_addr, reg_addr).await? {
                        Some(value) => {
                            let _ = samples.push(value);
                        }
                        None => answered = false,
                    }
                }
            }
        }
        Ok(answered.then_some(samples))
    }

    /// ## Read the die temperature of one asic, in °C
    ///
    /// Returns `None` if the asic has no die temperature reading or did not answer. Other
    /// responses received meanwhile, nonces included, are dropped.
    pub async fn read_die_temp(
        &mut self,
        asic_index: usize,
    ) -> Result<Option<f32>, U::Error, OB::Error, OR::Error> {
        Ok(self
            .read_sensor(asic_index, |asic, dest| asic.die_temp_next(dest))
            .await?
            .and_then(|samples| self.asic.die_temp_celsius(&samples)))
    }

    /// ## Read a supply voltage of one asic, in mV
    ///
    /// `channel` is one of the chip's sense channels (0 to 3). Returns `None` if the asic has no
    /// such reading or did not answer. Other responses received meanwhile, nonces included, are
    /// dropped.
    pub async fn read_chip_supply_voltage(
        &mut self,
        asic_index: usize,
        channel: u8,
    ) -> Result<Option<f32>, U::Error, OB::Error, OR::Error> {
        Ok(self
            .read_sensor(asic_index, |asic, dest| {
                asic.supply_voltage_next(dest, channel)
            })
            .await?
            .and_then(|samples| self.asic.supply_voltage_mv(channel, &samples)))
    }

    /// ## Enumerate all asics on the chain
    ///
    /// Sets the `asic_addr_interval` to the largest power of two that fits all the asics enumerated
    /// in the 8-bit chip address space, as the stock firmware does (2 for 65 to 128 asics).
    ///
    /// ### Errors
    ///
    /// - I/O error
    /// - Gpio error
    /// - Unexpected response
    /// - Bad register response
    /// - Unexpected asic
    /// - Protocol error
    /// - Empty chain
    pub async fn enumerate(
        asic: A,
        uart: U,
        busy: OB,
        reset: OR,
        delay: D,
    ) -> Result<Self, U::Error, OB::Error, OR::Error> {
        let mut chain = Chain::<A, U, OB, OR, D> {
            asic_cnt: 0,
            asic,
            asic_addr_interval: 0,
            domain_cnt: 1,
            uart,
            rx_buf: [0; RX_BUF_SIZE],
            rx_free_pos: 0,
            rx_dropped: 0,
            busy,
            reset,
            delay,
            job_id: 0,
            version_rolling_mask: None,
            chip_nonce_space: 0,
            chip_freqs: Vec::new(),
            chip_nonces: Vec::new(),
            ticket_difficulty: 1,
            #[cfg(feature = "nonce-check")]
            jobs: [None; 32],
            #[cfg(feature = "nonce-check")]
            chip_hw_errors: Vec::new(),
        };

        chain.reset.set_high().map_err(Error::Reset)?;
        chain.delay.delay_ms(10).await;
        chain.busy.set_low().map_err(Error::Busy)?;
        let cmd = Command::read_reg(ChipIdentification::ADDR, Destination::All);
        chain.uart.write_all(&cmd).await.map_err(Error::Io)?;

        let mut asic_cnt = 0;
        // `poll_response` can return None while replies are still coming, so only stop once
        // no reply has come for `ENUM_QUIET_MS`.
        let mut quiet_ms = 0;
        while quiet_ms < ENUM_QUIET_MS {
            chain.delay.delay_ms(10).await;
            quiet_ms += 10;
            if let Some(resp) = chain.poll_response().await? {
                quiet_ms = 0;
                if let ResponseType::Reg(reg_resp) = resp {
                    if reg_resp.chip_addr != 0 || reg_resp.reg_addr != ChipIdentification::ADDR {
                        warn!("reg_resp: {:#?}, {}", reg_resp, ChipIdentification::ADDR);
                        return Err(Error::BadRegisterResponse { reg_resp });
                    }
                    let chip_ident = ChipIdentification(reg_resp.reg_value);
                    if chain.asic.chip_id_matches(chip_ident.chip_id()) {
                        asic_cnt += 1;
                    } else {
                        // Heterogeneous chain is forbidden
                        return Err(Error::UnexpectedAsic { chip_ident });
                    }
                } else {
                    return Err(Error::UnexpectedResponse { resp });
                };
            }
        }
        if asic_cnt == 0 {
            return Err(Error::EmptyChain);
        }
        debug!("Enumerated {} asics", asic_cnt);
        chain.asic_addr_interval =
            (1 << (256usize / asic_cnt).max(1).ilog2()).min(chain.asic.max_addr_interval());
        chain.asic_cnt = asic_cnt;
        let freq = chain.asic.hash_freq();
        chain.chip_freqs = (0..asic_cnt).map(|_| freq).collect();
        chain.chip_nonces = (0..asic_cnt).map(|_| 0).collect();
        #[cfg(feature = "nonce-check")]
        {
            chain.chip_hw_errors = (0..asic_cnt).map(|_| 0).collect();
        }
        chain.chip_nonce_space = chain.asic_addr_interval
            << (NONCE_BITS
                - (chain.asic.core_count().next_power_of_two().ilog2())
                - (chain
                    .asic
                    .core_small_core_count()
                    .next_power_of_two()
                    .ilog2())
                - CHIP_ADDR_BITS);
        // TODO: try to determine domain_cnt according to known topologies
        chain.delay.delay_ms(50).await;
        let core_reset = chain.asic.core_reset_at_enumeration();
        if core_reset {
            chain.delay.delay_ms(100).await;
            while let Some(step) = chain.asic.reset_core_next(Destination::All) {
                chain.send(step).await?;
            }
        }
        let cmd = Command::chain_inactive();
        chain.uart.write_all(&cmd).await.map_err(Error::Io)?;
        if !core_reset {
            chain.delay.delay_ms(2).await;
            chain.uart.write_all(&cmd).await.map_err(Error::Io)?;
            chain.delay.delay_ms(2).await;
            chain.uart.write_all(&cmd).await.map_err(Error::Io)?;
        }
        chain.delay.delay_ms(30).await;
        for i in 0..asic_cnt {
            let cmd = Command::set_chip_addr((i * chain.asic_addr_interval) as u8);
            chain.uart.write_all(&cmd).await.map_err(Error::Io)?;
            chain.delay.delay_ms(10).await;
        }
        chain.delay.delay_ms(100).await;
        Ok(chain)
    }

    /// ## Set the number of domains in the chain
    ///
    /// In case we enumarted an unknown topology (custom HB?), this function is mandatory to set the number of domains.
    pub fn set_domain_cnt(&mut self, domain_cnt: usize) {
        self.domain_cnt = domain_cnt;
    }

    /// ## Reset all asics on the chain
    ///
    /// Act on the physical NRST signal propagating through the chain.
    /// A new chain enumeration is required to release the NRST signal.
    /// The serial interface is set back to 115200 baud, the chips' default after a reset.
    pub async fn reset(&mut self) -> Result<(), U::Error, OB::Error, OR::Error> {
        self.reset.set_low().map_err(Error::Reset)?;
        self.asic.reset();
        let freq = self.asic.hash_freq();
        self.chip_freqs.iter_mut().for_each(|f| *f = freq);
        self.clear_chip_nonce_counts();
        self.ticket_difficulty = 1;
        self.uart.set_baudrate(115_200);
        Ok(())
    }

    /// ## Initialize all asics on the chain
    ///
    /// Will launch the sequence of initialization steps according to the ASIC detected during enumeration.
    pub async fn init(&mut self, difficulty: u32) -> Result<(), U::Error, OB::Error, OR::Error> {
        while let Some(step) = self.asic.init_next(difficulty) {
            self.send(step).await?;
        }
        // the ticket mask keeps the largest power of two not above the difficulty
        self.ticket_difficulty = 1 << (31 - difficulty.max(1).leading_zeros());
        self.delay.delay_ms(100).await;
        Ok(())
    }

    /// ## Change the baudrate used by the chain to communicate
    pub async fn change_baudrate(
        &mut self,
        baudrate: u32,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        while let Some(step) = self.asic.set_baudrate_next(
            baudrate,
            self.domain_cnt,
            self.asic_cnt / self.domain_cnt,
            self.asic_addr_interval,
        ) {
            self.send(step).await?;
        }
        self.delay.delay_ms(50).await;
        self.uart.set_baudrate(baudrate);
        self.delay.delay_ms(50).await;
        Ok(())
    }

    /// ## Reset all cores of all chip in the chain
    pub async fn reset_all_cores(&mut self) -> Result<(), U::Error, OB::Error, OR::Error> {
        for asic_i in 0..self.asic_cnt {
            while let Some(step) = self
                .asic
                .reset_core_next(Destination::Chip((asic_i * self.asic_addr_interval) as u8))
            {
                self.send(step).await?;
            }
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    /// ## Set the SHA Hashing Frequency
    ///
    /// Will launch the sequence of frequencies ramp-up.
    pub async fn set_hash_freq(
        &mut self,
        freq: HertzU64,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        while let Some(step) = self.asic.set_hash_freq_next(freq) {
            self.send(step).await?;
        }
        let freq = self.asic.hash_freq();
        self.chip_freqs.iter_mut().for_each(|f| *f = freq);
        self.delay.delay_ms(100).await;
        Ok(())
    }

    /// ## Get the difficulty of a share from its nonce response
    ///
    /// A nonce response with version rolling tells how many zero bits the share's hash has beyond
    /// those of the ticket mask: the share's difficulty is at least the ticket mask difficulty
    /// times `2^extra_zero_bits`, and less than twice that. `None` for other responses.
    pub fn share_difficulty(&self, resp: &ResponseType) -> Option<u64> {
        match resp {
            ResponseType::JobVer(j) => Some(
                ((self.ticket_difficulty as u128) << j.extra_zero_bits.min(64))
                    .min(u64::MAX as u128) as u64,
            ),
            _ => None,
        }
    }

    /// ## Get the hash frequency of each chip
    ///
    /// In chain order. [`Chain::set_hash_freq`] sets them all, [`Chain::set_chip_hash_freqs`] one by
    /// one.
    pub fn chip_hash_freqs(&self) -> &[HertzU64] {
        &self.chip_freqs
    }

    /// ## Set the hash frequency of each chip
    ///
    /// Moves each chip towards its target in `targets` (in chain order) by steps of `step`, one
    /// chip per voltage domain at a time: in each pass, the first chip of each domain that is not
    /// at its target moves one step, then the chain waits `pass_delay_ms`. Passes go on until every
    /// chip is at its target. The domains are the chips split into equal groups along the chain
    /// (see [`Chain::set_domain_cnt`]), so the current drawn by each domain changes little at a time.
    ///
    /// ### Errors
    ///
    /// - I/O error
    /// - Unsupported: the chip cannot be set alone
    pub async fn set_chip_hash_freqs(
        &mut self,
        targets: &[HertzU64],
        step: HertzU64,
        pass_delay_ms: u32,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        let cnt = self.asic_cnt.min(targets.len()).min(self.chip_freqs.len());
        if cnt == 0 {
            return Ok(());
        }
        let domain_cnt = self.domain_cnt.clamp(1, cnt);
        let domain_asic_cnt = cnt.div_ceil(domain_cnt);
        loop {
            let mut moved = false;
            for domain in 0..domain_cnt {
                let start = domain * domain_asic_cnt;
                let end = (start + domain_asic_cnt).min(cnt);
                let Some(i) = (start..end).find(|&i| self.chip_freqs[i] != targets[i]) else {
                    continue;
                };
                let (current, target) = (self.chip_freqs[i], targets[i]);
                let freq = if target > current + step {
                    current + step
                } else if current > target + step {
                    current - step
                } else {
                    target
                };
                let dest = Destination::Chip((i * self.asic_addr_interval) as u8);
                let mut sent = false;
                while let Some(cmd) = self.asic.set_chip_hash_freq_next(dest, freq) {
                    self.send(cmd).await?;
                    sent = true;
                }
                if !sent {
                    return Err(Error::Unsupported);
                }
                self.chip_freqs[i] = freq;
                moved = true;
            }
            if !moved {
                return Ok(());
            }
            self.delay.delay_ms(pass_delay_ms).await;
        }
    }

    /// ## Get the nonces received from each chip since the last clear
    ///
    /// In chain order, counted by [`Chain::poll_response`] when the nonce tells which chip found it.
    pub fn chip_nonce_counts(&self) -> &[u32] {
        &self.chip_nonces
    }

    /// ## Clear the nonce count of each chip
    pub fn clear_chip_nonce_counts(&mut self) {
        self.chip_nonces.iter_mut().for_each(|c| *c = 0);
        #[cfg(feature = "nonce-check")]
        self.chip_hw_errors.iter_mut().for_each(|c| *c = 0);
    }

    /// ## Get the nonces from each chip that did not meet the ticket mask since the last clear
    ///
    /// In chain order: the hardware errors found by checking each nonce of a job sent with version
    /// rolling (see [`Chain::share_zero_bits`]).
    #[cfg(feature = "nonce-check")]
    pub fn chip_hw_error_counts(&self) -> &[u32] {
        &self.chip_hw_errors
    }

    /// ## Get the number of nonces expected from one chip in `duration`
    ///
    /// `f × small cores × t / (2^32 × d)`, with `f` the chip's hash frequency and `d` the ticket
    /// mask difficulty set by [`Chain::init`].
    pub fn expected_chip_nonces(&self, asic_index: usize, duration: Duration) -> f32 {
        let freq = self
            .chip_freqs
            .get(asic_index)
            .map_or(0.0, |f| f.to_raw() as f32);
        freq * self.asic.small_core_count() as f32 * duration.as_secs_f32()
            / (4_294_967_296.0 * self.ticket_difficulty as f32)
    }

    /// ## Split some Nonce space between chips
    pub async fn split_nonce_between_chips(
        &mut self,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        while let Some(step) = self
            .asic
            .split_nonce_between_chips_next(self.asic_cnt, self.asic_addr_interval)
        {
            self.send(step).await?;
        }
        self.chip_nonce_space = self.asic.cno_interval()
            << (NONCE_BITS
                - (self.asic.core_count().next_power_of_two().ilog2())
                - if self.version_rolling_mask.is_none() {
                    0
                } else {
                    self.asic
                        .core_small_core_count()
                        .next_power_of_two()
                        .ilog2()
                }
                - self.asic.cno_bits());
        Ok(())
    }

    /// ## Enable Version Rolling in chips
    ///
    /// Enable Hardware Version Rolling with the given version mask.
    pub async fn enable_version_rolling(
        &mut self,
        mask: u32,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        if self.version_rolling_mask.is_none() {
            while let Some(step) =
                self.asic
                    .set_version_rolling_next(mask, self.asic_cnt, self.asic_addr_interval)
            {
                self.send(step).await?;
            }
            self.delay.delay_ms(100).await;
            self.version_rolling_mask = Some(mask);
            // when hw version rolling is enabled, the small cores split version_space and not nonce_space anymore
            self.chip_nonce_space <<= self
                .asic
                .core_small_core_count()
                .next_power_of_two()
                .ilog2();
        }
        Ok(())
    }

    /// ## Send a Job to the chain
    ///
    /// Return the Job ID affected for this job.
    pub async fn send_job(
        &mut self,
        version: u32,
        prev_block_header_hash: [u8; 32],
        merkle_root: [u8; 32],
        n_bits: u32,
        n_time: u32,
    ) -> Result<u8, U::Error, OB::Error, OR::Error> {
        // The job id is sent shifted left by 3 bits (by 2 on chips with 4 small cores per core).
        // With up to 8 small cores per core, the chip echoes the whole sent byte (bit 7 included,
        // seen on a stock S19 XP), so 32 job ids are usable.
        // With 16 small cores per core, only 4 job id bits fit next to the small core id in the
        // response: keep the sent byte below 0x80.
        let job_id_cnt = if self.asic.core_small_core_count() > 8 {
            16
        } else {
            32
        };
        self.job_id = (self.job_id + 1) % job_id_cnt;
        #[cfg(feature = "nonce-check")]
        {
            self.jobs[self.job_id as usize] = self.version_rolling_mask.map(|_| SentJob {
                version,
                prev_block_header_hash,
                merkle_root,
                n_bits,
                n_time,
            });
        }
        if self.version_rolling_mask.is_some() {
            let cmd = Command::job_header(
                self.job_id,
                n_bits,
                n_time,
                merkle_root,
                prev_block_header_hash,
                version,
            );
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        } else {
            let merkle_root_end =
                u32::from_be_bytes(merkle_root[merkle_root.len() - 4..].try_into().unwrap());
            let mut midstates: Vec<[u8; 32], 4> = Vec::new();
            for _i in 0..self.asic.core_small_core_count() {
                midstates.push([0u8; 32]).unwrap(); // TODO finish midstate computation
            }
            let cmd = Command::job_midstate(
                self.job_id,
                n_bits,
                n_time,
                merkle_root_end,
                midstates,
                self.asic.core_small_core_count(),
            );
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        };
        Ok(self.job_id)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use bm1366::BM1366;
    use bm13xx_protocol::response::{JobVersionResponse, RegisterResponse};
    use core::convert::Infallible;
    use std::{collections::VecDeque, vec::Vec};

    /// ChipIdentification register response of a BM1397.
    const CHIP_ID: [u8; FRAME_SIZE] = [0xAA, 0x55, 0x13, 0x97, 0x18, 0x00, 0x00, 0x00, 0x06];

    /// A serial interface that returns the queued bytes and keeps what is written.
    struct MockUart(VecDeque<u8>, Vec<u8>);

    impl embedded_io_async::ErrorType for MockUart {
        type Error = Infallible;
    }

    impl Read for MockUart {
        async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Infallible> {
            let n = buf.len().min(self.0.len());
            for (b, d) in buf.iter_mut().zip(self.0.drain(..n)) {
                *b = d;
            }
            Ok(n)
        }
    }

    impl ReadReady for MockUart {
        fn read_ready(&mut self) -> core::result::Result<bool, Infallible> {
            Ok(!self.0.is_empty())
        }
    }

    impl Write for MockUart {
        async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Infallible> {
            self.1.extend_from_slice(buf);
            Ok(buf.len())
        }
        async fn flush(&mut self) -> core::result::Result<(), Infallible> {
            Ok(())
        }
    }

    impl Baud for MockUart {
        fn set_baudrate(&mut self, _baudrate: u32) {}
    }

    struct MockPin;

    impl embedded_hal::digital::ErrorType for MockPin {
        type Error = Infallible;
    }

    impl OutputPin for MockPin {
        fn set_low(&mut self) -> core::result::Result<(), Infallible> {
            Ok(())
        }
        fn set_high(&mut self) -> core::result::Result<(), Infallible> {
            Ok(())
        }
    }

    struct MockDelay;

    impl DelayNs for MockDelay {
        async fn delay_ns(&mut self, _ns: u32) {}
    }

    type MockChain = Chain<BM1366, MockUart, MockPin, MockPin, MockDelay>;

    fn chain(rx: &[u8]) -> MockChain {
        Chain {
            asic_cnt: 1,
            asic: BM1366::default(),
            asic_addr_interval: 256,
            domain_cnt: 1,
            uart: MockUart(rx.iter().copied().collect(), Vec::new()),
            rx_buf: [0; RX_BUF_SIZE],
            rx_free_pos: 0,
            rx_dropped: 0,
            busy: MockPin,
            reset: MockPin,
            delay: MockDelay,
            job_id: 0,
            version_rolling_mask: None,
            chip_nonce_space: 0,
            chip_freqs: heapless::Vec::from_slice(&[HertzU64::MHz(50)]).unwrap(),
            chip_nonces: heapless::Vec::from_slice(&[0]).unwrap(),
            ticket_difficulty: 1,
            #[cfg(feature = "nonce-check")]
            jobs: [None; 32],
            #[cfg(feature = "nonce-check")]
            chip_hw_errors: heapless::Vec::from_slice(&[0]).unwrap(),
        }
    }

    async fn poll_all(chain: &mut MockChain) -> Vec<ResponseType> {
        let mut resps = Vec::new();
        for _ in 0..8 {
            if let Some(resp) = chain.poll_response().await.unwrap() {
                resps.push(resp);
            }
        }
        resps
    }

    fn chip_id_resp() -> ResponseType {
        ResponseType::Reg(RegisterResponse {
            chip_addr: 0,
            reg_addr: 0,
            reg_value: 0x1397_1800,
        })
    }

    #[tokio::test]
    async fn poll_response_skips_garbage() {
        let mut chain = chain(&[[0x00, 0x12, 0xAA].as_slice(), &CHIP_ID].concat());
        assert_eq!(poll_all(&mut chain).await, [chip_id_resp()]);
        assert_eq!(chain.rx_dropped_bytes(), 3);
    }

    #[tokio::test]
    async fn poll_response_keeps_frame_after_false_preamble() {
        // a false Preamble whose Frame has a bad CRC must not swallow the real Frame after it
        let mut chain = chain(&[[0xAA, 0x55, 0x01, 0x02].as_slice(), &CHIP_ID].concat());
        assert_eq!(poll_all(&mut chain).await, [chip_id_resp()]);
        assert_eq!(chain.rx_dropped_bytes(), 4);
    }

    #[tokio::test]
    async fn poll_response_keeps_trailing_preamble_byte() {
        let mut chain = chain(&[[0x00; 10].as_slice(), &CHIP_ID[..1]].concat());
        assert_eq!(poll_all(&mut chain).await, []);
        assert_eq!(chain.rx_dropped_bytes(), 10);
        chain.uart.0.extend(&CHIP_ID[1..]);
        assert_eq!(poll_all(&mut chain).await, [chip_id_resp()]);
        assert_eq!(chain.rx_dropped_bytes(), 10);
    }

    /// ChipIdentification register response of a BM1366, with its CRC.
    fn bm1366_chip_id() -> [u8; FRAME_SIZE] {
        let mut frame = [0xAA, 0x55, 0x13, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00];
        frame[8] = (0..0x20)
            .find(|&crc| {
                frame[8] = crc;
                Response::parse(&frame, 8).is_ok()
            })
            .unwrap();
        frame
    }

    async fn enumerate(asic_cnt: usize) -> MockChain {
        let rx = bm1366_chip_id().repeat(asic_cnt);
        let uart = MockUart(rx.into_iter().collect(), Vec::new());
        Chain::enumerate(BM1366::default(), uart, MockPin, MockPin, MockDelay)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn enumerate_one_chip() {
        let chain = enumerate(1).await;
        assert_eq!(chain.asic_cnt, 1);
        assert_eq!(chain.asic_addr_interval, 256);
    }

    #[tokio::test]
    async fn enumerate_three_chips() {
        let chain = enumerate(3).await;
        assert_eq!(chain.asic_cnt, 3);
        assert_eq!(chain.asic_addr_interval, 64);
    }

    /// A chain of `asic_cnt` BM1366 chips `asic_addr_interval` apart in `domain_cnt` domains.
    fn chain_of(asic_cnt: usize, asic_addr_interval: usize, domain_cnt: usize) -> MockChain {
        let mut chain = chain(&[]);
        chain.asic_cnt = asic_cnt;
        chain.asic_addr_interval = asic_addr_interval;
        chain.domain_cnt = domain_cnt;
        chain.chip_freqs = (0..asic_cnt).map(|_| HertzU64::MHz(50)).collect();
        chain.chip_nonces = (0..asic_cnt).map(|_| 0).collect();
        #[cfg(feature = "nonce-check")]
        {
            chain.chip_hw_errors = (0..asic_cnt).map(|_| 0).collect();
        }
        chain
    }

    #[tokio::test]
    async fn set_chip_hash_freqs_moves_one_chip_per_domain_per_pass() {
        let mut chain = chain_of(4, 64, 2);
        let targets = [
            HertzU64::kHz(62_500),
            HertzU64::kHz(62_500),
            HertzU64::MHz(50),
            HertzU64::kHz(56_250),
        ];
        chain
            .set_chip_hash_freqs(&targets, HertzU64::kHz(6250), 0)
            .await
            .unwrap();
        assert_eq!(chain.chip_hash_freqs(), &targets);
        // one PLL0 write per step: chip 0 and chip 3, chip 0, chip 1, chip 1
        let writes: Vec<(u8, u8)> = chain.uart.1.chunks(11).map(|c| (c[4], c[5])).collect();
        assert_eq!(
            writes,
            [
                (0x00, 0x08),
                (0xC0, 0x08),
                (0x00, 0x08),
                (0x40, 0x08),
                (0x40, 0x08)
            ]
        );
    }

    #[tokio::test]
    async fn nonces_are_counted_per_chip() {
        // nonce from chip address 0x80 (bits 24:17), version rolling on
        let nonce: u32 = 0x80 << 17;
        let mut frame = [0xAA, 0x55, 0, 0, 0, 0, 0x00, 0x00, 0x00, 0x00, 0x80];
        frame[2..6].copy_from_slice(&nonce.to_le_bytes());
        frame[10] = (0x80..0xA0)
            .find(|&b| {
                frame[10] = b;
                Response::parse_version(&frame, 8).is_ok()
            })
            .unwrap();
        let mut chain = chain_of(4, 64, 1);
        chain.uart.0 = frame.repeat(3).into_iter().collect();
        chain.version_rolling_mask = Some(0x1fff_e000);
        for _ in 0..8 {
            chain.poll_response().await.unwrap();
        }
        assert_eq!(chain.chip_nonce_counts(), &[0, 0, 3, 0]);
        chain.clear_chip_nonce_counts();
        assert_eq!(chain.chip_nonce_counts(), &[0, 0, 0, 0]);
    }

    #[test]
    fn expected_chip_nonces_follow_frequency_and_ticket_mask() {
        let mut chain = chain_of(1, 256, 1);
        chain.chip_freqs[0] = HertzU64::MHz(500);
        chain.ticket_difficulty = 256;
        // 500 MHz × 894 small cores / 2^32 / 256 = 0.4065 nonce per second
        let n = chain.expected_chip_nonces(0, Duration::from_secs(100));
        assert!((n - 40.65).abs() < 0.01, "{n}");
        assert_eq!(chain.expected_chip_nonces(1, Duration::from_secs(100)), 0.0);
    }

    /// Register response of chip `chip_addr` without version rolling, with its CRC.
    fn reg_frame(value: u32, chip_addr: u8, reg_addr: u8) -> [u8; FRAME_SIZE] {
        let mut frame = [0xAA, 0x55, 0, 0, 0, 0, chip_addr, reg_addr, 0];
        frame[2..6].copy_from_slice(&value.to_be_bytes());
        frame[8] = (0..0x20)
            .find(|&crc| {
                frame[8] = crc;
                Response::parse(&frame, 8).is_ok()
            })
            .unwrap();
        frame
    }

    #[tokio::test]
    async fn read_chip_hash_clock_converts_the_count() {
        let mut chain = chain_of(2, 128, 1);
        chain.uart.0 = reg_frame(0x0001_1800, 128, 0x6C).into_iter().collect();
        assert_eq!(
            chain.read_chip_hash_clock(1).await.unwrap(),
            Some(HertzU64::MHz(600))
        );
        // the read command went to chip 128
        assert_eq!(chain.uart.1[2..6], [0x42, 0x05, 128, 0x6C]);
        chain.uart.0 = reg_frame(0x0000_1800, 0, 0x6C).into_iter().collect();
        assert_eq!(chain.read_chip_hash_clock(0).await.unwrap(), None); // no count ready
    }

    #[tokio::test]
    async fn read_chip_pll_locked_reads_bit_31() {
        let mut chain = chain_of(1, 256, 1);
        chain.uart.0 = reg_frame(0xC0A2_0255, 0, 0x08).into_iter().collect();
        assert_eq!(chain.read_chip_pll_locked(0).await.unwrap(), Some(true));
        chain.uart.0 = reg_frame(0x40A2_0255, 0, 0x08).into_iter().collect();
        assert_eq!(chain.read_chip_pll_locked(0).await.unwrap(), Some(false));
        assert_eq!(chain.read_chip_pll_locked(0).await.unwrap(), None); // no answer
    }

    #[test]
    fn share_difficulty_counts_the_extra_zero_bits() {
        let mut chain = chain(&[]);
        chain.ticket_difficulty = 256;
        let share = |extra_zero_bits| {
            ResponseType::JobVer(JobVersionResponse {
                nonce: 0,
                extra_zero_bits,
                job_id: 0,
                small_core_id: 0,
                version_bit: 0,
            })
        };
        assert_eq!(chain.share_difficulty(&share(0)), Some(256));
        assert_eq!(chain.share_difficulty(&share(3)), Some(2048));
        assert_eq!(chain.share_difficulty(&share(255)), Some(u64::MAX));
        assert_eq!(chain.share_difficulty(&chip_id_resp()), None);
    }

    /// The block header fields of the Bitcoin genesis block, hashes in display order.
    #[cfg(feature = "nonce-check")]
    const GENESIS_MERKLE_ROOT: [u8; 32] = [
        0x4a, 0x5e, 0x1e, 0x4b, 0xaa, 0xb8, 0x9f, 0x3a, 0x32, 0x51, 0x8a, 0x88, 0xc3, 0x1b, 0xc8,
        0x7f, 0x61, 0x8f, 0x76, 0x67, 0x3e, 0x2c, 0xc7, 0x7a, 0xb2, 0x12, 0x7b, 0x7a, 0xfd, 0xed,
        0xa3, 0x3b,
    ];

    #[cfg(feature = "nonce-check")]
    #[tokio::test]
    async fn nonces_are_checked_against_their_job() {
        let mut chain = chain_of(1, 256, 1);
        chain.version_rolling_mask = Some(0x1fff_e000);
        let job_id = chain
            .send_job(1, [0; 32], GENESIS_MERKLE_ROOT, 0x1d00_ffff, 1_231_006_505)
            .await
            .unwrap();
        let share = |nonce| {
            ResponseType::JobVer(JobVersionResponse {
                nonce,
                extra_zero_bits: 0,
                job_id: job_id as usize,
                small_core_id: 0,
                version_bit: 0,
            })
        };
        // the genesis block hash 000000000019d6689c08…
        assert_eq!(chain.share_zero_bits(&share(2_083_236_893)), Some(43));
        chain.count_nonce(&share(2_083_236_893));
        chain.count_nonce(&share(2_083_236_894));
        assert_eq!(chain.chip_nonce_counts(), &[1]);
        assert_eq!(chain.chip_hw_error_counts(), &[1]);
        chain.clear_chip_nonce_counts();
        assert_eq!(chain.chip_hw_error_counts(), &[0]);
    }

    #[tokio::test]
    async fn send_job_cycles_32_job_ids_with_8_small_cores() {
        let mut chain = chain(&[]);
        chain.version_rolling_mask = Some(0x1fff_e000);
        let mut job_ids = Vec::new();
        for _ in 0..33 {
            job_ids.push(chain.send_job(0, [0; 32], [0; 32], 0, 0).await.unwrap());
        }
        assert_eq!(job_ids, (1..32).chain(0..2).collect::<Vec<u8>>());
    }
}
