#![deny(missing_docs)]
#![doc(html_root_url = "http://arcnmx.github.io/i2c-rs/")]

//! Generic traits encompassing operations on an I2C bus.
//!
//! Implementation of these traits are left up to downstream crates such as:
//!
//! - [i2c-linux](https://crates.io/crates/i2c-linux)
//! - [i2cdev](https://crates.io/crates/i2c-i2cdev)

#[macro_use]
extern crate bitflags;

use std::io::{self, Read, Write};

mod smbus;

pub use smbus::SmbusReadWrite;

/// Indicates an ability to communicate with the I2C protocol.
pub trait Master {
    /// The error type returned by I2C operations.
    type Error;
}

/// An I2C master can address different slaves on an I2C bus.
pub trait Address: Master {
    /// Sets the current slave to address.
    ///
    /// This should *not* be shifted to include the read/write bit, and
    /// therefore should be only 7 bits wide normally.
    fn set_slave_address(&mut self, addr: u16, tenbit: bool) -> Result<(), Self::Error>;
}

/// An I2C master that can communicate using the standard Read/Write traits.
///
/// The `i2c_read`/`i2c_write` methods are only provided to expose the original
/// error type, and should otherwise be identical to `read`/`write`.
pub trait ReadWrite: Master {
    /// Initiate an isolated read transfer on the I2C bus, followed by a STOP.
    fn i2c_read(&mut self, value: &mut [u8]) -> Result<usize, Self::Error>;

    /// Initiate an isolated write transfer on the I2C bus, followed by a STOP.
    fn i2c_write(&mut self, value: &[u8]) -> Result<(), Self::Error>;
}

impl<T: Master + Read + Write> ReadWrite for T where T::Error: From<io::Error> {
    fn i2c_read(&mut self, value: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(value).map_err(From::from)
    }

    fn i2c_write(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.write(value)
            .and_then(|len| if len != value.len() {
                Err(io::Error::new(io::ErrorKind::Interrupted, format!("I2C write was truncated to {} bytes", len)))
            } else {
                Ok(())
            }).map_err(From::from)
    }
}

/// SMBus operations
pub trait Smbus: Master {
    /// Sends a single bit to the device, in the place of the rd/wr address bit.
    fn smbus_write_quick(&mut self, value: bool) -> Result<(), Self::Error>;

    /// Reads a single byte from a device without specifying a register.
    fn smbus_read_byte(&mut self) -> Result<u8, Self::Error>;

    /// Sends a single byte to the device
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), Self::Error>;

    /// Reads a byte from the designated register.
    fn smbus_read_byte_data(&mut self, command: u8) -> Result<u8, Self::Error>;

    /// Writes a byte to the designated register.
    fn smbus_write_byte_data(&mut self, command: u8, value: u8) -> Result<(), Self::Error>;

    /// Reads a 16-bit word from the designated register.
    fn smbus_read_word_data(&mut self, command: u8) -> Result<u16, Self::Error>;

    /// Writes a 16-bit word to the designated register.
    fn smbus_write_word_data(&mut self, command: u8, value: u16) -> Result<(), Self::Error>;

    /// Writes a 16-bit word to the specified register, then reads a 16-bit word
    /// in response.
    fn smbus_process_call(&mut self, command: u8, value: u16) -> Result<u16, Self::Error>;

    /// Reads up to 32 bytes from the designated device register.
    ///
    /// Returns the number of bytes read, which may be larger than `value.len()`
    /// if the read was truncated.
    fn smbus_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error>;

    /// Writes up to 32 bytes to the designated device register.
    fn smbus_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error>;
}

/// SMBus 2.0 operations
pub trait Smbus20: Smbus {
    /// Sends up to 31 bytes of data to the designated register, and reads up to
    /// 31 bytes in return.
    ///
    /// Returns the number of bytes read, which may be larger than `read.len()`
    /// if the read was truncated.
    fn smbus_process_call_block(&mut self, command: u8, write: &[u8], read: &mut [u8]) -> Result<usize, Self::Error>;
}

/// SMBus Packet Error Checking
pub trait SmbusPec: Smbus {
    /// Enables or disables SMBus Packet Error Checking
    fn smbus_set_pec(&mut self, pec: bool) -> Result<(), Self::Error>;
}

/// Basic I2C transfer without including length prefixes associated with SMBus.
pub trait I2cBlock: Master {
    /// Reads a block of bytes from the designated device register.
    ///
    /// Unlike `smbus_read_block_data` this does not receive a data length.
    fn i2c_read_block_data(&mut self, command: u8, value: &mut [u8]) -> Result<usize, Self::Error>;

    /// Writes a block of bytes to the designated device register.
    ///
    /// Unlike `smbus_write_block_data` this does not transfer the data length.
    fn i2c_write_block_data(&mut self, command: u8, value: &[u8]) -> Result<(), Self::Error>;
}

/// Advanced I2C transfer queues that support repeated START operations.
pub trait BulkTransfer: Master {
    /// Specifies the flags that this implementation supports.
    fn i2c_transfer_support(&mut self) -> Result<(ReadFlags, WriteFlags), Self::Error>;

    /// Executes a queue of I2C transfers, separated by repeated START
    /// conditions. Data buffers are truncated to the actual read length on
    /// completion.
    fn i2c_transfer(&mut self, messages: &mut [Message]) -> Result<(), Self::Error>;
}

/// Part of a combined I2C transaction.
pub enum Message<'a> {
    /// I2C read command
    Read {
        /// The slave address of the device to read from.
        address: u16,
        /// A data buffer to read into.
        data: &'a mut [u8],
        /// Additional flags can modify the operation to work around device quirks.
        flags: ReadFlags,
    },
    /// I2C write command
    Write {
        /// The slave address of the device to write to.
        address: u16,
        /// The data to write.
        data: &'a [u8],
        /// Additional flags can modify the operation to work around device quirks.
        flags: WriteFlags,
    },
}

impl<'a> Message<'a> {
    /// Byte length of the message data buffer.
    pub fn len(&self) -> usize {
        match *self {
            Message::Read { ref data, .. } => data.len(),
            Message::Write { ref data, .. } => data.len(),
        }
    }

    /// Address of the message's slave.
    pub fn address(&self) -> u16 {
        match *self {
            Message::Read { address, .. } => address,
            Message::Write { address, .. } => address,
        }
    }

    /// The data buffer of the message.
    pub fn data(&self) -> &[u8] {
        match *self {
            Message::Read { ref data, .. } => data,
            Message::Write { data, .. } => data,
        }
    }
}

bitflags! {
    /// Flags to work around device quirks.
    #[derive(Default)]
    pub struct ReadFlags: u16 {
        /// The first received byte will indicate the remaining length of the transfer.
        const RECEIVE_LEN = 0x01;
        /// NACK bit is generated for this read.
        const NACK = 0x02;
        /// Flips the meaning of the read/write address bit for misbehaving devices.
        const REVERSE_RW = 0x04;
        /// Do not generate a START condition or the address start byte. When
        /// used for the first message, a START condition is still generated.
        ///
        /// This can be used to combine multiple buffers into a single I2C transfer,
        /// usually without a direction change.
        const NO_START = 0x08;
        /// Force a STOP condition after this message.
        const STOP = 0x10;
    }
}

bitflags! {
    /// Flags to work around device quirks.
    #[derive(Default)]
    pub struct WriteFlags: u16 {
        /// Treat NACK as an ACK and prevent it from interrupting the transfer.
        const IGNORE_NACK = 0x01;
        /// Flips the meaning of the read/write address bit for misbehaving devices.
        const REVERSE_RW = 0x02;
        /// Do not generate a START condition or the address start byte. When
        /// used for the first message, a START condition is still generated.
        ///
        /// This can be used to combine multiple buffers into a single I2C transfer,
        /// usually without a direction change.
        const NO_START = 0x04;
        /// Force a STOP condition after this message.
        const STOP = 0x08;
    }
}
