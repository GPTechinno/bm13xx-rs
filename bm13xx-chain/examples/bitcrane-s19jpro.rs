#![allow(dead_code)]

use bm1366::BM1366;
use bm13xx_chain::Chain;

use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_bus::i2c::MutexDevice;
use inquire::Select;
use std::{error::Error, sync::Mutex, time::Duration};
use tokio::time::sleep;
use tokio_adapter::FromTokio;
use tokio_serial::SerialStream;

struct Delay;

impl DelayNs for Delay {
    async fn delay_ns(&mut self, n: u32) {
        sleep(Duration::from_nanos(n.into())).await;
    }
}

struct FakeOutputPin;

impl ErrorType for FakeOutputPin {
    type Error = core::convert::Infallible;
}

impl OutputPin for FakeOutputPin {
    fn set_high(&mut self) -> Result<(), core::convert::Infallible> {
        Ok(())
    }
    fn set_low(&mut self) -> Result<(), core::convert::Infallible> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Open FTDI FT4232H channel A from Bitcrane for APW and I2C connection
    let channel_a = ftdi::find_by_vid_pid(0x0403, 0x6011)
        .interface(ftdi::Interface::A)
        .open()
        .expect("Failed to open FTDI FT4232H channel A");
    let hal_ch_a = ftdi_embedded_hal::FtHal::init_freq(channel_a, 400_000)?;
    let i2c = hal_ch_a.i2c()?;
    let _psu_en = hal_ch_a.ad4()?;
    let _apw12_i2c = MutexDevice::new(&Mutex::new(&i2c));
    // TODO: use a apw device driver using apw12_i2c i2c@16
    let _fan_alert = hal_ch_a.adi6()?;
    let _emc2305_i2c = MutexDevice::new(&Mutex::new(&i2c));
    // TODO: use a emc2305 device driver using emc2305_i2c i2c@0x4d

    let bm1362 = BM1366::default(); // TODO: change it to BM1362 when implemented

    // Only use HB0 for now
    // Open FTDI FT4232H channel B from Bitcrane for HB0 connection
    let channel_b = ftdi::find_by_vid_pid(0x0403, 0x6011)
        .interface(ftdi::Interface::B)
        .open()
        .expect("Failed to open FTDI FT4232H channel B");
    let hal_ch_b = ftdi_embedded_hal::FtHal::init_default(channel_b)?;
    let mut hb0_plug0 = hal_ch_b.adi3()?;
    if hb0_plug0.is_high().unwrap() {
        let hb0_busy = FakeOutputPin;
        let hb0_reset = hal_ch_b.ad2()?;
        let _hb0_pic_i2c = MutexDevice::new(&Mutex::new(&i2c));
        // TODO: use a pic1704 device driver using hb0_pic_i2c i2c@32
        let _hb0_temp_a_i2c = MutexDevice::new(&Mutex::new(&i2c));
        // TODO: use a lm75a device driver using hb0_temp_a_i2c i2c@72
        let _hb0_temp_b_i2c = MutexDevice::new(&Mutex::new(&i2c));
        // TODO: use a lm75a device driver using hb0_temp_b_i2c i2c@76

        // query interactively the serial port
        // TODO: try to get it from the FTDI device
        let hb0_port = {
            let ports = tokio_serial::available_ports()?;
            let ports: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
            Select::new("Which serial port should be used?", ports).prompt()?
        };
        let hb0_builder = tokio_serial::new(hb0_port, 115_200).timeout(Duration::from_millis(50));
        let hb0_serial = SerialStream::open(&hb0_builder)?;
        let hb0_uart = FromTokio::new(hb0_serial);

        let mut chain0 = Chain::new(126, bm1362, 42, hb0_uart, hb0_busy, hb0_reset, Delay);
        chain0.enumerate().await?;
        println!("Enumerated {} asics", chain0.asic_cnt);
        println!("Interval: {}", chain0.asic_addr_interval);
        chain0.init(256).await?;
        chain0.change_baudrate(1_000_000).await?;
        // chain0.enumerate().await?; // just to be sure the new baudrate is well setup
        // println!("Enumerated {} asics", chain0.asic_cnt);
        // println!("Interval: {}", chain0.asic_addr_interval);
        chain0.reset_all_cores().await?;
        // chain0.set_hash_freq(HertzU64::MHz(525)).await?;
        // chain0.enable_version_rolling(0x1fff_e000).await?;
    } else {
        println!("HB0 is not connected");
    }
    Ok(())
}

mod tokio_adapter {
    //! Adapters to/from `tokio::io` traits.

    use core::future::poll_fn;
    use core::pin::Pin;
    use core::task::Poll;

    /// Adapter from `tokio::io` traits.
    #[derive(Clone)]
    pub struct FromTokio<T: ?Sized> {
        inner: T,
    }

    impl<T> FromTokio<T> {
        /// Create a new adapter.
        pub fn new(inner: T) -> Self {
            Self { inner }
        }

        /// Consume the adapter, returning the inner object.
        pub fn into_inner(self) -> T {
            self.inner
        }
    }

    impl<T: ?Sized> FromTokio<T> {
        /// Borrow the inner object.
        pub fn inner(&self) -> &T {
            &self.inner
        }

        /// Mutably borrow the inner object.
        pub fn inner_mut(&mut self) -> &mut T {
            &mut self.inner
        }
    }

    impl<T: ?Sized> embedded_io::ErrorType for FromTokio<T> {
        type Error = std::io::Error;
    }

    impl<T: tokio::io::AsyncRead + Unpin + ?Sized> embedded_io_async::ReadReady for FromTokio<T> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(true) // TODO: fix this
        }
    }

    impl<T: tokio::io::AsyncRead + Unpin + ?Sized> embedded_io_async::Read for FromTokio<T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            // The current tokio implementation (https://github.com/tokio-rs/tokio/blob/tokio-1.33.0/tokio/src/io/poll_evented.rs#L165)
            // does not consider the case of buf.is_empty() as a special case,
            // which can cause Poll::Pending to be returned at the end of the stream when called with an empty buffer.
            // This poll will, however, never become ready, as no more bytes will be received.
            if buf.is_empty() {
                return Ok(0);
            }

            poll_fn(|cx| {
                let mut buf = tokio::io::ReadBuf::new(buf);
                match Pin::new(&mut self.inner).poll_read(cx, &mut buf) {
                    Poll::Ready(r) => match r {
                        Ok(()) => Poll::Ready(Ok(buf.filled().len())),
                        Err(e) => Poll::Ready(Err(e)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            })
            .await
        }
    }

    impl<T: tokio::io::AsyncWrite + Unpin + ?Sized> embedded_io_async::Write for FromTokio<T> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            match poll_fn(|cx| Pin::new(&mut self.inner).poll_write(cx, buf)).await {
                Ok(0) if !buf.is_empty() => Err(std::io::ErrorKind::WriteZero.into()),
                Ok(n) => Ok(n),
                Err(e) => Err(e),
            }
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            poll_fn(|cx| Pin::new(&mut self.inner).poll_flush(cx)).await
        }
    }

    impl<T: tokio_serial::SerialPort> bm13xx_chain::Baud for FromTokio<T> {
        fn set_baudrate(&mut self, baudrate: u32) {
            self.inner_mut().set_baud_rate(baudrate).unwrap()
        }
    }
}
