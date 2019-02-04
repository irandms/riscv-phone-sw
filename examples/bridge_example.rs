#![no_std]

extern crate hifive;

use hifive::hal::prelude::*;
use hifive::hal::e310x;
use hifive::hal::stdout::*;

// Horrible workaround for not moving to separate crates
#[path="../src/sc18is600.rs"]
mod sc18is600;
#[path="../src/spi.rs"]
mod spi;

fn delay() {
    for _i in 0..1000 {

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

    let coreclk_speed = clocks.measure_coreclk(&clint.mtime, &clint.mcycle).0;
    let qspi1 = p.QSPI1;
    let desired_speed = 1_200_000;

    unsafe {
        let div_val = (coreclk_speed / (2 * desired_speed)) - 1;
        qspi1.div.write(|w| w.bits(div_val));
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

    let read_clock = sc18is600::read_clock(&qspi1);
    writeln!(Stdout(&mut tx), "I2CClk: {}", read_clock).unwrap();
    sc18is600::write_clock(&qspi1, 255);
    let read_clock = sc18is600::read_clock(&qspi1);
    writeln!(Stdout(&mut tx), "I2CClk: {}", read_clock).unwrap();

    let i2c_payload: [u8; 5] = [1, 2, 3, 4, 5];
    let clock_speeds: [u32; 5] = [7200, 97000, 204000, 263000, 369000];
    let mut clock_iter = clock_speeds.into_iter().cycle();

    loop {
        delay();

        let current_speed = clock_iter.next();
        sc18is600::write_clock(&qspi1, *current_speed.unwrap());
        let read_clock = sc18is600::read_clock(&qspi1);
        writeln!(Stdout(&mut tx), "I2CClk: {}", read_clock).unwrap();

        sc18is600::write_n_bytes(&qspi1, 0x05, &i2c_payload);
    }
}
