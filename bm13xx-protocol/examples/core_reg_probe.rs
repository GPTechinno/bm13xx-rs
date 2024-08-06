extern crate bm13xx_protocol;

use std::thread::sleep;
use std::time::Duration;

use bm13xx_protocol::command::{Command, Destination};

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

    // BM1366 need this
    /* Soft Open Core */
    println!("Soft Open Core");
    let cmd = Command::write_reg(0xA8, 0x0007_010f, Destination::All);
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");
    let cmd = Command::write_reg(0x18, 0xFF00_C100, Destination::All);
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");
    sleep(Duration::from_millis(200));

    let cmd = Command::write_reg(0xA8, 0x0007_0000, Destination::All);
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");
    let cmd = Command::write_reg(0x18, 0xFF0F_C100, Destination::All);
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");
    sleep(Duration::from_millis(400));

    let cmd = Command::write_reg(0x3c, 0x8000_82aa, Destination::All);
    println!(">> {:x?}", cmd);
    port.write_all(&cmd).expect("Write failed!");
    sleep(Duration::from_millis(200));

    // loop over all possible core register id (5 bits)
    println!("Read all Core Registers");
    for core_reg_id in 0u32..32 {
        if core_reg_id == 4 {
            let cmd = Command::write_reg(0x3c, 0x8000_8301, Destination::Chip(0));
            println!(">> {:x?}", cmd);
            port.write_all(&cmd).expect("Write failed!");
            sleep(Duration::from_millis(10));
        }
        if core_reg_id == 16 {
            let cmd = Command::write_reg(0x3c, 0x8000_8f06, Destination::Chip(0));
            println!(">> {:x?}", cmd);
            port.write_all(&cmd).expect("Write failed!");
            sleep(Duration::from_millis(10));
        }
        let cmd = Command::write_reg(0x3c, 0x8000_00ff + (core_reg_id << 8), Destination::Chip(0));
        println!(">> {:x?}", cmd);
        port.write_all(&cmd).expect("Write failed!");

        sleep(Duration::from_millis(10));

        let cmd = Command::read_reg(0x40, Destination::Chip(0));
        println!(">> {:x?}", cmd);
        port.write_all(&cmd).expect("Write failed!");

        let mut resp = [0u8; 9];
        if port.read_exact(&mut resp).is_ok() {
            println!("<< {:x?}", resp);
            println!(".insert({}, 0x{:02x}).unwrap()", core_reg_id, resp[5]);
        }
        while port.read_exact(&mut resp).is_ok() {
            println!("<< {:x?}", resp);
        }
        sleep(Duration::from_millis(100));
    }
}
