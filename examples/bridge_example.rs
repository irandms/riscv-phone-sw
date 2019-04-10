#![no_std]
#![no_main]
#![feature(asm)]

extern crate hifive1;
extern crate panic_halt;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::stdout::*;
use hifive1::hal::e310x;

// Horrible workaround for not moving to separate crates
#[path="../src/sc18is600.rs"]
mod sc18is600;
#[path="../src/qspi.rs"]
mod qspi;

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
    let p = e310x::Peripherals::take().unwrap();
    let clint = p.CLINT.split();
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
    let PRCI_ptr = e310x::PRCI::ptr();

    unsafe {
        let external_en = (*PRCI_ptr).hfxosccfg.read().enable().bit();
        let internal_en = (*PRCI_ptr).hfrosccfg.read().enable().bit();
        if external_en {
            writeln!(Stdout(&mut tx), "Ext clk enabled").unwrap();
        }
        if internal_en {
            writeln!(Stdout(&mut tx), "Int clk enabled").unwrap();
        }
    }

    let coreclk_hz = clocks.measure_coreclk(&clint.mcycle).0.hz();
    let qspi1 = p.QSPI1;
    let desired_sck = 1_000_000;

    unsafe {
        qspi::set_sck(&qspi1, desired_sck.hz(), clocks.coreclk());
        qspi1.mode.write(|w| w.bits(0b11));
        qspi1.csmode.write(|w| w.bits(0b00));
    }

    let div = qspi1.div.read().bits();
    let calculated_sck = 16_000_000 / (2*(div + 1));
    if calculated_sck != desired_sck {
        writeln!(Stdout(&mut tx), "Calculated SCK: {:6} Desired SCK: {:6}",
                 calculated_sck,
                 desired_sck)
            .unwrap();
    }
    writeln!(Stdout(&mut tx), "coreclk: {}", coreclk_hz.0).unwrap();

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

    let desired_scl = 100_000;
    writeln!(Stdout(&mut tx), "Attempting to write desired SCL").unwrap();
    sc18is600::write_clock(&qspi1, desired_scl);

    writeln!(Stdout(&mut tx), "Attempting to read SCL").unwrap();
    /*
    let read_clock = sc18is600::read_clock(&qspi1);
    let calculated_scl = 7_372_800 / (4 * read_clock as u32);
    */
    writeln!(Stdout(&mut tx), "After calculating SCL").unwrap();

    /*
    if calculated_scl != desired_scl {
        writeln!(Stdout(&mut tx), "Calculated SCL: {:6} Desired SCL: {:6}",
                 calculated_scl,
                 desired_scl)
            .unwrap();
    }

    loop {
        delay_ms(1);
        let read_clock = sc18is600::read_clock(&qspi1);
        writeln!(Stdout(&mut tx), "scl: {}", read_clock).unwrap();
    }
    */

    let i2c_payload: [u8; 5] = [1, 2, 3, 4, 5];
    //let clock_speeds: [u32; 5] = [7200, 97000, 204000, 263000, 369000];
    let clock_speeds: [u32; 1] = [7200];
    let mut clock_iter = clock_speeds.into_iter().cycle();

    loop {
        delay_ms(1);
        let current_speed = clock_iter.next();
        sc18is600::write_clock(&qspi1, *current_speed.unwrap());
        //sc18is600::write_n_bytes(&qspi1, 0x05, &i2c_payload);
    }

    /*
    loop {
        unsafe {
            qspi1.csid.write(|w| w.bits(0b10));
            qspi1.csmode.write(|w| w.bits(0b10));
            qspi1.mode.write(|w| w.bits(0b11));

            qspi::xfer(&qspi1, 0x20);
            qspi::xfer(&qspi1, 0x02);
            qspi::xfer(&qspi1, 0x19);

            qspi::xfer(&qspi1, 0x21);
            qspi::xfer(&qspi1, 0x02);
            let readval = qspi::xfer(&qspi1, 0x00);

            let calc_i2c_clk = 7_372_800 / (4 * readval);
            writeln!(Stdout(&mut tx), "Current Speed: {:6} Caclulated Speed: {:6}", 97000, calc_i2c_clk).unwrap();

            qspi1.csmode.write(|w| w.bits(0b00));
        }
    }
    */
}
