#![no_std]

extern crate riscv;
extern crate hifive;
extern crate atomiqueue;

use atomiqueue::AtomiQueue;
use riscv::interrupt;
use hifive::hal::e310x;
use hifive::hal::stdout::*;
use hifive::hal::prelude::*;
use hifive::hal::clint::{MTIME, MTIMECMP};
use riscv::register::mcause::{Trap, Interrupt};

static mut MTIMECMP_G: *mut hifive::hal::clint::MTIMECMP = core::ptr::null_mut();
static mut MTIME_G: *mut hifive::hal::clint::MTIME = core::ptr::null_mut();
static mut MSG_QUEUE: *mut atomiqueue::AtomiQueue<u64> = core::ptr::null_mut();

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 65535;
    mtimecmp.set_mtimecmp(next);
}

fn main() {
    let mut queue = AtomiQueue::<u64>::new();
    let p = e310x::Peripherals::take().unwrap();

    let mut clint = p.CLINT.split();
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

    unsafe {
        MTIME_G = &mut clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
        MSG_QUEUE = &mut queue;
    }

    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp);

    clint.mtimer.enable();

    unsafe {
        interrupt::enable();
    }

    let button_pin = gpio.pin13.into_pull_up_input(
        &mut gpio.pullup,
        &mut gpio.input_en,
        &mut gpio.iof_en
    );

    loop {
        if button_pin.is_low() {
            let action = match queue.pop() {
                Ok(a) => {
                    Some(a);
                }
                Err(_e) => {
                    ();
                }
            };

            writeln!(Stdout(&mut tx), "Action: {:?}", action).unwrap();
        }
    }
}

#[no_mangle]
fn trap_handler(trap: Trap) {
    match trap {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            unsafe {
                (*MSG_QUEUE).push((*MTIME_G).mtime()).unwrap();
                set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
            }
        },
        _ => {},
    }
}
