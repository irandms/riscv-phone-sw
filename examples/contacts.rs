#![no_std]
#![no_main]
#![feature(asm, fn_traits)]
#![allow(unreachable_code)] 
#![feature(type_ascription)]

#[path="../src/eeprom.rs"]
mod eeprom;

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;
extern crate shared_bus;

use riscv::{
    register::{
        mcause,
        mcause::{
            Interrupt,
            Trap, 
        },
    },
};
use riscv_rt::entry;
use hifive1::hal::{
    prelude::*,
    e310x::{
        Peripherals,
        UART0,
        PLIC,
        GPIO0,
    },
    plic::{
        Priority,
    },
    spi::Spi,
    stdout::*,
};
use embedded_hal::spi::MODE_0;

static mut DBG_TX: *mut Tx<UART0> = core::ptr::null_mut(); 

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
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
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks).listen();
    let (mut tx, _rx) = serial.split();

    let mut mux_sel = gpio.pin18.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    mux_sel.set_high();

    unsafe {
        clint.mtimer.disable();
        plic.mext.disable(); // MEIE bit in MIE register
        plic.uart0.disable(); // Enable the UART0 receive interrupt
        plic.threshold.set(Priority::P0); // Listen to any interrupt with priority > 0

        DBG_TX = &mut tx;
        (*PLIC::ptr()).enable[0].write(|w| w.bits(0));
        (*PLIC::ptr()).enable[1].write(|w| w.bits(0));
        (*PLIC::ptr()).priority[3].modify(|r, w| w.bits(r.bits() | 3));
        (*GPIO0::ptr()).rise_ie.write(|w| w.bits(0));

        riscv::interrupt::enable(); // MIE bit in MSTATUS register, MSIE in MIE
    };


    let mosi = gpio.pin3.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let miso = gpio.pin4.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let sck = gpio.pin5.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let mut eeprom_cs = gpio.pin2.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);
    let spi_pins = (mosi, miso, sck);
    let spi = Spi::spi1(p.QSPI1, spi_pins, MODE_0, 1_000_000_u32.hz(), clocks);

    //let spi_manager = shared_bus::RISCVBusManager::new(spi);
    //let eeprom = eeprom::M95xxx::new(spi_manager.acquire(), eeprom_cs).unwrap();
    let mut eeprom = eeprom::M95xxx::new(spi, eeprom_cs).unwrap();

    writeln!(Stdout(&mut tx), "EEPROM/Contacts test").unwrap();
    if cfg!(debug_assertions) {
        writeln!(Stdout(&mut tx), "Debug enabled").unwrap();
    }

    let mut overall_loop = 0;
    writeln!(Stdout(&mut tx), "Writing 11000011 into the first page of memory (64 bytes)").unwrap();
    let mut eeprom_addr = 0;
    loop {
        //writeln!(Stdout(&mut tx), "Writing byte {}...", eeprom_addr).unwrap();
        match eeprom.write(eeprom_addr as u16, 0b1100_0011) {
            Ok(_) => {
                eeprom_addr += 1;
            },
            Err(e) => {
                //writeln!(Stdout(&mut tx), "{:?}", e).unwrap();
            }
        }
        if eeprom_addr == 64 {
            break;
        }
        //overall_loop += 1;
        if overall_loop > 100000 {
            break;
        }
    }

    writeln!(Stdout(&mut tx), "Reading the first page of memory (64 bytes)").unwrap();
    for eeprom_addr in 0..64 {
        let eeprom_read_byte = eeprom.read(eeprom_addr as u16).unwrap();
        writeln!(Stdout(&mut tx), "{:b}", eeprom_read_byte as u8).unwrap();
    }

    writeln!(Stdout(&mut tx), "Example over").unwrap();
    loop {
    }
}

#[no_mangle]
unsafe fn handle_interrupt(intr: Interrupt) {
    match intr {
        _ => {
            writeln!(Stdout(&mut *DBG_TX), "machine ??? int").unwrap();
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
