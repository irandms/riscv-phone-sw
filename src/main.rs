#![no_std]
#![no_main]
#![feature(asm, fn_traits)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;

mod sc18is600;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x::Peripherals;
use hifive1::hal::stdout::*;
use sc18is600::Sc18is600;

#[allow(dead_code)]
fn delay_ms(ms: u32) {
    let clint_ptr = e310x::CLINT::ptr();
    unsafe {
        let mtime_reg = &(*clint_ptr).mtime;
        let goal = mtime_reg.read().bits() + 32 * ms;
        while mtime_reg.read().bits() < goal {
            asm!("NOP");
        }
    }
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let _clint = p.CLINT.split();
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 16.mhz().into());

    let mut gpio = p.GPIO0.split();
    let (tx, rx) = hifive1::tx_rx(
        gpio.pin17,
        gpio.pin16,
        &mut gpio.out_xor,
        &mut gpio.iof_sel,
        &mut gpio.iof_en
    );
    let serial = Serial::uart0(p.UART0, (tx, rx), 115_200.bps(), clocks);
    let (mut tx, _) = serial.split();
    writeln!(Stdout(&mut tx), "\nSC18IS600 Example\n").unwrap();

    let mosi = gpio.pin3.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let miso = gpio.pin4.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let sck = gpio.pin5.into_iof0(&mut gpio.out_xor, &mut gpio.iof_sel, &mut gpio.iof_en);
    let mut bridge_cs = gpio.pin10.into_output(&mut gpio.output_en, &mut gpio.drive, &mut gpio.out_xor, &mut gpio.iof_en);

    let qspi = p.QSPI1;
    let mut bridge = Sc18is600 {
        cs: &mut bridge_cs,
        clocks: clocks,
    };

    let mut cfg = 0;
    let mut state = 0;
    let mut clk = 0;
    let mut to = 0;
    let mut stat = 0;
    let mut addr = 0;


    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.write_clock(97000);
            cfg     = w.read_io_config();
            state   = w.read_io_state();
            clk     = w.read_clock();
            to      = w.read_bus_timeout();
            stat    = w.read_bus_status();
            addr    = w.read_bus_address();
        }
    );

    writeln!(Stdout(&mut tx), "Config: {}", cfg).unwrap();
    writeln!(Stdout(&mut tx), "State: {}", state).unwrap();
    writeln!(Stdout(&mut tx), "Clock: {}", clk).unwrap();
    writeln!(Stdout(&mut tx), "Timeout: {}", to).unwrap();
    writeln!(Stdout(&mut tx), "Status: {}", stat).unwrap();
    writeln!(Stdout(&mut tx), "Addr: {}", addr).unwrap();

    let (qspi, (mosi, miso, sck)) = bridge.session(qspi, (mosi, miso, sck),
        |mut w| {
            w.write_clock(97000);
            stat    = w.read_bus_status();
        }
    );

    //writeln!(Stdout(&mut tx), "\nReading once from each register\n").unwrap();
    /*
    loop {
        for clock_speed in clock_speeds.iter() {
            let mut buf = [0x21, 0x2, 0xFF];
            bridge_cs.set_low();
            let old_clk = spi.transfer(&mut buf);
            bridge_cs.set_high();

            let mut buf = [0x20, 0x2, clock_speed];
            bridge_cs.set_low();
            spi.transfer(&mut buf);
            bridge_cs.set_high();

            let mut buf = [0x21, 0x2, 0xFF];
            bridge_cs.set_low();
            let new_clk = spi.transfer(&mut buf);
            bridge_cs.set_high();

            //writeln!(Stdout(&mut tx), "read olld clock register as {}", old_clk).unwrap();
            writeln!(Stdout(&mut tx), "read newe clock register as {}", new_clk).unwrap();
            //delay_ms(10);
        }
    }
    */

    loop {};
}
