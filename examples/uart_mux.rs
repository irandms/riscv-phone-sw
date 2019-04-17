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
    let mut mux_sel = gpio.pin18.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, _) = serial.split();
    mux_sel.set_low();
    writeln!(Stdout(&mut tx), "\nUART Mux Example\n").unwrap();
    mux_sel.set_high();
    writeln!(Stdout(&mut tx), "\nUART Mux Example\n").unwrap();

    let mut count = 0;
    loop {
        mux_sel.toggle();
        writeln!(Stdout(&mut tx), "Count: {}", count).unwrap();
        count += 1;
	delay_ms(500);
    }
}
