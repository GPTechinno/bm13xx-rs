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
    response::{Response, ResponseType},
};

use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, Write};
use fugit::HertzU64;

use bm13xx_asic::register::AnalogMuxControlV2;
use bm13xx_asic::register::CoreRegisterControl;

pub trait Baud {
    fn set_baudrate(&mut self, baudrate: u32);
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
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
        let mut post_s19jpro = false;
        loop {
            debug!("Enumerating asic: {}", asic_cnt);
            // FIXME: This is a workaround for the Timeout based loop
            if asic_cnt == self.asic_cnt {
                break;
            }
            // TODO: fix the Timeout based loop
            let mut resp = [0u8; 9];
            match self.port.read(&mut resp).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error reading response: {:?}", e);
                    continue;
                }
            }
            // self.port.read(&mut resp).await.map_err(Error::Io)?;
            if let ResponseType::Reg(reg_resp) = Response::parse(&resp)? {
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
        self.port.write_all(&cmd).await.map_err(Error::Io)?;
        if !post_s19jpro {
            self.delay.delay_ms(2).await;
            self.port.write_all(&cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(2).await;
            self.port.write_all(&cmd).await.map_err(Error::Io)?;
        }
        self.delay.delay_ms(30).await;
        for i in 0..asic_cnt {
            let cmd = Command::set_chip_addr((i as u16 * self.asic_addr_interval) as u8);
            self.port.write_all(&cmd).await.map_err(Error::Io)?;
            self.delay.delay_ms(10).await;
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn send(&mut self, step: CmdDelay) -> Result<(), P::Error> {
        self.port.write_all(&step.cmd).await.map_err(Error::Io)?;
        self.delay.delay_ms(step.delay_ms).await;
        Ok(())
    }

    pub async fn send_job(&mut self, job: &[u8]) -> Result<u8, P::Error> {
        self.port.write_all(job).await.map_err(Error::Io)?;
        Ok(job.len() as u8)
    }

    pub async fn read_job(&mut self, job: &mut [u8]) -> Result<u8, P::Error> {
        self.port.read_exact(job).await.map_err(Error::Io).unwrap();
        Ok(job.len() as u8)
    }

    pub async fn init(&mut self, diffculty: u32) -> Result<(), P::Error> {
        while let Some(step) = self.asic.init_next(diffculty) {
            self.send(step).await?;
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn set_baudrate(&mut self, baudrate: u32) -> Result<(), P::Error> {
        while let Some(step) = self.asic.set_baudrate_next(
            baudrate,
            self.domain_cnt,
            self.asic_cnt / self.domain_cnt,
            self.asic_addr_interval,
        ) {
            self.send(step).await?;
        }
        self.delay.delay_ms(50).await;
        self.port.set_baudrate(baudrate);
        self.delay.delay_ms(50).await;
        Ok(())
    }

    pub async fn reset_all_cores(&mut self) -> Result<(), P::Error> {
        for asic_i in 0..self.asic_cnt {
            while let Some(step) = self
                .asic
                .reset_core_next(Destination::Chip(asic_i * self.asic_addr_interval as u8))
            {
                self.send(step).await?;
            }
        }
        self.delay.delay_ms(290).await;

        self.send(CmdDelay {
            cmd: Command::write_reg(0xB9, 0x0000_4480, Destination::All),
            delay_ms: 20,
        })
        .await
        .unwrap();
        self.send(CmdDelay {
            cmd: Command::write_reg(AnalogMuxControlV2::ADDR, 0x0000_0002, Destination::All),
            delay_ms: 100,
        })
        .await
        .unwrap();
        self.send(CmdDelay {
            cmd: Command::write_reg(0xB9, 0x0000_4480, Destination::All),
            delay_ms: 20,
        })
        .await
        .unwrap();
        self.send(CmdDelay {
            cmd: Command::write_reg(CoreRegisterControl::ADDR, 0x8000_8DEE, Destination::All),
            delay_ms: 100,
        })
        .await
        .unwrap();

        self.delay.delay_ms(100).await;

        Ok(())
    }

    pub async fn set_hash_freq(&mut self, freq: HertzU64) -> Result<(), P::Error> {
        while let Some(step) = self.asic.set_hash_freq_next(freq) {
            self.send(step).await?;
        }
        self.delay.delay_ms(100).await;
        Ok(())
    }

    pub async fn set_version_rolling(&mut self, mask: u32) -> Result<(), P::Error> {
        if self.asic.has_version_rolling() {
            while let Some(step) = self.asic.set_version_rolling_next(mask) {
                self.send(step).await?;
            }
            self.delay.delay_ms(100).await;
        }
        Ok(())
    }
}
