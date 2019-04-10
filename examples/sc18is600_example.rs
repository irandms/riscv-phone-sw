#![no_std]
#![no_main]
#![feature(asm)]

extern crate hifive1;
extern crate panic_halt;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x::Peripherals;
use hifive1::hal::stdout::*;
use hifive1::hal::spi::{Spi, Mode, Polarity, Phase};

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
    writeln!(Stdout(&mut tx), "SC18IS600 Example").unwrap();

    // Configure SPI pins
    let mosi = gpio.pin3.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let miso = gpio.pin4.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let sck = gpio.pin5.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let mut cs = gpio.pin10.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);

    // Configure SPI
    let pins = (mosi, miso, sck);
    let mode3 = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };
    let spi_ptr = hifive1::hal::e310x::QSPI1::ptr();
    let mut spi = Spi::spi1(p.QSPI1, pins, mode3, 1_000_000.hz(), clocks);

    unsafe { (*spi_ptr).delay0.write(|w| w.cssck().bits(0b1).sckcs().bits(0b1)); };
    unsafe { (*spi_ptr).delay1.write(|w| w.intercs().bits(8).interxfr().bits(8)); };

    let mut reg_idx = 5;

    let mut buf = [0x21, reg_idx, 0xFF];
    cs.set_low();
    let r2 = spi.transfer(&mut buf);
    cs.set_high();
    writeln!(Stdout(&mut tx), "Register 0x{:x}: {:?}", reg_idx, r2.unwrap().last()).unwrap();

    let mut buf = [0x20, reg_idx, 0x34];
    writeln!(Stdout(&mut tx), "Writing {:?} to Register 0x{:x}", buf[2], reg_idx).unwrap();
    cs.set_low();
    spi.transfer(&mut buf).unwrap();
    cs.set_high();

    let mut buf = [0x21, reg_idx, 0xFF];
    cs.set_low();
    let r2 = spi.transfer(&mut buf);
    cs.set_high();
    writeln!(Stdout(&mut tx), "Register 0x{:x}: {:?}", reg_idx, r2.unwrap().last()).unwrap();

    loop {
        /*
        let mut buf = [0x20, 0x02, 19];
        cs.set_low();
        let r1 = spi.transfer(&mut buf);
        cs.set_high();

        let mut buf = [0x21, 0x02, 0xFF];
        cs.set_low();
        let r2 = spi.transfer(&mut buf);
        cs.set_high();
        */

        let mut buf = [0x21, reg_idx, 0xFF];
        cs.set_low();
        let r2 = spi.transfer(&mut buf);
        cs.set_high();

        //writeln!(Stdout(&mut tx), "r1: {:?}", r1.unwrap()).unwrap();
        writeln!(Stdout(&mut tx), "Register 0x{:x}: {:?}", reg_idx, r2.unwrap().last()).unwrap();
        delay_ms(1000);
        reg_idx += 1;
        if reg_idx > 5 {
            reg_idx = 0;
        }
    };
}
