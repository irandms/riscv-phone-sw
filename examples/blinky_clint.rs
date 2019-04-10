#![no_std]
#![no_main]

extern crate riscv;
extern crate hifive1;
extern crate panic_halt;

use riscv_rt::entry;
use core::sync::atomic::{AtomicBool, Ordering};
use hifive1::hal::prelude::*;
use hifive1::hal::e310x;
use hifive1::hal::clint::{MTIME, MTIMECMP};
use hifive1::led;
use riscv::interrupt;
use riscv::register::mcause::{Trap, Interrupt};


static CLINT_TIMEOUT: AtomicBool = AtomicBool::new(false);
static mut MTIMECMP_G: *mut hifive1::hal::clint::MTIMECMP = core::ptr::null_mut();
static mut MTIME_G: *mut hifive1::hal::clint::MTIME = core::ptr::null_mut();

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 32768;
    mtimecmp.set_mtimecmp(next);
}

#[entry]
fn main() -> ! {
    let p = e310x::Peripherals::take().unwrap();
    let mut gpio = p.GPIO0.split();
    let mut clint = p.CLINT.split();
    let _clocks = Clocks::freeze(p.PRCI.constrain(), p.AONCLK.constrain());

    let (_red, mut green, mut blue) = led::rgb(
        gpio.pin22,
        gpio.pin19,
        gpio.pin21,
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    unsafe { 
        MTIME_G = &mut clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
    }

    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp); 
    clint.mtimer.enable();

    unsafe {
        interrupt::enable();
    }

    green.toggle();
    loop {
        if CLINT_TIMEOUT.load(Ordering::Relaxed) {
            CLINT_TIMEOUT.store(false, Ordering::Relaxed);
            blue.toggle();
            green.toggle();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler(trap: Trap) {
    match trap {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            CLINT_TIMEOUT.store(true, Ordering::Relaxed);
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        },
        _ => {}
    }
}
