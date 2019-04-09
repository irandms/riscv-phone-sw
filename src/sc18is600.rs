#![allow(dead_code)]

use hifive::hal::e310x::QSPI1;
use qspi::xfer;
use core::iter;

const DUMMY_BYTE: u8 = 0x00;

enum Command {
    WriteN             = 0x00,
    ReadN              = 0x01,
    I2cReadAfterWrite  = 0x02,
    I2cWriteAfterWrite = 0x03,
    ReadBuffer         = 0x06,
    ConfigureSpi       = 0x18,
    RegWrite           = 0x20,
    RegRead            = 0x21,
    Sleep              = 0x30,
}

enum RegAddr {
    IoConfig   = 0x00,
    IoState    = 0x01,
    I2cClock   = 0x02,
    I2cTimeout = 0x03,
    I2cStatus  = 0x04,
    I2cAddress = 0x05,
}

fn qspi_configure(qspi: &QSPI1) {
    unsafe {
        qspi.mode  .write(|w| w.phase().set_bit().polarity().set_bit());
        qspi.csid  .write(|w| w.bits(0b10));
        qspi.csmode.write(|w| w.bits(0b10));
    }
}

fn qspi_write_all(
    qspi: &QSPI1,
    command: &[u8],
    payload: impl IntoIterator<Item=u8>,
) -> u8 {
    qspi_configure(qspi);

    let result = command
        .iter()
        .cloned()
        .chain(payload)
        .fold(DUMMY_BYTE, |_, byte| xfer(qspi, byte));

    /*
    let mut result = 0;

    for byte in command {
        xfer(qspi, *byte);
    }
    for byte in payload {
        result = xfer(qspi, byte);
    }
    //let result = xfer(qspi, DUMMY_BYTE);
    */

    unsafe {
        qspi.csmode.write(|w| w.bits(0b00));
    }
    result
}

pub fn write_clock(qspi: &QSPI1, clock_hz: u32) -> u8 {
    let div_val = (7372800) / (4 * clock_hz);

    // 5 and 255 are min/max values specified in datasheet
    let clamped_div = match div_val {
        v if v < 5   => 5,
        v if v > 255 => 255,
        _            => div_val,
    };

    qspi_write_all(
        qspi,
        &[Command::RegWrite as u8, RegAddr::I2cClock as u8],
        iter::once(clamped_div as u8),
    )
}

pub fn read_clock(qspi: &QSPI1) -> u8 {
    qspi_write_all(
        qspi,
        &[Command::RegRead as u8, RegAddr::I2cClock as u8],
        iter::once(DUMMY_BYTE),
    )
}

pub fn write_timeout(qspi: &QSPI1, timeout: u8, enable: bool) -> u8 {
    let timeout = (timeout & 0xFE) | (enable as u8);

    qspi_write_all(
        qspi,
        &[Command::RegWrite as u8, RegAddr::I2cTimeout as u8],
        iter::once(timeout),
    )
}

pub fn write_n_bytes(qspi: &QSPI1, device_addr: u8, bytes: &[u8]) {
    qspi_write_all(
        qspi,
        &[Command::WriteN as u8, bytes.len() as u8, device_addr],
        bytes.iter().cloned(),
    );
}
