#![no_std]


extern crate riscv;
extern crate hifive;

use core::sync::atomic::{AtomicBool, Ordering};
use hifive::hal::prelude::*;
use hifive::hal::e310x;
use hifive::hal::clint::{MTIME, MTIMECMP};
use hifive::hal::stdout::*;
use riscv::interrupt;
use riscv::register::mcause::{Trap, Interrupt};
use riscv::register::mstatus;
use riscv::register::mie;
use riscv::register::mip;
use riscv::register::mcause;
use hifive::hal::plic::CLAIM;

use riscv::interrupt::Nr;

static CLINT_TIMEOUT: AtomicBool = AtomicBool::new(false);
static mut MTIMECMP_G: *mut hifive::hal::clint::MTIMECMP = core::ptr::null_mut();
static mut MTIME_G: *mut hifive::hal::clint::MTIME = core::ptr::null_mut();
static mut CLAIM_G: *mut hifive::hal::plic::CLAIM = core::ptr::null_mut();
static mut TX: *mut hifive::hal::prelude::Tx<hifive::hal::e310x::UART0> = core::ptr::null_mut();

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 32768;
    mtimecmp.set_mtimecmp(next);
}

fn main() {
    let p = e310x::Peripherals::take().unwrap();
    let mut gpio = p.GPIO0.split();
    let mut clint = p.CLINT.split();
    let mut plic = p.PLIC.split();
    let clocks = Clocks::freeze(p.PRCI.constrain(),
                                p.AONCLK.constrain(),
                                &clint.mtime);

    let (_red, _green, mut blue) = hifive::rgb(
        gpio.pin22,
        gpio.pin19,
        gpio.pin21,
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );
    let (tx, rx) = hifive::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, _) = serial.split();
    //let test: i32 = tx;

    unsafe { 
        TX = &mut tx; 
        MTIME_G = &mut clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
        CLAIM_G = &mut plic.claim;
    }

    writeln!(Stdout(&mut tx), "hello world!").unwrap();
    unsafe {
        writeln!(Stdout(&mut *TX), "hello GLOBE").unwrap();
    }

    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp);

    clint.mtimer.enable();

    unsafe {
        //mstatus::set_mie();
        //mie::set_mtimer();
        interrupt::enable();
    }

    loop {
        //writeln!(Stdout(&mut tx), "enter loop").unwrap();
        if CLINT_TIMEOUT.load(Ordering::Relaxed) {
            CLINT_TIMEOUT.store(false, Ordering::Relaxed);
            writeln!(Stdout(&mut tx), "BEGIN TOGGLE").unwrap();
            blue.toggle();
            writeln!(Stdout(&mut tx), "Toggled LED").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler(trap: Trap) {
    let cause = mcause::read().bits();
    writeln!(Stdout(&mut *TX), "Contents of mcause: {:b}", cause).unwrap();
    let contents = mip::read().bits();
    writeln!(Stdout(&mut *TX), "in trap, mip: {:b}", contents).unwrap();
    match trap {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            let contents = mip::read().bits();
            writeln!(Stdout(&mut *TX), "in MachineTimer, mip: {:b}", contents).unwrap();
            writeln!(Stdout(&mut *TX), "8 {:b}", 8).unwrap();
            CLINT_TIMEOUT.store(true, Ordering::Relaxed);
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        },
        _ => {
            //let inter = (*CLAIM_G).claim().unwrap().nr();
            writeln!(Stdout(&mut *TX), "In another interrupt").unwrap();
            //writeln!(Stdout(&mut *TX), "Claimed int: {}", inter).unwrap();
        }
    }
}
