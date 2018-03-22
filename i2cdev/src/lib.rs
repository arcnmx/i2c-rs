#![deny(missing_docs)]
#![doc(html_root_url = "http://arcnmx.github.io/i2c-rs/")]

//! Implements the [`i2c::*`](https://crates.io/crates/i2c) traits for the
//! [`i2cdev` crate](https://crates.io/crates/i2cdev).

extern crate i2c;
extern crate i2cdev;

use std::{cmp, io};
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;

/// A wrapper around an `I2CDevice` type that impls the i2c traits.
///
/// This is required due to Rust's orphan rules.
pub struct I2cDev<I>(pub I);

impl<I: I2CDevice> I2cDev<I> {
    fn map_buffer(data: Vec<u8>, value: &mut [u8]) -> usize {
        let len = cmp::min(value.len(), data.len());
        value[..len].copy_from_slice(&data[..len]);
        data.len()
    }
}

impl<I: I2CDevice> i2c::Master for I2cDev<I> {
    type Error = I::Error;
}

impl i2c::Address for I2cDev<LinuxI2CDevice> {
    fn set_slave_address(&mut self, addr: u16, tenbit: bool) -> Result<(), Self::Error> {
        if tenbit {
            Err(io::Error::new(io::ErrorKind::Other, "10bit address not implemented in i2cdev").into())
        } else {
            self.0.set_slave_address(addr)
        }
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
            .map(|data| Self::map_buffer(data, value))
    }

    fn smbus_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        // TODO: error rather than let i2cdev silently truncate the data?
        self.0.smbus_write_block_data(command, value)
    }
}

impl<I: I2CDevice> i2c::BlockTransfer for I2cDev<I> {
    fn i2c_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.smbus_read_i2c_block_data(command, value.len() as _)
            .map(|data| Self::map_buffer(data, value))
    }

    fn i2c_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error> {
        // TODO: error rather than let i2cdev silently truncate the data?

        self.0.smbus_write_i2c_block_data(command, value)
    }
}

impl<I: I2CDevice> i2c::Smbus20 for I2cDev<I> {
    fn smbus_process_call_block(&mut self, command: u8, write: &[u8], read: &mut [u8]) -> Result<usize, Self::Error> {
        // TODO: error rather than let i2cdev silently truncate the data?

        self.0.smbus_process_block(command, write)
            .map(|data| Self::map_buffer(data, read))
    }
}
