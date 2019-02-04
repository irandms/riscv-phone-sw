#![no_std]

extern crate riscv;
extern crate hifive;
extern crate atomiqueue;

use atomiqueue::AtomiQueue;
use core::sync::atomic::{AtomicBool, Ordering};

use riscv::interrupt;
use hifive::hal::e310x;
use hifive::hal::stdout::*;
use hifive::hal::prelude::*;
use hifive::hal::clint::{MTIME, MTIMECMP};
use riscv::register::mcause::{Trap, Interrupt};
use riscv::interrupt::Nr;

static PUSHED: AtomicBool = AtomicBool::new(false);
static mut MTIMECMP_G: *mut hifive::hal::clint::MTIMECMP = core::ptr::null_mut();
static mut MTIME_G: *mut hifive::hal::clint::MTIME = core::ptr::null_mut();
static mut MSG_QUEUE: *mut atomiqueue::AtomiQueue<u64> = core::ptr::null_mut();
static mut CLAIM_G: *mut hifive::hal::plic::CLAIM = core::ptr::null_mut();
static mut TX: *mut hifive::hal::prelude::Tx<hifive::hal::e310x::UART0> = core::ptr::null_mut();

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 32768;
    mtimecmp.set_mtimecmp(next);
}

fn main() {
    let mut queue = AtomiQueue::<u64>::new();
    let p = e310x::Peripherals::take().unwrap();
    let mut clint = p.CLINT.split();
    let mut plic = p.PLIC.split();
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
    writeln!(Stdout(&mut tx), "AtomiQueue Example").unwrap();

    unsafe {
        TX = &mut tx;
        CLAIM_G = &mut plic.claim;
        MTIME_G = &mut clint.mtime;
        MTIMECMP_G = &mut clint.mtimecmp;
        MSG_QUEUE = &mut queue;
    }

    let button_pin = gpio.pin13.into_pull_up_input(
        &mut gpio.pullup,
        &mut gpio.input_en,
        &mut gpio.iof_en
    );

    set_mtimecmp(&clint.mtime, &mut clint.mtimecmp);
    clint.mtimer.enable();
    writeln!(Stdout(&mut tx), "PRE").unwrap();
    unsafe {
        interrupt::enable();
    }
    writeln!(Stdout(&mut tx), "POST").unwrap();

    loop {
        if button_pin.is_low() {
            if let Ok(Some(timestamp)) = queue.pop() {
                writeln!(Stdout(&mut tx), "Time: {:?} has been popped!", timestamp).unwrap();
            };
        }

        if PUSHED.load(Ordering::Relaxed) {
            PUSHED.store(false, Ordering::Relaxed);
            writeln!(Stdout(&mut tx), "Timestamp has been pushed!").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler(trap: Trap) {
    match trap {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            (*MSG_QUEUE).push((*MTIME_G).mtime()).unwrap();
            PUSHED.store(true, Ordering::Relaxed);
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        },
        _ => {
            //writeln!(Stdout(&mut *TX), "In ISR: {}", (*CLAIM_G).claim().unwrap().nr()).unwrap();
        },
    }
}
