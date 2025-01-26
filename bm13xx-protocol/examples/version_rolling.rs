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

    // Read ChipIdentification before enabling Version Rolling
    let cmd = Command::read_reg(0x00, Destination::Chip(0));
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");

    let mut resp = [0u8; 9];
    port.read_exact(&mut resp).expect("Found no data!");
    println!("<< {:x?}", resp);
    match Response::parse(&resp).expect("Error parsing") {
        ResponseType::Reg(reg) => println!("{:x?}", reg),
        ResponseType::Job(job) => println!("{:x?}", job),
        ResponseType::JobVer(job) => println!("{:x?}", job),
    };

    sleep(Duration::from_millis(50));

    // Enable Version Rolling
    let cmd = Command::write_reg(0xA4, 0x8000FFFF, Destination::Chip(0));
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");

    sleep(Duration::from_millis(50));

    // Read ChipIdentification after enabling Version Rolling
    let cmd = Command::read_reg(0x00, Destination::Chip(0));
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");

    let mut resp = [0u8; 11];
    port.read_exact(&mut resp).expect("Found no data!");
    println!("<< {:x?}", resp);
    match Response::parse_version(&resp, 8).expect("Error parsing") {
        ResponseType::Reg(reg) => println!("{:x?}", reg),
        ResponseType::Job(job) => println!("{:x?}", job),
        ResponseType::JobVer(job) => println!("{:x?}", job),
    };
}
