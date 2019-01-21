#![no_std]

extern crate hifive;

use hifive::hal::prelude::*;
use hifive::hal::e310x;
use hifive::hal::stdout::*;
//use hifive::hal::gpio::gpio0::{Pin2, Pin3, Pin4, Pin5, Pin9, Pin10, IOF_SEL, IOF_EN};
//use hifive::hal::gpio::{IOF0, NoInvert};

/*
extern {
    fn init_leds();
}

pub fn call() {
    unsafe {
        init_leds();
    }
}
*/

fn delay() {
    //block!(clint.wait()).unwrap();
    //clint.restart();
    for _i in 0..1000 {

    }
}

fn qspi1_xfer(qspi : &hifive::hal::e310x::QSPI1, payload: u32) -> u8 {
    loop {
        if qspi.txdata.read().full().bit_is_clear() { break; }
    }
    unsafe {
        qspi.txdata.write(|w| w.bits(payload));
    }
    loop {
        if qspi.rxdata.read().empty().bit_is_set() { break; }
    }
    return qspi.rxdata.read().data().bits();
}

fn max7221_init(qspi : &hifive::hal::e310x::QSPI1) {
    unsafe {
        qspi.csmode.write(|w| w.bits(2));
        qspi1_xfer(qspi, 0x09);
        qspi1_xfer(qspi, 0xFF);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi1_xfer(qspi, 0x0A);
        qspi1_xfer(qspi, 0x0F);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi1_xfer(qspi, 0x0B);
        qspi1_xfer(qspi, 0x03);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi1_xfer(qspi, 0x0C);
        qspi1_xfer(qspi, 0x01);
        qspi.csmode.write(|w| w.bits(3));

        qspi.csmode.write(|w| w.bits(2));
        qspi1_xfer(qspi, 0x0F);
        qspi1_xfer(qspi, 0x00);
        qspi.csmode.write(|w| w.bits(3));
    }
}

fn disp_val(qspi : &hifive::hal::e310x::QSPI1, val : u32) {
    let mut newval = val;
    for i in (0..5).rev() {
        let digval = newval % 10;
        unsafe {
            qspi.csmode.write(|w| w.bits(2));
            qspi1_xfer(qspi, i);
            qspi1_xfer(qspi, digval);
            qspi.csmode.write(|w| w.bits(3));
        }
        newval /= 10;
    }
}

fn main() {
    #[link(name="led", kind="static")]
    extern { fn init_leds(); }
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
    let desired_speed = 1_000_000;
    unsafe {
        qspi1.div.write(|w| w.bits(coreclk_speed / (2 * (desired_speed + 1))));
    }

    gpio.pin2.into_iof0(
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
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

    let mut dval = 1000;
    loop {
        disp_val(&qspi1, dval);
        dval += 1;
        delay();
        if dval == 1235 {
            break;
        }
    }

    unsafe { init_leds(); }
}
