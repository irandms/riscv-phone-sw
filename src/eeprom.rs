#![allow(dead_code)]

extern crate embedded_hal as hal;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::Mode;
use hal::spi::MODE_0;

#[derive(Debug)]
pub enum M95Error {
    // A write is currently happening, and subsequent writes will fail
    WriteInProgress,
    // The BP bits in the status register are write-protecting regions of memory
    WriteIsBlockProtected,
    WriteOutOfPage, // TODO: Implement this; writes beyond page boundaries begin back at the 0th offset of that page
}

pub const MAX_ADDR: u16     = 1 << 15; // 2^15 = 262,144 bits, 32768 bytes
pub const PAGE_SIZE: u32    = 64;
pub const MODE: Mode        = MODE_0;

pub mod status_reg {
    pub const SRWD_BIT: u8  = 1 << 7;
    pub const BP1_BIT: u8   = 1 << 3;
    pub const BP0_BIT: u8   = 1 << 2;
    pub const WEL_BIT: u8   = 1 << 1;
    pub const WIP_BIT: u8   = 1 << 0;
}

pub struct M95xxx<SPI, CS> {
    spi: SPI,
    cs: CS,
}

#[derive(Clone, Copy)]
enum Instruction {
    WriteEnable         = 0b0000_0110,
    WriteDisable        = 0b0000_0100,
    ReadStatusReg       = 0b0000_0101,
    WriteStatusReg      = 0b0000_0001,
    Read                = 0b0000_0011,
    Write               = 0b0000_0010,
    ReadIDPage          = 0b1000_0011,
    WriteIDPage         = 0b1000_0010,
    ContinueLastInstr   = 0b1111_1111,
}

impl <CS, SPI> M95xxx<SPI, CS>
where
    SPI: spi::Transfer<u8, Error = M95Error> + spi::Write<u8, Error = M95Error>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Result<Self, M95Error> {
        let mut eeprom = M95xxx { spi, cs };
        Ok(eeprom)
    }

    pub fn read(&mut self, addr: u16) -> Result<u8, M95Error> {
        let mut buffer = [
            Instruction::Read as u8,
            (addr >> 8) as u8,
            addr as u8,
            Instruction::ContinueLastInstr as u8,
        ];

        self.with_cs_low(|m95| {
            let buffer = m95.spi.transfer(&mut buffer).unwrap();

            Ok(buffer[3])
        })
    }

    pub fn read_n<'b>(&mut self, addr: u16, buffer: &'b mut [u8]) -> Result<&'b [u8], M95Error> {
        let mut cmd_buf = [
            Instruction::Read as u8,
            (addr >> 8) as u8,
            addr as u8,
        ];

        self.with_cs_low(|m95| {
            m95.spi.transfer(&mut cmd_buf).unwrap();

            let n = buffer.len();
            for byte in &mut buffer[..n] {
                *byte = m95.spi.transfer(&mut [Instruction::ContinueLastInstr as u8]).unwrap()[0];
            }

            Ok(&*buffer)
        })
    }

    pub fn status(&mut self) -> Result<u8, M95Error> {
        let mut buffer = [
            Instruction::ReadStatusReg as u8,
            Instruction::ContinueLastInstr as u8,
        ];

        self.with_cs_low(|m95| {
            let buffer = m95.spi.transfer(&mut buffer).unwrap();

            Ok(buffer[1])
        })
    }

    pub fn write_in_progress(&mut self) -> bool {
        match self.status() {
            Ok(status) => {
                (status & status_reg::WIP_BIT) != 0
            }
            _ => {
                false // TODO: Better error handling?
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<u8, M95Error> {
        if self.write_in_progress() {
            return Err(M95Error::WriteInProgress)
        }

        let mut buffer = [
            Instruction::Write as u8,
            (addr >> 8) as u8,
            addr as u8,
            data,
        ];

        self.with_cs_low(|m95| {
            let buffer = m95.spi.transfer(&mut buffer);
        });

        Ok(0)
    }

    fn with_cs_low<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.cs.set_low();
        let result = f(self);
        self.cs.set_high();

        result
    }
}
