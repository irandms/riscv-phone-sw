#![no_std]
#![no_main]
#![feature(asm)]

extern crate hifive1;
extern crate pcd8544;
extern crate riscv;
extern crate panic_halt;

use riscv_rt::entry;
use hifive1::hal::prelude::*;
use hifive1::hal::e310x;
use hifive1::hal::stdout::*;
use pcd8544::PCD8544;

fn delay() {
    for _i in 0..1000 {
        unsafe {
            asm!("NOP");
        }
    }
}

#[entry]
fn main() -> ! {
    let p = e310x::Peripherals::take().unwrap();

    let _clint = p.CLINT.split();
    let _clocks = Clocks::freeze(p.PRCI.constrain(), p.AONCLK.constrain());
    let mut gpio = p.GPIO0.split();

    let mut pcd_clk = gpio.pin5.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_din = gpio.pin3.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_dc = gpio.pin20.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_ce = gpio.pin2.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_rst = gpio.pin11.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_light = gpio.pin9.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut pcd_light_real = gpio.pin19.into_output(
        &mut gpio.output_en,
        &mut gpio.drive,
        &mut gpio.out_xor,
        &mut gpio.iof_en,
    );

    let mut display = PCD8544::new(
        &mut pcd_clk,
        &mut pcd_din,
        &mut pcd_dc,
        &mut pcd_ce,
        &mut pcd_rst,
        &mut pcd_light,
    );

    display.reset();
    pcd_light_real.set_high();
    writeln!(display, "Standby").unwrap();
    writeln!(display, "").unwrap();
    writeln!(display, ">  Call").unwrap();
    writeln!(display, "2. Text").unwrap();
    writeln!(display, "3. Contacts").unwrap();
    writeln!(display, "4. Settings").unwrap();

    for _ in 0..200 {
        delay();
    }
    pcd_light_real.set_low();
    display.reset();

    loop {};
}
