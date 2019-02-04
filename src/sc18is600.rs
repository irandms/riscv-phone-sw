extern crate hifive;

use hifive::hal::e310x::QSPI1;
use spi::qspi_xfer;

//enum command {
mod command {
    pub const WRITE_N: u8                  = 0x00;
    pub const READ_N: u8                   = 0x01;
    pub const I2C_READ_AFTER_WRITE: u8     = 0x02;
    pub const I2C_WRITE_AFTER_WRITE: u8    = 0x03;
    pub const READ_BUFFER: u8              = 0x06;
    pub const CONFIGURE_SPI: u8            = 0x18;
    pub const REG_WRITE: u8                = 0x20;
    pub const REG_READ: u8                 = 0x21;
    pub const SLEEP: u8                    = 0x30;
}

//enum reg_addr {
mod reg_addr {
    pub const IO_CONFIG: u8    = 0x00;
    pub const IO_STATE: u8     = 0x01;
    pub const I2C_CLOCK: u8    = 0x02;
    pub const I2C_TIMEOUT: u8  = 0x03;
    pub const I2C_STATUS: u8   = 0x04;
    pub const I2C_ADDRESS: u8  = 0x05;
}

pub fn write_clock(qspi: &QSPI1, clock_hz: u32) -> u8 {
    unsafe {
        qspi.mode.write(|w| w.phase().set_bit().polarity().set_bit());
        qspi.csid.write(|w| w.bits(2));
        qspi.csmode.write(|w| w.bits(0));

        let divisor = 4 * clock_hz / (7372800);

        qspi_xfer(qspi, command::REG_WRITE);
        qspi_xfer(qspi, reg_addr::I2C_CLOCK);
        return qspi_xfer(qspi, divisor as u8);
    }
}

pub fn read_clock(qspi: &QSPI1) -> u8 {
    unsafe {
        qspi.mode.write(|w| w.phase().set_bit().polarity().set_bit());
        qspi.csid.write(|w| w.bits(2));
        qspi.csmode.write(|w| w.bits(0));

        qspi_xfer(qspi, command::REG_READ);
        qspi_xfer(qspi, reg_addr::I2C_CLOCK);
        let dummy_byte = 0x00;
        return qspi_xfer(qspi, dummy_byte);
    }
}

pub fn write_timeout(qspi: &QSPI1, timeout: u8, enable: bool) -> u8 {
    let timeout_value = (timeout & 0xFE) | (enable as u8);

    unsafe {
        qspi.mode.write(|w| w.phase().set_bit().polarity().set_bit());
        qspi.csid.write(|w| w.bits(2));
        qspi.csmode.write(|w| w.bits(0));

        qspi_xfer(qspi, command::REG_WRITE);
        qspi_xfer(qspi, reg_addr::I2C_TIMEOUT);
        return qspi_xfer(qspi, timeout_value);
    }
}

pub fn write_n_bytes(qspi: &QSPI1, device_addr: u8, bytes: &[u8]) {
    unsafe {
        qspi.mode.write(|w| w.phase().set_bit().polarity().set_bit());
        qspi.csid.write(|w| w.bits(2));
        qspi.csmode.write(|w| w.bits(0));

        qspi_xfer(qspi, command::WRITE_N);
        qspi_xfer(qspi, bytes.len() as u8);
        qspi_xfer(qspi, device_addr);
        for byte in bytes.iter() {
            qspi_xfer(qspi, *byte);
        }
    }
}
