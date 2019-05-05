#![no_std]
#![no_main]
#![feature(asm, fn_traits, never_type)]
#![allow(unreachable_code)] 
#![feature(type_ascription)]


extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;

use riscv::register::{
    mcause,
    mcause::{
        Interrupt,
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
    plic::{
        Priority,
        CLAIM,
    },
    gpio::{
        Input,
        PullUp,
        Output,
        Regular,
        NoInvert,
        gpio0::{
            Pin16,
            Pin17,
            Pin18,
        },
    },
    stdout::*,
};
use atomiqueue::AtomiQueue;
use embedded_hal::{
    serial,
    digital::OutputPin,
};

#[no_mangle]
fn delay_cycles(cyc: u32) {
    let clint_ptr = e310x::CLINT::ptr();
    unsafe {
        for _ in 0..cyc {
           asm!("NOP" :::: "volatile");
        }
    }
}

use embedded_hal::serial::Write;
use nb::Result;

struct SoftwareSerial<'pin, Pin: OutputPin>(pub &'pin mut Pin);

impl<'pin, Pin: OutputPin> Write<u8> for SoftwareSerial<'pin, Pin> {
    type Error = !;

    fn flush(&mut self) -> Result<(), Self::Error> { Ok(()) }

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        for bit in 0..8 {
            if (byte.wrapping_shr(bit) & 0b1) == 0b1 {
                self.0.set_high();
            } else {
                self.0.set_low();
            }

            delay_cycles(139);
        }

        Ok(())
    }
}

static mut CLAIM: *mut CLAIM = core::ptr::null_mut();
static RX_BUF: AtomiQueue<u8> = AtomiQueue::new();

fn init_peripherals() -> (Pin17<Output<Regular<NoInvert>>>,
                          Pin16<Input<PullUp>>,
                          Pin18<Output<Regular<NoInvert>>>) {
    let p = Peripherals::take().unwrap();
    let mut clint = p.CLINT.split();
    let mut plic = p.PLIC.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 16.mhz().into());

    p.GPIO0.rise_ie.write(|w| w.pin0().bit(true));
    let mut gpio = p.GPIO0.split();
    let mut sw_tx   = gpio.pin17.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let sw_rx       = gpio.pin16.into_pull_up_input(&mut gpio.pullup, &mut gpio.input_en, &mut gpio.iof_en);
    let mut mux_sel = gpio.pin18.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    mux_sel.set_high();
    plic.mext.enable(); // MEIE bit in MIE register
    plic.uart0.enable(); // Enable the UART0 receive interrupt
    plic.threshold.set(Priority::P0); // Listen to any interrupt with priority > 0
    clint.mtimer.disable(); // Disable timer interrupts

    unsafe {
        CLAIM = &mut plic.claim;
        (*PLIC::ptr()).enable[0].modify(|r, w| w.bits(r.bits() | (1 << 3)));
        (*PLIC::ptr()).priority[3].modify(|r, w| w.bits(r.bits() | 3));
        (*UART0::ptr()).ie.write(|w| w.txwm().bit(false).rxwm().bit(true));

        riscv::interrupt::enable(); // MIE bit in MSTATUS register, MSIE in MIE
    };

    (sw_tx, sw_rx, mux_sel)
}

#[entry]
fn main() -> ! {
    let (mut sw_tx, sw_rx, mut mux_sel) = init_peripherals();

    writeln!(Stdout(&mut sw_tx), "UART REPL").unwrap();
    if cfg!(debug_assertions) {
        writeln!(Stdout(&mut sw_tx), "Debug enabled").unwrap();
    }

    write!(Stdout(&mut sw_tx), "\n>").unwrap(); // prompt
    loop {
        if let Ok(Some(ch)) = RX_BUF.back() {
            if ch == '\r' as u8 {
                mux_sel.toggle();
                // Display the received string upon CR
                while let Ok(Some(print_ch)) = RX_BUF.pop() {
                    write!(Stdout(&mut sw_tx), "{}", print_ch as char).unwrap();
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
            match RX_BUF.push(read_char) {
                Ok(_) => {}
                Err(_) => {} // TODO: Do something in Err
            }
            while (*UART0::ptr()).txdata.read().bits() != 0 {};
            (*UART0::ptr()).txdata.write(|w| w.data().bits(read_char));
        }
        _ => {
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
                }
            }
            handle_mext_interrupt(claim.unwrap());
            (*CLAIM).complete(claim.unwrap());
        }
        _ => {
        }
    }
}

#[no_mangle]
unsafe fn trap_handler() {
    let trap = mcause::read().cause();
    if let Trap::Interrupt(intr) = trap {
        handle_interrupt(intr);
    } 
    else if let Trap::Exception(_excpt) = trap {
        panic!("An exception occured");
    }
    else if cfg!(debug_assertions) {
        panic!("No interrupt or exception detected");
    }
}
