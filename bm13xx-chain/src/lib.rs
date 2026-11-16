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

use core::time::Duration;

pub use self::error::{Error, Result};

use bm13xx_asic::{register::ChipIdentification, Asic, CmdDelay};
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

const NONCE_BITS: u32 = u32::BITS;
const CHIP_ADDR_BITS: u32 = u8::BITS;

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
        Duration::from_secs_f32(space / (self.asic.hash_freq().raw() as f32))
    }

    /// ## Get the theoretical Hashrate in GH/s
    pub fn theoretical_hashrate_ghs(&self) -> f32 {
        (self.asic.hash_freq().raw() as f32 * self.asic.small_core_count() as f32 / 1_000_000_000.0)
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
        Ok(resp)
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
        };

        chain.reset.set_high().map_err(Error::Reset)?;
        chain.delay.delay_ms(10).await;
        chain.busy.set_low().map_err(Error::Busy)?;
        let cmd = Command::read_reg(ChipIdentification::ADDR, Destination::All);
        chain.uart.write_all(&cmd).await.map_err(Error::Io)?;

        let mut asic_cnt = 0;
        let mut post_s19jpro = false;
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
                    if chip_ident.core_num() == 0 {
                        post_s19jpro = true;
                    }
                    if chip_ident.chip_id() == chain.asic.chip_id() {
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
        chain.asic_addr_interval = 1 << (256usize / asic_cnt).max(1).ilog2();
        chain.asic_cnt = asic_cnt;
        chain.chip_nonce_space = chain.asic_addr_interval
            << (NONCE_BITS
                - (chain.asic.core_count().ilog2() + 1)
                - (chain.asic.core_small_core_count().ilog2() + 1)
                - CHIP_ADDR_BITS);
        // TODO: try to determine domain_cnt according to known topologies
        chain.delay.delay_ms(50).await;
        if post_s19jpro {
            chain.delay.delay_ms(100).await;
            while let Some(step) = chain.asic.reset_core_next(Destination::All) {
                chain.send(step).await?;
            }
        }
        let cmd = Command::chain_inactive();
        chain.uart.write_all(&cmd).await.map_err(Error::Io)?;
        if !post_s19jpro {
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
        self.delay.delay_ms(100).await;
        Ok(())
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
                - (self.asic.core_count().ilog2() + 1)
                - if self.version_rolling_mask.is_none() {
                    0
                } else {
                    self.asic.core_small_core_count().ilog2() + 1
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
            while let Some(step) = self.asic.set_version_rolling_next(mask) {
                self.send(step).await?;
            }
            self.delay.delay_ms(100).await;
            self.version_rolling_mask = Some(mask);
            // when hw version rolling is enabled, the small cores split version_space and not nonce_space anymore
            self.chip_nonce_space <<= self.asic.core_small_core_count().ilog2() + 1;
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
        // TODO: store the job in a `heapless::HistoryBuffer` to be able to compute corrsponding share difficulty
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
    use bm13xx_protocol::response::RegisterResponse;
    use core::convert::Infallible;
    use std::{collections::VecDeque, vec::Vec};

    /// ChipIdentification register response of a BM1397.
    const CHIP_ID: [u8; FRAME_SIZE] = [0xAA, 0x55, 0x13, 0x97, 0x18, 0x00, 0x00, 0x00, 0x06];

    /// A serial interface that returns the queued bytes and discards what is written.
    struct MockUart(VecDeque<u8>);

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
            Ok(buf.len())
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
            uart: MockUart(rx.iter().copied().collect()),
            rx_buf: [0; RX_BUF_SIZE],
            rx_free_pos: 0,
            rx_dropped: 0,
            busy: MockPin,
            reset: MockPin,
            delay: MockDelay,
            job_id: 0,
            version_rolling_mask: None,
            chip_nonce_space: 0,
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
        let uart = MockUart(rx.into_iter().collect());
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
