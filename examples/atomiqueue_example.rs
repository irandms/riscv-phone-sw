#![no_std]
#![no_main]

extern crate riscv;
extern crate hifive1;
extern crate atomiqueue;
extern crate panic_halt;

use riscv_rt::entry;
use core::{
    sync::atomic::{AtomicBool, Ordering},
    ptr::null_mut,
};
use atomiqueue::AtomiQueue;
use hifive1::hal::{
    e310x,
    stdout::*,
    prelude::*,
    clint::{MTIME, MTIMECMP},
};
use riscv::{
    interrupt,
    register::mcause::{Trap, Interrupt},
};

static PUSHED: AtomicBool      = AtomicBool::new(false);
static QUEUE:  AtomiQueue<u64> = AtomiQueue::new();

static mut MTIMECMP_G: *mut MTIMECMP         = null_mut();
static mut MTIME_G:    *mut MTIME            = null_mut();

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    mtimecmp.set_mtimecmp(mtime.mtime() + 0x8000);
}

#[entry]
fn main() -> ! {
    let p = e310x::Peripherals::take().unwrap();
    let mut clint = p.CLINT.split();
    let clocks = Clocks::freeze(
        p.PRCI.constrain(),
        p.AONCLK.constrain());
    let mut gpio = p.GPIO0.split();

    let txrx = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );

    let (mut tx, _) = Serial::uart0(
        p.UART0,
        txrx,
        115_200.bps(),
        clocks,
    ).split();

    let mut stdout = Stdout(&mut tx);
    writeln!(stdout, "AtomiQueue Example").unwrap();

    unsafe {
        MTIME_G    = &mut clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
    }

    let button_pin = gpio.pin13.into_pull_up_input(
        &mut gpio.pullup,
        &mut gpio.input_en,
        &mut gpio.iof_en
    );

    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp);
    clint.mtimer.enable();

    writeln!(stdout, "PRE").unwrap();
    unsafe { interrupt::enable(); }
    writeln!(stdout, "POST").unwrap();

    loop {
        if button_pin.is_low() {
            if let Ok(Some(timestamp)) = QUEUE.pop() {
                writeln!(stdout, "Time: {:?} has been popped!", timestamp).unwrap();
            };
        }

        if PUSHED.compare_and_swap(true, false, Ordering::Relaxed) {
            writeln!(stdout, "Timestamp has been pushed!").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler(trap: Trap) {
    match trap {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            QUEUE.push((*MTIME_G).mtime()).unwrap();
            PUSHED.store(true, Ordering::Relaxed);
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        },
        _ => {
        },
    }
}
