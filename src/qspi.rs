#![allow(dead_code)]

extern crate e310x_hal as hal;

use self::hal::e310x::QSPI1;
use self::hal::time;

pub fn xfer(qspi: &QSPI1, payload: u8) -> u8 {
    while qspi.txdata.read().full().bit_is_set() {};

    unsafe {
        qspi.txdata.write(|w| w.bits(payload.into()));
    }

    while qspi.rxdata.read().empty().bit_is_clear() {};

    return qspi.rxdata.read().data().bits();
}

pub fn set_sck(qspi: &QSPI1, speed: time::Hertz, coreclk: time::Hertz) {
    let div_val = (coreclk.0 / (2 * speed.0) - 1) & 0xFFF;
    unsafe {
        qspi.div.write(|w| w.bits(div_val));
    }
}
