#![cfg(feature = "blocking")]

use eeprom24x::{addr_size, page_size, unique_serial, Eeprom24x, Error, SlaveAddr};
use embedded_hal::i2c::{ErrorType, I2c, Operation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestError { Boom }

// Implement embedded-hal I2C Error so TestError satisfies the required bound
impl embedded_hal::i2c::Error for TestError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind { embedded_hal::i2c::ErrorKind::Other }
}

struct FailingI2c;

impl ErrorType for FailingI2c { type Error = TestError; }

impl I2c for FailingI2c {
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        Err(TestError::Boom)
    }

    fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        Err(TestError::Boom)
    }

    fn write_read(&mut self, _address: u8, _write: &[u8], _read: &mut [u8]) -> Result<(), Self::Error> {
        Err(TestError::Boom)
    }

    fn transaction<'a>(&mut self, _address: u8, _operations: &mut [Operation<'a>]) -> Result<(), Self::Error> {
        Err(TestError::Boom)
    }
}

#[test]
fn blocking_i2c_error_paths_map_correctly() {
    // Use a typical 1-byte address, 8-byte page device
    let mut dev: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01(FailingI2c, SlaveAddr::default());

    // read_byte
    let e = dev.read_byte(0x00).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // read_data
    let mut buf = [0u8; 2];
    let e = dev.read_data(0x00, &mut buf).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // write_byte
    let e = dev.write_byte(0x00, 0x12).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // write_page (single byte within page)
    let e = dev.write_page(0x00, &[0xAB]).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // current address read
    let e = dev.read_current_address().err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));
}

#[test]
fn blocking_serial_number_error_paths_map_correctly() {
    // One-byte address device with serial
    let mut cs01: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx01(FailingI2c, SlaveAddr::default());
    let e = cs01.read_unique_serial().err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // Two-byte address device with serial
    let mut cs32: Eeprom24x<_, page_size::B32, addr_size::TwoBytes, unique_serial::Yes> =
        Eeprom24x::new_24csx32(FailingI2c, SlaveAddr::default());
    let e = cs32.read_unique_serial().err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));
}

#[test]
fn blocking_i2c_error_paths_map_correctly_two_bytes() {
    // Two-byte address, 32-byte page device
    let mut dev: Eeprom24x<_, page_size::B32, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x32(FailingI2c, SlaveAddr::default());

    // read_byte
    let e = dev.read_byte(0x0000).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // read_data
    let mut buf = [0u8; 2];
    let e = dev.read_data(0x0000, &mut buf).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // write_byte
    let e = dev.write_byte(0x0000, 0x12).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // write_page (single byte within page)
    let e = dev.write_page(0x0000, &[0xAB]).err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));

    // current address read
    let e = dev.read_current_address().err().unwrap();
    assert!(matches!(e, Error::I2C(TestError::Boom)));
}
