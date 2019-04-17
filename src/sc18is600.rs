#![allow(dead_code)]

use hifive1::hal::{
    self,
    prelude::*,
    e310x::QSPI1,
    spi::Spi,
};

use embedded_hal::{
    digital::OutputPin,
    spi::MODE_3,
};

const DUMMY_BYTE: u8 = 0x00;
const BUFFER_LEN: u8 = 96;
const WR_BIT:     u8 = 0 << 0;
const RD_BIT:     u8 = 1 << 0;

enum Command {
    I2cWriteN          = 0x00,
    I2cReadN           = 0x01,
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

pub struct Sc18is600<'a, Cs> {
    pub cs:   &'a mut Cs,
    pub clocks: Clocks,
}

impl <'a, Cs: OutputPin> Sc18is600<'a, Cs> {
    // Open a session of communication with the SC18IS600 over QSPI1
    pub fn session<Mosi, Miso, Sck>(
        &mut self,
        qspi: QSPI1,
        pins: (Mosi, Miso, Sck),
        predicate: impl FnOnce(Writer<Mosi, Miso, Sck, Cs>)
    ) -> (QSPI1, (Mosi, Miso, Sck))
    where (Mosi, Miso, Sck): hal::spi::Pins<QSPI1>
    {
        let spi_ptr = hifive1::hal::e310x::QSPI1::ptr();
        // Create Spi object which will be freed at the end of this session
        let mut spi = Spi::spi1(qspi, pins, MODE_3, 1_000_000_u32.hz(), self.clocks);
        // the API doesn't expose the delay registers, so here are some raw writes into those registers
        // to configure them to meet the timing requirements of the SC18IS600
        unsafe { (*spi_ptr).delay0.write(|w| w.cssck().bits(0b1).sckcs().bits(0b1)); };
        unsafe { (*spi_ptr).delay1.write(|w| w.intercs().bits(1).interxfr().bits(8)); };

        // Run all of the commands desired in a single session through the Writer struct
        predicate(Writer(&mut spi, &mut *self.cs));

        spi.free()
    }
}

pub struct Writer<'a, Mosi, Miso, Sck, Cs: OutputPin>(&'a mut Spi<QSPI1, (Mosi, Miso, Sck)>, &'a mut Cs);

impl <'a, Mosi, Miso, Sck, Cs: OutputPin> Writer<'a, Mosi, Miso, Sck, Cs> {
    // Transfer a slice of data to the SC18IS600
    fn transfer<'buf>(&mut self, buf: &'buf mut [u8]) -> &'buf [u8] {
        self.1.set_low();
        let return_buf = self.0.transfer(buf);
        self.1.set_high();

        return_buf.unwrap()
    }

    pub fn write_clock(&mut self, clock_hz: u32) -> u8 {
        let div_val = (7372800) / (4 * clock_hz);

        // 5 and 255 are min/max values specified in datasheet
        let clamped_div  = match div_val {
            v if v < 5   => 5,
            v if v > 255 => 255,
            _            => div_val,
        };

        *self.transfer(&mut[
            Command::RegWrite as u8,
            RegAddr::I2cClock as u8,
            clamped_div as u8
        ]).last().unwrap()
    }

    pub fn read_io_config(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::IoConfig as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn read_io_state(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::IoState as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn read_clock(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::I2cClock as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn read_bus_timeout(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::I2cTimeout as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn get_bus_status(&mut self) -> &'static str {
        match self.read_bus_status() {
            0xF0 => "Transmission successful",
            0xF1 => "No ACK",
            0xF2 => "No ACK?",
            0xF3 => "Bus is busy",
            0xF8 => "Bus time-out",
            0xF9 => "Invalid data count.",
            _    => "Something horrible occured",
        }
    }

    pub fn read_bus_status(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::I2cStatus as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn read_bus_address(&mut self) -> u8 {
        *self.transfer(&mut[
            Command::RegRead as u8,
            RegAddr::I2cAddress as u8,
            DUMMY_BYTE
        ]).last().unwrap()
    }

    pub fn write_timeout(&mut self, timeout: u8, enable: bool) -> u8 {
        let timeout = (timeout & 0xFE) | (enable as u8);

        *self.transfer(&mut[
            Command::RegWrite as u8,
            RegAddr::I2cTimeout as u8,
            timeout
        ]).last().unwrap()
    }

    pub fn write_n_bytes(&mut self, device_addr: u8, bytes: &mut[u8]) {
        // Send using two calls to the underlying transfer,
        // in order to only have one falling edge on the chip select line
        self.1.set_low();
        self.0.transfer(&mut[
            Command::I2cWriteN as u8,
            bytes.len() as u8, 
            device_addr | WR_BIT,
        ]).unwrap();
        self.0.transfer(bytes.as_mut()).unwrap();
        self.1.set_high();
    }

    pub fn read_n_bytes(&mut self, device_addr: u8, n: u8) {
        // Send using two calls to the underlying transfer,
        // in order to only have one falling edge on the chip select line
        self.1.set_low();
        self.0.transfer(&mut[
            Command::I2cReadN as u8,
            n,
            device_addr | RD_BIT,
        ]).unwrap();
        self.1.set_high();
    }

    #[allow(unused_must_use)]
    pub fn read_after_write(&mut self, device_addr: u8, wr_bytes: &mut[u8], read_n: u8) {
        self.1.set_low();
        self.0.transfer(&mut[
            Command::I2cReadAfterWrite as u8,
            wr_bytes.len() as u8,
            read_n,
            device_addr | WR_BIT,
        ]);
        self.0.transfer(wr_bytes.as_mut()).unwrap();
        self.0.transfer(&mut[
            device_addr | RD_BIT,
        ]);
        self.1.set_high();
    }

    pub fn read_buffer(&mut self, transfer_buf: &mut [u8]) {
        self.1.set_low();
        self.0.transfer(&mut[
            Command::ReadBuffer as u8,
        ]).unwrap();

        self.0.transfer(transfer_buf).unwrap();
        self.1.set_high();
    }

    pub fn sleep(&mut self) {
        self.transfer(&mut[
            Command::Sleep as u8,
            0x5A,
            0xA5,
        ]).last().unwrap();
    }
}
