#![no_std]

extern crate hifive;

use hifive::hal::prelude::*;
use hifive::hal::e310x;
use hifive::hal::stdout::*;

#[path="../src/spi.rs"]
mod spi;
use spi::qspi_xfer;


fn delay() {
    for _i in 0..1000 {

    }
}

fn max7221_init(qspi: &hifive::hal::e310x::QSPI1) {
    unsafe {
        qspi.csmode.write(|w| w.bits(2));
        qspi_xfer(qspi, 0x09);
        qspi_xfer(qspi, 0xFF);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi_xfer(qspi, 0x0A);
        qspi_xfer(qspi, 0x0F);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi_xfer(qspi, 0x0B);
        qspi_xfer(qspi, 0x03);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi_xfer(qspi, 0x0C);
        qspi_xfer(qspi, 0x01);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi_xfer(qspi, 0x0F);
        qspi_xfer(qspi, 0x00);
        qspi.csmode.write(|w| w.bits(3));
    }
}

fn disp_val(qspi : &hifive::hal::e310x::QSPI1, val : u32) {
    let mut newval = val;
    for i in (0..5).rev() {
        let digval = newval % 10;
        unsafe {
            qspi.csmode.write(|w| w.bits(2));
            qspi_xfer(qspi, i);
            qspi_xfer(qspi, digval);
            qspi.csmode.write(|w| w.bits(3));
        }
        newval /= 10;
    }
}

fn main() {
    let p = e310x::Peripherals::take().unwrap();

    let clint = p.CLINT.split();
    let clocks = Clocks::freeze(p.PRCI.constrain(),
        p.AONCLK.constrain(),
        &clint.mtime);
    let mut gpio = p.GPIO0.split();
    let (tx, rx) = hifive::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
        );
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, _) = serial.split();

    writeln!(Stdout(&mut tx), "hello world!").unwrap();

    let coreclk_speed = clocks.measure_coreclk(&clint.mtime, &clint.mcycle).0;
    writeln!(Stdout(&mut tx), "{}", coreclk_speed).unwrap();
    let qspi1 = p.QSPI1;
    let desired_speed = 4_000_000;
    unsafe {
        let div_val = coreclk_speed / (2 * desired_speed);
        qspi1.div.write(|w| w.bits(div_val));
    }

    gpio.pin2.into_iof0(
        & mut gpio.out_xor,
        & mut gpio.iof_sel,
        & mut gpio.iof_en
    );
    gpio.pin3.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    gpio.pin4.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    gpio.pin5.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    gpio.pin9.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    gpio.pin10.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );

    let div_val = qspi1.div.read().bits();
    writeln!(Stdout(&mut tx), "After setting sckdiv to {}!", div_val).unwrap();

    max7221_init(&qspi1);
    writeln!(Stdout(&mut tx), "After max7221_init").unwrap();

    disp_val(&qspi1, 5000);
    let mut dval = 0;
    loop {
        disp_val(&qspi1, dval);
        dval += 1;
        delay();
    }
}
