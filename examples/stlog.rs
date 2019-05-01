#![no_std]
#![no_main]
#![feature(never_type)]

extern crate hifive1;
extern crate panic_halt;
extern crate stlog;

use hifive1::hal::prelude::*;
use riscv_rt::entry;
use stlog::{error, global_logger, info, GlobalLog};

struct Logger;
impl GlobalLog for Logger {
    fn log(&self, addr: u8) {
        const UART0_ADDRESS: usize = 0x10013000;
        let dbg_txdata = UART0_ADDRESS as *mut i32;
        unsafe {
            while (*dbg_txdata) & (1 << 31) != 0 {}
            (*dbg_txdata) |= addr as i32;
        }
    }
}

#[global_logger]
static LOGGER: Logger = Logger;

#[entry]
unsafe fn main() -> ! {
    let p = e310x::Peripherals::take().unwrap();

    let clint = p.CLINT.split();
    let clocks = Clocks::freeze(p.PRCI.constrain(),
                                p.AONCLK.constrain());
    let mut gpio = p.GPIO0.split();
    let mut mux_sel = gpio.pin18.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    mux_sel.set_high();
    let (tx, rx) = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en,
    );
    let dbg_serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (tx, _) = dbg_serial.split();

    /*
    let hello_world = [72, 69, 76, 76, 79, 32, 87, 79, 82, 76, 68];
    let dbg_txdata = UART0_ADDRESS as *mut i32;
    for ch in hello_world.into_iter() {
        unsafe {
            while (*dbg_txdata) & (1 << 31) != 0 {}
            (*dbg_txdata) |= *ch;
        }
    }
    */

    info!("Hello!");
    error!("Bye!");
    loop {
        info!("yeeeeeeet!");
    }
}

