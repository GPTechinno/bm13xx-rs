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
#![macro_use]
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
        Duration::from_secs_f32(space / (self.asic.hash_freq().raw() as f32) / 1_000.0)
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
    /// In case of receiving a Frame but with bad CRC, this function will ignore the frame and return None.
    /// In case of receiving a Frame but with bad Preamble, this function will try to resync and return None.
    pub async fn poll_response(
        &mut self,
    ) -> Result<Option<ResponseType>, U::Error, OB::Error, OR::Error> {
        let mut resp = None;
        let expected_frame_size = if self.version_rolling_mask.is_some() {
            FRAME_SIZE_VER
        } else {
            FRAME_SIZE
        };

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

        if self.rx_free_pos >= expected_frame_size {
            let frame = &self.rx_buf[..expected_frame_size];
            let used = match if self.version_rolling_mask.is_some() {
                Response::parse_version(
                    frame.try_into().unwrap(),
                    self.asic.core_small_core_count(),
                    self.asic_cnt,
                )
            } else {
                Response::parse(frame.try_into().unwrap(), self.asic.core_small_core_count())
            } {
                Ok(r) => {
                    resp = Some(r);
                    expected_frame_size
                }
                Err(bm13xx_protocol::Error::InvalidCrc { expected, actual }) => {
                    error!(
                        "Ignoring Frame {:x?} with bad CRC: {:02x}!={:02x}",
                        frame, expected, actual
                    );
                    expected_frame_size
                }
                Err(bm13xx_protocol::Error::InvalidPreamble) => {
                    let offset = frame
                        .windows(2)
                        .position(|w| w == [0xAA, 0x55])
                        .unwrap_or(expected_frame_size);
                    error!(
                        "Resync Frame {:x?} because bad preamble, dropping first {} bytes",
                        frame, offset
                    );
                    offset
                }
                Err(bm13xx_protocol::Error::UnsupportedCoreSmallCoreCnt) => {
                    error!(
                        "Ignoring Frame {:x?} because bad CoreSmallCoreCnt {}",
                        frame,
                        self.asic.core_small_core_count()
                    );
                    expected_frame_size
                }
            };
            if self.rx_free_pos > used {
                debug!("copy reminder {} bytes @0", self.rx_free_pos - used);
                self.rx_buf.copy_within(used..self.rx_free_pos, 0);
            }
            self.rx_free_pos -= used;
        }
        Ok(resp)
    }

    /// ## Enumerate all asics on the chain
    ///
    /// Sets the `asic_addr_interval` according to the number of asics enumerated
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
        loop {
            chain.delay.delay_ms(10).await;
            if let Some(resp) = chain.poll_response().await? {
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
            } else {
                break;
            }
        }
        if asic_cnt == 0 {
            return Err(Error::EmptyChain);
        }
        debug!("Enumerated {} asics", asic_cnt);
        chain.asic_addr_interval = 256 / asic_cnt;
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
    pub async fn reset(&mut self) -> Result<(), U::Error, OB::Error, OR::Error> {
        self.reset.set_low().map_err(Error::Reset)?;
        self.asic.reset();
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
                - (self.asic.core_count().leading_zeros().ilog2() + 1)
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
        self.job_id = if self.job_id == 31 {
            0
        } else {
            self.job_id + 1
        };
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
