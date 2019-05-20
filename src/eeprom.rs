#![allow(dead_code)]

extern crate embedded_hal as hal;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::Mode;
use hal::spi::MODE_0;

#[derive(Debug, Clone, Copy)]
pub enum Error<E> {
    // A write is currently happening, and subsequent writes will fail
    WriteInProgress,
    // The BP bits in the status register are write-protecting regions of memory
    WriteIsBlockProtected,
    WriteOutOfPage, // TODO: Implement this; writes beyond page boundaries begin back at the 0th offset of that page
    StatusReadFail,
    Spi(E),
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

impl <CS, SPI, E> M95xxx<SPI, CS>
where
    SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Result<Self, E> {
        Ok(M95xxx { spi, cs })
    }

    pub fn read(&mut self, addr: u16) -> Result<u8, E> {
        let mut buffer = [
            Instruction::Read as u8,
            (addr >> 8) as u8,
            addr as u8,
            Instruction::ContinueLastInstr as u8,
        ];

        let mut result = [
            Instruction::ContinueLastInstr as u8,
        ];

        self.with_cs_low(|m95| {
            m95.spi.transfer(&mut [Instruction::Read as u8])?;
            m95.spi.transfer(&mut [(addr >> 8) as u8])?;
            m95.spi.transfer(&mut [addr as u8])?;
            m95.spi.transfer(&mut result)?;

            Ok(result[0])
        })
    }

    pub fn read_n<'b>(&mut self, addr: u16, buffer: &'b mut [u8]) -> Result<&'b [u8], E> {
        let mut cmd_buf = [
            Instruction::Read as u8,
            (addr >> 8) as u8,
            addr as u8,
        ];

        self.with_cs_low(move |m95| {
            m95.spi.transfer(&mut cmd_buf)?;

            let n = buffer.len();
            for byte in &mut buffer[..n] {
                *byte = m95.spi.transfer(&mut [Instruction::ContinueLastInstr as u8])?[0];
            }

            Ok(&*buffer)
        })
    }

    pub fn status(&mut self) -> Result<u8, E> {
        let mut buffer = [
            Instruction::ReadStatusReg as u8,
            Instruction::ContinueLastInstr as u8,
        ];

        self.with_cs_low(|m95| {
            let buffer = m95.spi.transfer(&mut buffer)?;

            Ok(buffer[1])
        })
    }

    pub fn write_in_progress(&mut self) -> Result<bool, Error<E>> {
        match self.status() {
            Ok(status) => {
                Ok(status & status_reg::WIP_BIT != 0)
            }
            Err(e) => {
                Err(Error::WriteInProgress)
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error<E>> {
        match self.write_in_progress() {
            Ok(wip) => {
                if wip {
                    return Err(Error::WriteInProgress);
                }
            }
            Err(e) => {
                return Err(Error::StatusReadFail);
            }
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

        Ok(())
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
