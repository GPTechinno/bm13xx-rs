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
#![allow(stable_features)] // remove this once rust 1.81 is stable
#![feature(error_in_core)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]

mod error;

pub use self::error::{Error, Result};

use bm13xx_asic::{register::ChipIdentification, Asic, CmdDelay};
use bm13xx_protocol::{
    command::{Command, Destination},
    response::{Response, ResponseType},
};

use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, Write};
use fugit::HertzU64;

pub trait Baud {
    fn set_baudrate(&mut self, baudrate: u32);
}

pub struct Chain<A, P, D> {
    pub asic_cnt: u8,
    asic: A,
    pub asic_addr_interval: u16,
    domain_cnt: u8,
    port: P,
    delay: D,
}

impl<A: Asic, P: Read + Write + Baud, D: DelayNs> Chain<A, P, D> {
    pub fn new(asic_cnt: u8, asic: A, domain_cnt: u8, port: P, delay: D) -> Self {
        Chain::<A, P, D> {
            asic_cnt,
            asic,
            asic_addr_interval: 0,
            domain_cnt,
            port,
            delay,
        }
    }

    /// ## Enumerate all asics on the chain
    ///
    /// Sets the `asic_addr_interval` according to the number of asics enumerated
    ///
    /// ### Errors
    ///
    /// - I/O error
    /// - Unexpected response
    /// - Bad register response
    /// - Unexpected asic
    /// - Protocol error
    /// - Unexpected asic count
    pub async fn enumerate(&mut self) -> Result<(), P::Error> {
        let cmd = Command::read_reg(ChipIdentification::ADDR, Destination::All);
        self.port.write_all(&cmd).await.map_err(Error::Io)?;

        let mut asic_cnt = 0;
        // loop { // TODO: fix the Timeout based loop
        let mut resp = [0u8; 9];
        self.port.read(&mut resp).await.map_err(Error::Io)?;
        if let ResponseType::Reg(reg_resp) = Response::parse(&resp)? {
            if reg_resp.chip_addr != 0 || reg_resp.reg_addr != ChipIdentification::ADDR {
                return Err(Error::BadRegisterResponse { reg_resp });
            }
            let chip_ident = ChipIdentification(reg_resp.reg_value);
            if chip_ident.chip_id() == self.asic.chip_id() {
                asic_cnt += 1;
            } else {
                return Err(Error::UnexpectedAsic { chip_ident });
            }
        } else {
            return Err(Error::UnexpectedResponse { resp });
        };
        // }
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
        let cmd = Command::chain_inactive();
        self.port.write_all(&cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(2).await;
        self.port.write_all(&cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(2).await;
        self.port.write_all(&cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(30).await;
        for i in 0..asic_cnt {
            let cmd = Command::set_chip_addr((i as u16 * self.asic_addr_interval) as u8);
            self.port.write_all(&cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(10).await;
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    async fn send(&mut self, steps: impl Iterator<Item = &CmdDelay>) -> Result<(), P::Error> {
        for step in steps {
            self.port.write_all(&step.cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(step.delay_ms).await;
        }
        Ok(())
    }

    pub async fn init(&mut self, initial_diffculty: u32) -> Result<(), P::Error> {
        let steps = self.asic.send_init(
            initial_diffculty,
            self.domain_cnt,
            self.asic_cnt / self.domain_cnt,
            self.asic_addr_interval,
        );
        self.send(steps.iter()).await?;
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn set_baudrate(&mut self, baudrate: u32) -> Result<(), P::Error> {
        let steps = self.asic.send_baudrate(baudrate);
        self.send(steps.iter()).await?;
        self.delay.delay_ms(50).await;
        self.port.set_baudrate(baudrate);
        self.delay.delay_ms(50).await;
        Ok(())
    }

    pub async fn reset_core(&mut self) -> Result<(), P::Error> {
        for asic_i in 0..self.asic_cnt {
            let steps = self
                .asic
                .send_reset_core(Destination::Chip(asic_i * self.asic_addr_interval as u8));
            self.send(steps.iter()).await?;
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn set_hash_freq(&mut self, freq: HertzU64) -> Result<(), P::Error> {
        let steps = self.asic.send_hash_freq(freq);
        self.send(steps.iter()).await?;
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn set_version_rolling(&mut self, mask: u32) -> Result<(), P::Error> {
        if self.asic.has_version_rolling() {
            let steps = self.asic.send_version_rolling(mask);
            self.send(steps.iter()).await?;
            self.delay.delay_ms(100).await;
        }
        Ok(())
    }
}
