#![no_std]
#![no_main]

extern crate riscv;
extern crate hifive1;
extern crate panic_halt;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x::Peripherals;
use hifive1::hal::stdout::*;

#[entry]
fn main() -> ! {
    #[link(name="chello", kind="static")]
    extern {
        fn hello_from_C(); 
    }

    let p = Peripherals::take().unwrap();
    let _clint = p.CLINT.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());
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

    writeln!(Stdout(&mut tx), "Hello from Rust!").unwrap();

    unsafe {
        hello_from_C();
    }

    loop {};
}
