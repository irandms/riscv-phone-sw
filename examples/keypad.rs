#![no_std]
#![no_main]
#![feature(asm, fn_traits)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x::Peripherals;
use hifive1::hal::stdout::*;

#[path="../src/sc18is600.rs"]
mod sc18is600;

use sc18is600::Sc18is600;

fn delay_ms(ms: u32) {
    let clint_ptr = e310x::CLINT::ptr();
    unsafe {
        let mtime_reg = &(*clint_ptr).mtime;
        let goal = mtime_reg.read().bits() + 32 * ms;
        while mtime_reg.read().bits() < goal {
            asm!("NOP");
        }
    }
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let _clint = p.CLINT.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 16.mhz().into());

    let mut gpio = p.GPIO0.split();
    let (tx, rx) = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, _) = serial.split();
    writeln!(Stdout(&mut tx), "\n\n\nTCA8418 through SC18IS600 Example").unwrap();

    let mosi = gpio.pin3.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let miso = gpio.pin4.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let sck = gpio.pin5.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let bridge_int = gpio.pin22.into_pull_up_input(&mut gpio.pullup, &mut gpio.input_en, &mut gpio.iof_en);
    let mut bridge_cs = gpio.pin10.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);

    let qspi = p.QSPI1;
    let mut bridge = Sc18is600 {
        cs: &mut bridge_cs,
        clocks: clocks,
    };

    let printf_dbg = true;
    let keypad_addr = 0b01101000;
    let keypad_reg_cfg_addr = 0x01;
    let mut keypad_reg_cfg_contents = 0x11; // 0b_0001_0001
    let mut stat_msg = "";
    let mut i2c_buf: [u8; 96] = [0;96];

    // Read cfg reg
    if printf_dbg {
        writeln!(Stdout(&mut tx), "\nReading keypad config register (0x{:02x})", keypad_reg_cfg_addr).unwrap();
    }
    i2c_buf[0] = keypad_reg_cfg_addr;
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.read_after_write(keypad_addr, &mut i2c_buf[..1], 1);
        }
    );
    //while bridge_int.is_high() {};
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            stat_msg = w.get_bus_status();
            w.read_buffer(&mut i2c_buf[..1]);
        }
    );
    if printf_dbg {
        writeln!(Stdout(&mut tx), "Bus status: {}", stat_msg).unwrap();
        write!(Stdout(&mut tx), "Buffer contents: [").unwrap();
        for byte in i2c_buf[..1].into_iter() {
            write!(Stdout(&mut tx), "{:08b},", byte).unwrap();
        }
        writeln!(Stdout(&mut tx), "]").unwrap();

        // Write cfg reg
        writeln!(Stdout(&mut tx), "\nWriting 0x{:x} to keypad config register (0x{:02x})", 
                keypad_reg_cfg_contents, 
                keypad_reg_cfg_addr)
        .unwrap();
    }
    i2c_buf[0] = keypad_reg_cfg_addr;
    i2c_buf[1] = keypad_reg_cfg_contents;
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.write_n_bytes(keypad_addr, &mut i2c_buf[..2]);
        }
    );
    while bridge_int.is_high() {};
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            stat_msg = w.get_bus_status();
        }
    );
    if printf_dbg {
        writeln!(Stdout(&mut tx), "Bus status: {}", stat_msg).unwrap();

        // Read cfg reg
        writeln!(Stdout(&mut tx), "\nReading keypad config register (0x{:02x})", keypad_reg_cfg_addr).unwrap();
    }
    i2c_buf[0] = keypad_reg_cfg_addr;
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.read_after_write(keypad_addr, &mut i2c_buf[..1], 1);
        }
    );
    while bridge_int.is_high() {};
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            stat_msg = w.get_bus_status();
            w.read_buffer(&mut i2c_buf[..1]);
        }
    );
    if printf_dbg {
        writeln!(Stdout(&mut tx), "Bus status: {}", stat_msg).unwrap();
        write!(Stdout(&mut tx), "Buffer contents: [").unwrap();
        for byte in i2c_buf[..1].into_iter() {
            write!(Stdout(&mut tx), "{:08b},", byte).unwrap();
        }
        writeln!(Stdout(&mut tx), "]").unwrap();
    }

    keypad_reg_cfg_contents = 0x12;
    // Write cfg reg
    if printf_dbg {
        writeln!(Stdout(&mut tx), "\nWriting 0x{:x} to keypad config register (0x{:02x})", 
                keypad_reg_cfg_contents, 
                keypad_reg_cfg_addr)
        .unwrap();
    }
    i2c_buf[0] = keypad_reg_cfg_addr;
    i2c_buf[1] = keypad_reg_cfg_contents;
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.write_n_bytes(keypad_addr, &mut i2c_buf[..2]);
        }
    );
    while bridge_int.is_high() {};
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            stat_msg = w.get_bus_status();
        }
    );
    if printf_dbg {
        writeln!(Stdout(&mut tx), "Bus status: {}", stat_msg).unwrap();

        // Read cfg reg
        writeln!(Stdout(&mut tx), "\nReading keypad config register (0x{:02x})", keypad_reg_cfg_addr).unwrap();
    }
    i2c_buf[0] = keypad_reg_cfg_addr;
    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.read_after_write(keypad_addr, &mut i2c_buf[..1], 1);
            stat_msg = w.get_bus_status();
            w.read_buffer(&mut i2c_buf[..1]);
        }
    );
    if printf_dbg {
        writeln!(Stdout(&mut tx), "Bus status: {}", stat_msg).unwrap();
        write!(Stdout(&mut tx), "Buffer contents: [").unwrap();
        for byte in i2c_buf[..1].into_iter() {
            write!(Stdout(&mut tx), "{:08b},", byte).unwrap();
        }
        writeln!(Stdout(&mut tx), "]").unwrap();
    }


    // event fold
    (0..).fold(
        (qspi, (mosi, miso, sck)),
        |(qspi, pins), _| {
            let ret = bridge.session(qspi, pins,
                |_w| {
                }
            );
            delay_ms(1000);
            ret
        }
    );

    loop {};
}
