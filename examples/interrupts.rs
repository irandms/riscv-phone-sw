#![no_std]
#![no_main]
#![feature(asm, fn_traits)]
#![allow(unreachable_code)] 
#![feature(type_ascription)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;
extern crate stlog;

use stlog::{error, global_logger, info, GlobalLog};
use riscv::register::{
    mcause,
    mcause::{
        Interrupt,
        Exception,
        Trap, 
    },
    mstatus,
    mie,
};
use riscv_rt::entry;
use hifive1::hal::{
    prelude::*,
    e310x::{
        Peripherals,
        PLIC,
        Interrupt as e310x_Interrupt,
    },
    clint::{
        MTIME,
        MTIMECMP,
    },
    stdout::*,
};

#[global_logger]
static LOGGER: Logger = Logger;

static mut MTIMECMP_G: *mut MTIMECMP = core::ptr::null_mut(); 
static mut MTIME_G: *const MTIME = core::ptr::null();
static mut DBG_TX: *mut hifive1::hal::prelude::Tx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut DBG_RX: *mut hifive1::hal::prelude::Rx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut CLAIM: *mut hifive1::hal::plic::CLAIM = core::ptr::null_mut();

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


fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 32768;
    mtimecmp.set_mtimecmp(next);
}

fn init_peripherals() -> (hifive1::hal::gpio::gpio0::Pin0<hifive1::hal::gpio::Input<hifive1::hal::gpio::Floating>>, hifive1::hal::prelude::Tx<hifive1::hal::e310x::UART0>) {
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
    let pushbtn = gpio.pin0.into_floating_input(&mut gpio.pullup, &mut gpio.input_en, &mut gpio.iof_en);
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks).listen();
    let (mut tx, mut rx) = serial.split();
    plic.mext.enable(); // MEIE bit in MIE register
    plic.uart0.enable();
    clint.mtimer.enable();
    unsafe {
        (*PLIC::ptr()).enable[0].modify(|r, w| w.bits(r.bits() | (1 << 8)));
        (*PLIC::ptr()).enable[0].modify(|r, w| w.bits(r.bits() | (1 << 3)));
        (*PLIC::ptr()).priority[8].modify(|r, w| w.bits(r.bits() | 3));
        (*PLIC::ptr()).priority[7].modify(|r, w| w.bits(r.bits() | 3));
        (*PLIC::ptr()).priority[3].modify(|r, w| w.bits(r.bits() | 3));
    }

    unsafe {
        DBG_TX = &mut tx;
        DBG_RX = &mut rx;
        MTIME_G = &clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
        CLAIM = &mut plic.claim;
        riscv::interrupt::enable(); // MIE bit in MSTATUS register, MSIE in MIE
    }
    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp);

    (pushbtn, tx)
}

#[entry]
fn main() -> ! {
    let (pushbtn, mut tx) = init_peripherals();

    info!("UART RX + pushbutton interrupt example");
    if cfg!(debug_assertions) {
        info!("Debug enabled");
    }

    loop {
        if cfg!(debug_assertions) {
            if pushbtn.is_low() {
                //info!("l");
            }
            else {
                //info!("h");
            }
        }
    }
}

#[no_mangle]
unsafe fn handle_interrupt(intr: Interrupt) {
    let claim = (*CLAIM).claim(); match claim {
        Some(_cause) => { }
        None => {
            error!("Interrupt Claim was empty");
        }
    }
    match intr {
        Interrupt::MachineTimer => {
            info!("Machine Timer interrupt");
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        }
        Interrupt::MachineSoft => {
            info!("Machine Soft interrupt");
        }
        Interrupt::MachineExternal => {
            info!("Machine Ext interrupt");
        }
        _ => {
            info!("Machine ??? interrupt");
        }
    }
    (*CLAIM).complete(claim.unwrap());
}

unsafe fn handle_exception(excpt: Exception) {
    match excpt {
        Exception::Breakpoint => {
            info!("Breakpoint");
        }
        _ => {
            info!("Unhandled Exception detected");
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
        error!("No interrupt or debug in trap_handler");
    }
}
