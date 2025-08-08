use crate::{
    addr_size::{OneByte, TwoBytes},
    unique_serial, Eeprom24x, Error,
};
use embedded_hal_async::i2c::I2c as AsyncI2c;

/// Determine the peripheral address for accessing the secure region
/// of 24CS devices.
fn secure_region_addr(address_bits: u8, base_addr: u8) -> u8 {
    match address_bits {
        7 | 8 | 12 | 13 => 0b101_1000 | (base_addr & 0b111), // CS01,CS02, CS32, CS64
        9 => 0b101_1000 | (base_addr & 0b110),               // CS04
        10 => 0b101_1000 | (base_addr & 0b100),              // CS08
        11 => 0b101_1000,                                    // CS16
        _ => unreachable!(),
    }
}

/// Async methods for interacting with the factory-programmed unique serial number
/// for devices with one byte addresses. e.g. 24CSx01, 24CSx02,24CSx04, 24CSx08,
/// and 24CSx16.
impl<I2C, PS, E> Eeprom24x<I2C, PS, OneByte, unique_serial::Yes>
where
    I2C: AsyncI2c<Error = E>,
{
    /// Read the 128-bit unique serial number.
    pub async fn read_unique_serial_async(&mut self) -> Result<[u8; 16], Error<E>> {
        let addr = secure_region_addr(self.address_bits, self.address.addr());
        let mut serial_bytes = [0u8; 16];
        self.i2c
            .write_read(addr, &[0x80], &mut serial_bytes)
            .await
            .map_err(Error::I2C)?;
        Ok(serial_bytes)
    }
}

/// Async methods for interacting with the factory-programmed unique serial number
/// for devices with two byte addresses. e.g. 24CSx32 and 24CSx64
impl<I2C, PS, E> Eeprom24x<I2C, PS, TwoBytes, unique_serial::Yes>
where
    I2C: AsyncI2c<Error = E>,
{
    /// Read the 128-bit unique serial number.
    pub async fn read_unique_serial_async(&mut self) -> Result<[u8; 16], Error<E>> {
        let secure_region_addr = 0b101_1000 | (self.address.addr() & 0b111);
        let mut serial_bytes = [0u8; 16];
        self.i2c
            .write_read(secure_region_addr, &[0x08, 0x0], &mut serial_bytes)
            .await
            .map_err(Error::I2C)?;
        Ok(serial_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::secure_region_addr;

    #[test]
    fn secure_region_addr_variants_mask_correct_bits() {
        let base = 0b101_0001u8; // Default base with A0 set
        // 7/8/12/13 keep A2..A0
        assert_eq!(0b101_1001, secure_region_addr(7, base));
        assert_eq!(0b101_1001, secure_region_addr(8, base));
        assert_eq!(0b101_1001, secure_region_addr(12, base));
        assert_eq!(0b101_1001, secure_region_addr(13, base));
        // 9 keeps A2..A1
        assert_eq!(0b101_1000, secure_region_addr(9, base));
        // 10 keeps only A2
        assert_eq!(0b101_1000, secure_region_addr(10, base));
        // 11 ignores A-bits entirely
        assert_eq!(0b101_1000, secure_region_addr(11, base));
    }

    #[test]
    #[should_panic]
    fn secure_region_addr_invalid_bits_panics() {
        // Any value outside the handled set should be unreachable
        let _ = secure_region_addr(0, 0b101_0000);
    }
}
