#![allow(dead_code)]

use bm1366::BM1366;
use bm13xx_chain::Chain;

use embedded_hal_async::delay::DelayNs;
use fugit::HertzU64;
use inquire::Select;
use linux_embedded_hal::SysfsPin;
use std::{env, error::Error, time::Duration};
use tokio::time::sleep;
use tokio_adapter::FromTokio;
use tokio_serial::SerialStream;

struct Delay;

impl DelayNs for Delay {
    async fn delay_ns(&mut self, n: u32) {
        sleep(Duration::from_nanos(n.into())).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut args: Vec<String> = env::args().collect();

    // use the first arg as serial port, query interactively if not given
    let port = if args.len() == 2 {
        args.pop().unwrap()
    } else {
        let ports = tokio_serial::available_ports()?;
        let ports: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
        Select::new("Which serial port should be used?", ports).prompt()?
    };

    let builder = tokio_serial::new(port, 115_200).timeout(Duration::from_millis(50));
    let serial = SerialStream::open(&builder)?;
    let uart = FromTokio::new(serial);

    let bm1366 = BM1366::default();
    let fake_busy = SysfsPin::new(127);
    let fake_reset = SysfsPin::new(128);

    let mut chain = Chain::new(1, bm1366, 1, uart, fake_busy, fake_reset, Delay);
    chain.enumerate().await?;
    println!("Enumerated {} asics", chain.asic_cnt);
    println!("Interval: {}", chain.asic_addr_interval);
    chain.init(256).await?;
    chain.set_baudrate(1_000_000).await?;
    // chain.enumerate().await?; // just to be sure the new baudrate is well setup
    // println!("Enumerated {} asics", chain.asic_cnt);
    // println!("Interval: {}", chain.asic_addr_interval);
    chain.reset_all_cores().await?;
    chain.set_hash_freq(HertzU64::MHz(525)).await?;
    chain.set_version_rolling(0x1fff_e000).await?;
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
