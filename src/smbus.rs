use {Master, ReadWrite, Smbus};

/// A wrapper around an `i2c::ReadWrite` that impls `Smbus`.
pub struct SmbusReadWrite<I>(pub I);

impl<I: Master> Master for SmbusReadWrite<I> {
    type Error = I::Error;

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
            self.i2c_write(&[])
        } else {
            self.i2c_read(&mut [])
                .map(drop)
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

    fn smbus_read_block_data(&mut self, _command: u8, _value: &mut [u8]) -> Result<usize, Self::Error> {
        unimplemented!()
    }

    fn smbus_write_block_data(&mut self, _command: u8, _value: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
