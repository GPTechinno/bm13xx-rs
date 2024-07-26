use bm1366::BM1366;
use bm13xx_chain::Chain;

use embedded_hal_async::delay::DelayNs;
use embedded_io_adapters::tokio_1::FromTokio;
// use embedded_io_adapters::std::FromStd;
use inquire::Select;
// use serialport::ClearBuffer;
use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tokio_serial::SerialStream;

struct Delay;

impl DelayNs for Delay {
    async fn delay_ns(&mut self, n: u32) {
        sleep(Duration::from_nanos(n.into())).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();

    // use the first arg as serial port, query interactively if not given
    let port = if args.len() == 2 {
        args.pop().unwrap()
    } else {
        let ports = tokio_serial::available_ports()?;
        // let ports = serialport::available_ports()?;
        let ports: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
        Select::new("Which serial port should be used?", ports).prompt()?
    };

    let builder = tokio_serial::new(port, 115_200).timeout(Duration::from_millis(50));
    let serial = SerialStream::open(&builder)?;
    let adapter = FromTokio::new(serial);

    // let serial = serialport::new(port, 115_200)
    //     .timeout(Duration::from_millis(50))
    //     .open()?;
    // let adapter = FromStd::new(serial);

    let bm1366 = BM1366::default();

    let mut chain = Chain::new(1, bm1366, 1, adapter, Delay);
    chain.enumerate().await?;
    println!("Enumerated {} asics", chain.asic_cnt);
    println!("Interval: {}", chain.asic_addr_interval);

    Ok(())
}

// impl embedded_io_async::Error for ConnectError {
//     fn kind(&self) -> embedded_io_async::ErrorKind {
//         match self {
//             ConnectError::ConnectionReset => embedded_io_async::ErrorKind::ConnectionReset,
//             ConnectError::TimedOut => embedded_io_async::ErrorKind::TimedOut,
//             ConnectError::NoRoute => embedded_io_async::ErrorKind::NotConnected,
//             ConnectError::InvalidState => embedded_io_async::ErrorKind::Other,
//         }
//     }
// }
