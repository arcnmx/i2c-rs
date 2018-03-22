use std::{iter, cmp};
use {Master, Address, I2cBlock, ReadWrite, Smbus, Smbus20};

/// A wrapper around an `i2c::ReadWrite` that attempts to impl `Smbus`.
///
/// Repeated START is not supported, which may confuse some devices and
/// probably makes this a non-conforming implementation.
pub struct SmbusReadWrite<I>(pub I);

impl<I: ReadWrite> SmbusReadWrite<I> {
    fn _smbus_read_block(&mut self, value: &mut [u8]) -> Result<usize, I::Error> {
        let mut buffer = vec![0u8; value.len() + 1];
        self.i2c_read(&mut buffer)
            .map(|len| {
                let len = cmp::min(cmp::min(len, buffer[0] as usize), value.len());
                value[..len].copy_from_slice(&buffer[1..len + 1]);
                buffer[0] as usize
            })
    }
}

impl<I: Master> Master for SmbusReadWrite<I> {
    type Error = I::Error;
}

impl<I: Address> Address for SmbusReadWrite<I> {
    fn set_slave_address(&mut self, addr: u16, tenbit: bool) -> Result<(), Self::Error> {
        self.0.set_slave_address(addr, tenbit)
    }
}

impl<I: ReadWrite> ReadWrite for SmbusReadWrite<I> {
    fn i2c_read(&mut self, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.i2c_read(value)
    }

    fn i2c_write(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.0.i2c_write(value)
    }
}

impl<I: ReadWrite> Smbus for SmbusReadWrite<I> {
    fn smbus_write_quick(&mut self, value: bool) -> Result<(), Self::Error> {
        if value {
            self.i2c_read(&mut [])
                .map(drop)
        } else {
            self.i2c_write(&[])
        }
    }

    fn smbus_read_byte(&mut self) -> Result<u8, Self::Error> {
        let mut buf = [0u8; 1];
        self.i2c_read(&mut buf)
            .and_then(|len| if len == buf.len() {
                Ok(buf[0])
            } else {
                unimplemented!()
            })
    }

    fn smbus_write_byte(&mut self, value: u8) -> Result<(), Self::Error> {
        self.i2c_write(&[value])
    }

    fn smbus_read_byte_data(&mut self, command: u8) -> Result<u8, Self::Error> {
        self.smbus_write_byte(command)?;
        self.smbus_read_byte()
    }

    fn smbus_write_byte_data(&mut self, command: u8, value: u8) -> Result<(), Self::Error> {
        self.i2c_write(&[command, value])
    }

    fn smbus_read_word_data(&mut self, command: u8) -> Result<u16, Self::Error> {
        let mut buf = [0u8; 2];
        self.smbus_write_byte(command)?;
        self.i2c_read(&mut buf)
            .and_then(|len| if len == buf.len() {
                Ok(buf[0] as u16 | (buf[1] as u16) << 8)
            } else {
                unimplemented!()
            })
    }

    fn smbus_write_word_data(&mut self, command: u8, value: u16) -> Result<(), Self::Error> {
        self.i2c_write(&[command, value as u8, (value >> 8) as u8])
    }

    fn smbus_process_call(&mut self, command: u8, value: u16) -> Result<u16, Self::Error> {
        self.smbus_write_word_data(command, value)?;
        let mut buf = [0u8; 2];
        self.i2c_read(&mut buf)
            .and_then(|len| if len == buf.len() {
                Ok(buf[0] as u16 | (buf[1] as u16) << 8)
            } else {
                unimplemented!()
            })
    }

    fn smbus_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.smbus_write_byte(command)?;
        self._smbus_read_block(value)
    }

    fn smbus_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        let buffer: Vec<_> = [command, value.len() as u8].iter().chain(value.iter()).cloned().collect();
        self.i2c_write(&buffer)
    }
}

impl<I: ReadWrite> I2cBlock for SmbusReadWrite<I> {
    fn i2c_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.smbus_write_byte(command)?;
        self.i2c_read(value)
    }

    fn i2c_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        let buffer: Vec<_> = iter::once(command).chain(value.iter().cloned()).collect();
        self.i2c_write(&buffer)
    }
}

impl<I: ReadWrite> Smbus20 for SmbusReadWrite<I> {
    fn smbus_process_call_block(&mut self, command: u8, write: &[u8], read: &mut [u8]) -> Result<usize, Self::Error> {
        self.smbus_write_block_data(command, write)?;
        self._smbus_read_block(read)
    }
}
