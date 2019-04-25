#![no_std]
#![no_main]
#![feature(asm, fn_traits)]
#![allow(unreachable_code)] 
#![feature(type_ascription)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;

use riscv::register::{
    mcause,
    mcause::{
        Interrupt,
        Exception,
        Trap, 
    },
    mstatus,
    mie,
    mip,
};
use riscv::interrupt::Nr;
use riscv_rt::entry;
use hifive1::hal::{
    prelude::*,
    e310x::{
        Peripherals,
        gpio0,
    },
    clint::{
        MTIME,
        MTIMECMP,
    },
    plic::Priority,
    stdout::*,
    gpio::{
        Output,
        Regular,
        NoInvert
    }
};

static mut MTIMECMP_G: *mut MTIMECMP = core::ptr::null_mut(); 
static mut MTIME_G: *const MTIME = core::ptr::null();
static mut MUX_SEL: *mut hifive1::hal::gpio::gpio0::Pin18<Output<Regular<NoInvert>>> = core::ptr::null_mut();
static mut DBG_TX: *mut hifive1::hal::prelude::Tx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut DBG_RX: *mut hifive1::hal::prelude::Rx<hifive1::hal::e310x::UART0> = core::ptr::null_mut(); 
static mut CLAIM: *mut hifive1::hal::plic::CLAIM = core::ptr::null_mut();
static mut BUF: [char;64] = ['\0';64]; 
static mut BUF_POS: usize = 0;

fn set_mtimecmp(mtime: &MTIME, mtimecmp: &mut MTIMECMP) {
    let next = mtime.mtime() + 65536;
    mtimecmp.set_mtimecmp(next);
}

fn delay_ms(mtime: &MTIME, ms: u32) {
    let goal = mtime.mtime() + 32 * ms as u64;
    while mtime.mtime() < goal {
    }
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let fallie;
    unsafe {
        p.GPIO0.fall_ie.write(|w| w.bits(1));
    }
    let mut clint = p.CLINT.split();
    let mut plic = p.PLIC.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 16.mhz().into());

    let mut gpio = p.GPIO0.split();
    let (tx, rx) = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    let mut mux_sel = gpio.pin18.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let _pushbtn = gpio.pin0.into_floating_input(&mut gpio.pullup, &mut gpio.input_en, &mut gpio.iof_en);
    let mut rts = gpio.pin1.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let mut dtr = gpio.pin12.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks).listen();
    let (mut tx, mut rx) = serial.split();
    mux_sel.set_high();
    plic.mext.enable(); // MEIE bit in MIE register
    plic.uart0.enable();
    clint.mtimer.disable();

    unsafe {
        DBG_TX = &mut tx;
        DBG_RX = &mut rx;
        MTIME_G = &clint.mtime;
        CLAIM = &mut plic.claim;
        riscv::interrupt::enable(); // MIE bit in MSTATUS register, MSIE in MIE
        let mie_bits = mie::read().bits();
        let mstatus = mstatus::read();
        writeln!(Stdout(&mut tx), "mie: {:b}\n", mie_bits).unwrap();
        writeln!(Stdout(&mut tx), "MIE: {} UIE: {}\n", mstatus.mie(), mstatus.uie()).unwrap();
    }

    writeln!(Stdout(&mut tx), "\nUART Mux Example Console\n").unwrap();
    delay_ms(&clint.mtime,1);
    mux_sel.set_low();
    delay_ms(&clint.mtime,1);

        rts.set_high();
        dtr.set_high();
        delay_ms(&clint.mtime,20);
        writeln!(Stdout(&mut tx), "AT&K0").unwrap();
        rts.set_low();
        dtr.set_low();
        delay_ms(&clint.mtime,900);

        rts.set_high();
        dtr.set_high();
        delay_ms(&clint.mtime,20);
        writeln!(Stdout(&mut tx), "AT\\Q0").unwrap();
        rts.set_low();
        dtr.set_low();
        delay_ms(&clint.mtime,900);

    loop {
        rts.set_high();
        dtr.set_high();
        delay_ms(&clint.mtime,20);
        writeln!(Stdout(&mut tx), "AT+IFC=0,0").unwrap();
        rts.set_low();
        dtr.set_low();
        delay_ms(&clint.mtime,900);

        rts.set_high();
        dtr.set_high();
        delay_ms(&clint.mtime,20);
        writeln!(Stdout(&mut tx), "AT+IFC").unwrap();
        rts.set_low();
        dtr.set_low();
        delay_ms(&clint.mtime,900);
    }

        delay_ms(&clint.mtime,21);
        writeln!(Stdout(&mut tx), "AT&W0").unwrap();
        delay_ms(&clint.mtime,900);

        delay_ms(&clint.mtime,21);
        writeln!(Stdout(&mut tx), "AT+IFC=0,0").unwrap();
        delay_ms(&clint.mtime,900);

    loop {
        writeln!(Stdout(&mut tx), "AT").unwrap();
        delay_ms(&clint.mtime,1000);
    }
}

#[no_mangle]
unsafe fn handle_interrupt(intr: Interrupt) {
    let claim = (*CLAIM).claim();
    match claim {
        Some(cause) => {
            writeln!(Stdout(&mut *DBG_TX), "handling int caused by {:?}", cause).unwrap();
        }
        None => {
            writeln!(Stdout(&mut *DBG_TX), "no claim").unwrap();
        }
    }
    match intr {
        Interrupt::MachineTimer => {
            writeln!(Stdout(&mut *DBG_TX), "in handle_interrupt handling tmr interrupt {:?}", intr).unwrap();
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        }
        Interrupt::MachineSoft => {
            writeln!(Stdout(&mut *DBG_TX), "in handle_interrupt sft interrupt {:?}", intr).unwrap();
        }
        Interrupt::MachineExternal => {
            writeln!(Stdout(&mut *DBG_TX), "in handle_interrupt handling ext interrupt {:?}", intr).unwrap();
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "in handle_interrupt handling UNKNOWN interrupt {:?}", intr).unwrap();
        }
    }
}

unsafe fn handle_exception(excpt: Exception) {
    match excpt {
        Exception::Breakpoint => {
            writeln!(Stdout(&mut *DBG_TX), "BRKPT").unwrap();
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "Unhandled Exception").unwrap();
        }
    }
}

#[no_mangle]
unsafe fn trap_handler() {
    (*MUX_SEL).set_high();
    /*
    let mip_bits = mip::read().bits();
    writeln!(Stdout(&mut *DBG_TX), "mip: {:b}", mip_bits).unwrap();
    let intr = (*CLAIM).claim();
    match intr {
        Some(i) => {
            writeln!(Stdout(&mut *DBG_TX), "Claimed {}", i.nr()).unwrap();
        }
        None => {
            writeln!(Stdout(&mut *DBG_TX), "Claimed nothing").unwrap();
        }
    }
    */

    //writeln!(Stdout(&mut *DBG_TX), "trap: {:?}", trap).unwrap();
    let trap = mcause::read().cause();
    if let Trap::Interrupt(intr) = trap {
        handle_interrupt(intr);
    } 
    else if let Trap::Exception(excpt) = trap {
        handle_exception(excpt);
    }
    else {
        writeln!(Stdout(&mut *DBG_TX), "No trap/int???").unwrap();
    }
    /*
    match cause {
        Trap::Interrupt(Interrupt::MachineExternal) => {
            writeln!(Stdout(&mut *DBG_TX), "MachExt").unwrap();
            /*
            match intr {
                e310x::Interrupt::UART0 => {
                    let ch = (*e310x::UART0::ptr()).rxdata.read().data().bits();
                    writeln!(Stdout(&mut *DBG_TX), "{}", ch).unwrap();
                }
                _ => {
                    writeln!(Stdout(&mut *DBG_TX), "Other interrupt").unwrap();
                }
            }
            */
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            writeln!(Stdout(&mut *DBG_TX), "MachineTimer").unwrap();
            writeln!(Stdout(&mut *DBG_TX), "timer: {}", (*MTIME_G).mtime()).unwrap();
            set_mtimecmp(&*MTIME_G, &mut *MTIMECMP_G);
        }
        Trap::Interrupt(Interrupt::UserSoft) => {
            writeln!(Stdout(&mut *DBG_TX), "UserSoft").unwrap();
            //writeln!(Stdout(&mut *DBG_TX), "Cause: {}", cause).unwrap();
        }
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "Other").unwrap();
            let intr = (*CLAIM).claim().unwrap();
            writeln!(Stdout(&mut *DBG_TX), "Claimed Other Interrupt: {}", intr.nr()).unwrap();
        }
    }

    match intr {
        Some(i) => {
            (*CLAIM).complete(i);
        }
        None => { }
    }
    */
}
