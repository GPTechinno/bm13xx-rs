extern crate bm13xx_protocol;

use std::thread::sleep;
use std::time::Duration;

use bm13xx_protocol::command::{Command, Destination};
use bm13xx_protocol::response::{Response, ResponseType};

fn main() {
    env_logger::init();

    // let ports = serialport::available_ports().expect("No ports found!");
    // for p in ports {
    //     println!("{}", p.port_name);
    // }
    let mut port = serialport::new("/dev/ttyUSB0", 115_200)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open port");

    // loop over all possible register addresses
    for reg_addr in (0x00u8..0xFC).step_by(4) {
        let cmd = Command::read_reg(reg_addr, Destination::Chip(0));
        println!(">> {:x?}", cmd);
        port.write_all(&cmd).expect("Write failed!");

        let mut resp: [u8; 9] = [0u8; 9];
        port.read_exact(&mut resp).expect("Found no data!");
        println!("<< {:x?}", resp);
        match Response::parse(&resp).expect("Error parsing") {
            ResponseType::Reg(reg) => println!("{:x?}", reg),
            ResponseType::Job(job) => println!("{:x?}", job),
            ResponseType::JobVer(job) => println!("{:x?}", job),
        };
        sleep(Duration::from_millis(50));
    }
}
