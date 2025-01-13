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
//! The example below demonstrates how to use it with an ESP32,
//! showcasing the strength of the embedded-hal abstractions.
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use embassy_executor::Spawner;
//! use embassy_time::{Duration, Timer, Delay};
//! use esp_backtrace as _;
//! use esp_hal::{
//!     clock::ClockControl,
//!     gpio::Io,
//!     peripherals::Peripherals,
//!     prelude::*,
//!     system::SystemControl,
//!     timer::timg::TimerGroup,
//!     uart::{config::Config, TxRxPins, Uart},
//! };
//! use esp_println::println;
//! use bm1366::BM1366;
//! use bm13xx_chain::{Baud, Chain};
//!
//! #[main]
//! async fn main(_s: Spawner) -> ! {
//!     let peripherals = Peripherals::take();
//!     let system = SystemControl::new(peripherals.SYSTEM);
//!     let clocks = ClockControl::max(system.clock_control).freeze();
//!
//!     let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
//!     esp_hal_embassy::init(&clocks, timg0);
//!
//!     let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
//!     let pins = TxRxPins::new_tx_rx(io.pins.gpio17, io.pins.gpio16);
//!
//!     let mut uart0 = Uart::new_async_with_config(
//!         peripherals.UART0,
//!         Config::default().baudrate(115_200),
//!         Some(pins),
//!         &clocks,
//!     );
//!     uart0
//!         .set_rx_fifo_full_threshold(bm13xx_protocol::READ_BUF_SIZE as u16)
//!         .unwrap();
//!
//!     let bm1366 = BM1366::default();
//!     let mut chain = Chain::new(1, bm1366, 1, &mut uart0, Delay);
//!     chain.enumerate().await.unwrap();
//!     println!("Enumerated {} asics", chain.asic_cnt);
//!     println!("Interval: {}", chain.asic_addr_interval);
//!     chain.init(256).await.unwrap();
//!     chain.set_baudrate(1_000_000).await.unwrap();
//!
//!     loop {
//!
//!         Timer::after(Duration::from_millis(30)).await;
//!     }
//! }
//! ```

#![no_std]
#![macro_use]
pub(crate) mod fmt;

mod error;

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

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Chain<A, U, OB, OR, D> {
    pub asic_cnt: u8,
    asic: A,
    pub asic_addr_interval: u16,
    domain_cnt: u8,
    uart: U,
    rx_buf: [u8; RX_BUF_SIZE],
    rx_free_pos: usize,
    busy: OB,
    reset: OR,
    delay: D,
    job_id: u8,
}

impl<A: Asic, U: Read + ReadReady + Write + Baud, OB: OutputPin, OR: OutputPin, D: DelayNs>
    Chain<A, U, OB, OR, D>
{
    pub fn new(
        asic_cnt: u8,
        asic: A,
        domain_cnt: u8,
        uart: U,
        busy: OB,
        reset: OR,
        delay: D,
    ) -> Self {
        Chain::<A, U, OB, OR, D> {
            asic_cnt,
            asic,
            asic_addr_interval: 0,
            domain_cnt,
            uart,
            rx_buf: [0; RX_BUF_SIZE],
            rx_free_pos: 0,
            busy,
            reset,
            delay,
            job_id: 0,
        }
    }

    async fn send(&mut self, step: CmdDelay) -> Result<(), U::Error, OB::Error, OR::Error> {
        self.uart.write_all(&step.cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(step.delay_ms).await;
        Ok(())
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
        let expected_frame_size = if self.asic.version_rolling_enabled() {
            FRAME_SIZE_VER
        } else {
            FRAME_SIZE
        };
        if self.rx_free_pos >= expected_frame_size {
            let frame = &self.rx_buf[..expected_frame_size];
            let used = match if self.asic.version_rolling_enabled() {
                Response::parse_version(frame.try_into().unwrap())
            } else {
                Response::parse(frame.try_into().unwrap())
            } {
                Ok(r) => {
                    resp = Some(r);
                    expected_frame_size
                }
                Err(bm13xx_protocol::Error::InvalidCrc { expected, actual }) => {
                    error!(
                        "Ignoring Frame {:x} with bad CRC: {:02x}!={:02x}",
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
                        "Resync Frame {:x} because bad preamble, dropping first {} bytes",
                        frame, offset
                    );
                    offset
                }
            };
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
    /// - Unexpected asic count
    pub async fn enumerate(&mut self) -> Result<(), U::Error, OB::Error, OR::Error> {
        self.reset.set_high().map_err(Error::Reset)?;
        self.delay.delay_ms(10).await;
        self.busy.set_low().map_err(Error::Busy)?;
        let cmd = Command::read_reg(ChipIdentification::ADDR, Destination::All);
        self.uart.write_all(&cmd).await.map_err(Error::Io)?;

        let mut asic_cnt = 0;
        let mut post_s19jpro = false;
        loop {
            debug!("Enumerating asic: {}", asic_cnt);
            // FIXME: This is a workaround for the Timeout based loop
            if asic_cnt == self.asic_cnt {
                break;
            }
            // TODO: fix the Timeout based loop
            if let Some(resp) = self.poll_response().await? {
                if let ResponseType::Reg(reg_resp) = resp {
                    if reg_resp.chip_addr != 0 || reg_resp.reg_addr != ChipIdentification::ADDR {
                        warn!("reg_resp: {:#?}, {}", reg_resp, ChipIdentification::ADDR);
                        return Err(Error::BadRegisterResponse { reg_resp });
                    }
                    let chip_ident = ChipIdentification(reg_resp.reg_value);
                    if chip_ident.core_num() == 0 {
                        post_s19jpro = true;
                    }
                    if chip_ident.chip_id() == self.asic.chip_id() {
                        asic_cnt += 1;
                    } else {
                        return Err(Error::UnexpectedAsic { chip_ident });
                    }
                } else {
                    return Err(Error::UnexpectedResponse { resp });
                };
            }
        }
        if asic_cnt > 0 {
            self.asic_addr_interval = 256 / (asic_cnt as u16);
        }
        if asic_cnt != self.asic_cnt {
            return Err(Error::UnexpectedAsicCount {
                expected_asic_cnt: self.asic_cnt,
                actual_asic_cnt: asic_cnt,
            });
        }
        self.delay.delay_ms(50).await;
        if post_s19jpro {
            self.delay.delay_ms(100).await;
            while let Some(step) = self.asic.reset_core_next(Destination::All) {
                self.send(step).await?;
            }
        }
        let cmd = Command::chain_inactive();
        self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        if !post_s19jpro {
            self.delay.delay_ms(2).await;
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(2).await;
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        }
        self.delay.delay_ms(30).await;
        for i in 0..asic_cnt {
            let cmd = Command::set_chip_addr((i as u16 * self.asic_addr_interval) as u8);
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(10).await;
        }
        self.delay.delay_ms(100).await;
        Ok(())
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
    pub async fn init(&mut self, diffculty: u32) -> Result<(), U::Error, OB::Error, OR::Error> {
        while let Some(step) = self.asic.init_next(diffculty) {
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
                .reset_core_next(Destination::Chip(asic_i * self.asic_addr_interval as u8))
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

    /// ## Enable Version Rolling in chips
    ///
    /// Enable Hardware Version Rolling with the given version mask.
    pub async fn enable_version_rolling(
        &mut self,
        mask: u32,
    ) -> Result<(), U::Error, OB::Error, OR::Error> {
        if !self.asic.version_rolling_enabled() {
            while let Some(step) = self.asic.set_version_rolling_next(mask) {
                self.send(step).await?;
            }
            self.delay.delay_ms(100).await;
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
        self.job_id = self.job_id.wrapping_add(self.asic.core_small_core_count());
        if self.asic.version_rolling_enabled() {
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
            let cmd =
                Command::job_midstate(self.job_id, n_bits, n_time, merkle_root_end, midstates);
            self.uart.write_all(&cmd).await.map_err(Error::Io)?;
        };
        Ok(self.job_id)
    }
}
