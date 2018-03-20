extern crate i2c;
extern crate i2cdev;

use std::cmp;
use i2cdev::core::I2CDevice;

pub struct I2cDev<I>(pub I);

impl<I: I2CDevice> i2c::Master for I2cDev<I> {
    type Error = I::Error;

    fn set_slave_address(&mut self, addr: u16, tenbit: bool) -> Result<(), Self::Error> {
        unimplemented!() // ugh why isn't this exposed
    }
}

impl<I: I2CDevice> i2c::ReadWrite for I2cDev<I> {
    fn i2c_read(&mut self, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(value)
            .map(|_| value.len())
    }

    fn i2c_write(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.0.write(value)
    }
}

impl<I: I2CDevice> i2c::Smbus for I2cDev<I> {
    fn smbus_write_quick(&mut self, value: bool) -> Result<(), Self::Error> {
        self.0.smbus_write_quick(value)
    }

    fn smbus_read_byte(&mut self) -> Result<u8, Self::Error> {
        self.0.smbus_read_byte()
    }

    fn smbus_write_byte(&mut self, value: u8) -> Result<(), Self::Error> {
        self.0.smbus_write_byte(value)
    }

    fn smbus_read_byte_data(&mut self, command: u8) -> Result<u8, Self::Error> {
        self.0.smbus_read_byte_data(command)
    }

    fn smbus_write_byte_data(&mut self, command: u8, value: u8) -> Result<(), Self::Error> {
        self.0.smbus_write_byte_data(command, value)
    }

    fn smbus_read_word_data(&mut self, command: u8) -> Result<u16, Self::Error> {
        self.0.smbus_read_word_data(command)
    }

    fn smbus_write_word_data(&mut self, command: u8, value: u16) -> Result<(), Self::Error> {
        self.0.smbus_write_word_data(command, value)
    }

    fn smbus_process_call(&mut self, command: u8, value: u16) -> Result<u16, Self::Error> {
        self.0.smbus_process_word(command, value)
    }

    fn smbus_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.smbus_read_block_data(command)
            .map(|data| {
                let len = cmp::min(value.len(), data.len());
                value[..len].copy_from_slice(&data[..len]);
                // TODO: warn about truncated data?
                len
            })
    }

    fn smbus_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        self.0.smbus_write_block_data(command, value)
    }
}

impl<I: I2CDevice> i2c::I2c for I2cDev<I> {
    fn i2c_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error> {
        if value.len() > 32 {
            unimplemented!()
        }

        self.0.smbus_read_i2c_block_data(command, value.len() as _)
            .map(|data| {
                let len = cmp::min(value.len(), data.len());
                value[..len].copy_from_slice(&data[..len]);
                // TODO: warn about truncated data?
                len
            })
    }

    fn i2c_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        if value.len() > 32 {
            unimplemented!()
        }

        // TODO: fix when https://github.com/rust-embedded/rust-i2cdev/issues/38 is resolved
        //self.0.smbus_write_i2c_block_data(command, value)
        self.0.smbus_process_block(command, value)
    }
}

/*
// TODO: impl when https://github.com/rust-embedded/rust-i2cdev/issues/38 to be resolved
impl<I: I2CDevice> i2c::Smbus20 for I2cDev<I> {
    fn smbus_process_call_block(&mut self, command: u8, write: &[u8], read: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.smbus_process_block(command, write)
            .map(|data| {
                let len = cmp::min(read.len(), data.len());
                read[..len].copy_from_slice(&data[..len]);
                // TODO: warn about truncated data?
                len
            })
    }
}
*/
