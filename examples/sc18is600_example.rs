#![no_std]
#![no_main]
#![feature(asm, fn_traits)]

extern crate hifive1;
extern crate panic_halt;
extern crate embedded_hal;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x::Peripherals;
use hifive1::hal::stdout::*;


#[path="../src/sc18is600.rs"]
mod sc18is600;

use sc18is600::Sc18is600;

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

    //              expecting 5, 19, 32, 6? 255
    let clock_speeds = [7200, 97000, 204000, 263000, 369000];
    let mut clock_iter = clock_speeds.into_iter().cycle();

    (0..).fold(
        (qspi, (mosi, miso, sck)),
        |(qspi, pins), _| {
            let mut clock_value_prev = 0;
            let mut clock_value_new  = 0;
            let ret = bridge.session(qspi, pins,
                |mut w| {
                    clock_value_prev = w.read_clock();
                    w.write_clock(*clock_iter.next().unwrap());
                    clock_value_new = w.read_clock();
                }
            );
            writeln!(Stdout(&mut tx), "read olld clock register as {}", clock_value_prev).unwrap();
            writeln!(Stdout(&mut tx), "read newe clock register as {}", clock_value_new).unwrap();
            delay_ms(100);
            ret
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
