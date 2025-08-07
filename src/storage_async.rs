use crate::{eeprom24x::MultiSizeAddr, eeprom24x_async::AsyncPageWrite, Eeprom24x, Error, Storage};
use core::cmp::min;

use embedded_hal_async::{delay::DelayNs as AsyncDelayNs, i2c::I2c as AsyncI2c};

/// Async common methods
impl<I2C, PS, AS, SN, D> Storage<I2C, PS, AS, SN, D> {}

/// Async common methods
impl<I2C, PS, AS, SN, D> Storage<I2C, PS, AS, SN, D>
where
    D: AsyncDelayNs,
{
    /// Create a new Storage instance wrapping the given Eeprom for async use
    pub fn new_async(eeprom: Eeprom24x<I2C, PS, AS, SN>, delay: D) -> Self {
        // When writing to the eeprom, we delay by 5 ms after each page
        // before writing to the next page.
        Storage { eeprom, delay }
    }
}

/// Async common methods
impl<I2C, PS, AS, SN, D> Storage<I2C, PS, AS, SN, D> {
    /// Destroy driver instance, return I²C bus and timer instance for async use.
    pub fn destroy_async(self) -> (I2C, D) {
        (self.eeprom.destroy_async(), self.delay)
    }
}

/// Async storage trait implementation
pub trait AsyncStorage {
    /// Inner implementation error.
    type Error;

    /// Read data from the storage asynchronously
    fn read_async(
        &mut self,
        offset: u32,
        bytes: &mut [u8],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>>;
    /// Write data to the storage asynchronously
    fn write_async(
        &mut self,
        offset: u32,
        bytes: &[u8],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>>;
    /// Return storage capacity in bytes
    fn capacity(&self) -> usize;
}

impl<I2C, E, PS, AS, SN, D> AsyncStorage for Storage<I2C, PS, AS, SN, D>
where
    I2C: AsyncI2c<Error = E>,
    AS: MultiSizeAddr,
    Eeprom24x<I2C, PS, AS, SN>: AsyncPageWrite<E>,
    D: AsyncDelayNs,
{
    type Error = Error<E>;

    async fn read_async(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.eeprom.read_data_async(offset, bytes).await
    }

    async fn write_async(&mut self, mut offset: u32, mut bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.capacity() {
            return Err(Error::TooMuchData);
        }
        let page_size = self.eeprom.page_size();
        while !bytes.is_empty() {
            let this_page_offset = offset as usize % page_size;
            let this_page_remaining = page_size - this_page_offset;
            let chunk_size = min(bytes.len(), this_page_remaining);
            self.eeprom
                .page_write_async(offset, &bytes[..chunk_size])
                .await?;
            offset += chunk_size as u32;
            bytes = &bytes[chunk_size..];
            // TODO At least ST's eeproms allow polling, i.e. trying the next i2c access which will
            // just be NACKed as long as the device is still busy. This could potentially speed up
            // the write process.
            // A (theoretically needless) delay after the last page write ensures that the user can
            // call Storage::write_async() again immediately.
            self.delay.delay_ms(5).await;
        }
        Ok(())
    }

    fn capacity(&self) -> usize {
        1 << self.eeprom.address_bits
    }
}
