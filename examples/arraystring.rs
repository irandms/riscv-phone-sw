#![no_std]
#![no_main]
#![feature(asm, fn_traits)]
#![allow(unreachable_code)] 
#![feature(type_ascription)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;
extern crate stlog;

use core::ptr;
use stlog::{global_logger, info, GlobalLog};
use riscv::register::{
    mcause,
    mcause::{
        Interrupt,
        Exception,
        Trap, 
    },
};
use riscv_rt::entry;
use hifive1::hal::{
    prelude::*,
    e310x::{
        Peripherals,
        UART0,
        PLIC,
        Interrupt as e310x_Interrupt,
    },
    plic::Priority,
    stdout::*,
};
use arrayvec::ArrayString;

#[global_logger]
static LOGGER: Logger = Logger;

static mut DBG_TX: *mut hifive1::hal::prelude::Tx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut DBG_RX: *mut hifive1::hal::prelude::Rx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut CLAIM: *mut hifive1::hal::plic::CLAIM = core::ptr::null_mut();
static mut RXBUF: Option<ArrayString<[u8; 32]>> = None;

struct Logger;
impl GlobalLog for Logger {
    fn log(&self, addr: u8) {
        const UART0_ADDRESS: usize = 0x10013000;
        let dbg_txdata = UART0_ADDRESS as *mut u32;
        unsafe {
            while ptr::read_volatile(dbg_txdata) != 0 {};
            ptr::write_volatile(dbg_txdata, addr as u32);
        }
    }
}

fn init_peripherals() -> hifive1::hal::prelude::Tx<hifive1::hal::e310x::UART0> {
    unsafe {
        RXBUF = Some(ArrayString::<[_; 32]>::new());
        RXBUF.unwrap().push('a');
        RXBUF.unwrap().push('b');
        RXBUF.unwrap().push('c');
    }
    let p = Peripherals::take().unwrap();
    let mut clint = p.CLINT.split();
    let mut plic = p.PLIC.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 16.mhz().into());

    p.GPIO0.rise_ie.write(|w| w.pin0().bit(true));
    let mut gpio = p.GPIO0.split();
    let (tx, rx) = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks).listen();
    let (mut tx, mut rx) = serial.split();
    plic.mext.enable(); // MEIE bit in MIE register
    plic.uart0.enable(); // Enable the UART0 receive interrupt
    plic.threshold.set(Priority::P0); // Listen to any interrupt with priority > 0
    clint.mtimer.disable(); // Disable timer interrupts

    unsafe {
        DBG_TX = &mut tx;
        DBG_RX = &mut rx;
        CLAIM = &mut plic.claim;
        (*PLIC::ptr()).enable[0].modify(|r, w| w.bits(r.bits() | (1 << 3)));
        (*PLIC::ptr()).priority[3].modify(|r, w| w.bits(r.bits() | 3));

        riscv::interrupt::enable(); // MIE bit in MSTATUS register, MSIE in MIE
    };

    tx
}

#[entry]
fn main() -> ! {
    let mut tx = init_peripherals();

    writeln!(Stdout(&mut tx), "UART RX Echo").unwrap();
    if cfg!(debug_assertions) {
        writeln!(Stdout(&mut tx), "Debug enabled").unwrap();
    }
    loop {
        unsafe {
            // TODO: Why does the successful pop() case never run?
            match RXBUF.unwrap().pop() {
                Some(ch) => {
                    writeln!(Stdout(&mut tx), "{}", ch).unwrap();
                }
                None => {
                }
            }
        }
    }
}

#[no_mangle]
unsafe fn handle_mext_interrupt(intr: e310x_Interrupt) {
    match intr {
        e310x_Interrupt::UART0 => {
            let read_char = (*UART0::ptr()).rxdata.read().data().bits();
            RXBUF.unwrap().push(read_char as char); // TODO: Does this work?
            //while (*UART0::ptr()).txdata.read().bits() != 0 {};
            //(*UART0::ptr()).txdata.write(|w| w.data().bits(read_char));
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "other mext int").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn handle_interrupt(intr: Interrupt) {
    match intr {
        Interrupt::MachineExternal => {
            let claim = (*CLAIM).claim(); match claim {
                Some(_cause) => { }
                None => {
                    writeln!(Stdout(&mut *DBG_TX), "claim empty").unwrap();
                }
            }
            handle_mext_interrupt(claim.unwrap());
            (*CLAIM).complete(claim.unwrap());
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "machine ??? int").unwrap();
        }
    }
}

unsafe fn handle_exception(excpt: Exception) {
    match excpt {
        Exception::Breakpoint => {
            info!("Breakpoint");
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "Unhandled exception").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler() {
    let trap = mcause::read().cause();
    if let Trap::Interrupt(intr) = trap {
        handle_interrupt(intr);
    } 
    else if let Trap::Exception(excpt) = trap {
        handle_exception(excpt);
    }
    else if cfg!(debug_assertions) {
        writeln!(Stdout(&mut *DBG_TX), "No int or excpt in trap_handler").unwrap();
    }
}
