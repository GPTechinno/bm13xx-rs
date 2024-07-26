//! BM13xx Chain representation.
#![no_std]
// #![feature(error_in_core)]
// #![allow(stable_features, reason = "remove this once rust 1.81 is stable")]
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

    pub async fn set_baudrate(&mut self, baudrate: u32) -> Result<(), P::Error> {
        let steps = self.asic.set_baudrate(baudrate);
        self.send(steps.iter()).await?;
        self.delay.delay_ms(50).await;
        self.port.set_baudrate(baudrate);
        Ok(())
    }

    pub async fn init(&mut self, initial_diffculty: u32) -> Result<(), P::Error> {
        let steps = self.asic.init(
            initial_diffculty,
            self.domain_cnt,
            self.asic_cnt / self.domain_cnt,
            self.asic_addr_interval,
        );
        self.send(steps.iter()).await?;
        self.delay.delay_ms(100).await;
        Ok(())
    }
}
