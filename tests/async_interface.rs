#![cfg(feature = "async")]

use eeprom24x::storage_async::AsyncStorage;
use eeprom24x::Eeprom24xAsyncTrait;
use eeprom24x::{addr_size, page_size, unique_serial, Eeprom24x, SlaveAddr, Storage};
use embedded_hal_async::i2c::{
    ErrorKind, ErrorType as AsyncI2cErrorType, I2c as AsyncI2c, Operation,
};

// A very small async I2C mock that returns predictable data
#[derive(Default)]
struct AsyncI2cMock {
    // next byte to return on 1-byte reads
    next_byte: u8,
}

impl AsyncI2cMock {
    fn with_byte(b: u8) -> Self {
        Self { next_byte: b }
    }
}

impl AsyncI2cErrorType for AsyncI2cMock {
    type Error = ErrorKind;
}

impl AsyncI2c for AsyncI2cMock {
    async fn read(&mut self, _address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        for r in read.iter_mut() {
            *r = self.next_byte;
        }
        Ok(())
    }

    async fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write_read(
        &mut self,
        _address: u8,
        _write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        for r in read.iter_mut() {
            *r = self.next_byte;
        }
        Ok(())
    }

    async fn transaction<'a>(
        &mut self,
        _address: u8,
        ops: &mut [Operation<'a>],
    ) -> Result<(), Self::Error> {
        for op in ops.iter_mut() {
            match op {
                Operation::Write(_w) => { /* ignore writes */ }
                Operation::Read(r) => {
                    for b in r.iter_mut() {
                        *b = self.next_byte;
                    }
                }
            }
        }
        Ok(())
    }
}

// Failing async I2C mock to exercise I2C error paths
struct AsyncI2cMockFail;
impl AsyncI2cErrorType for AsyncI2cMockFail {
    type Error = ErrorKind;
}
impl AsyncI2c for AsyncI2cMockFail {
    async fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        Err(ErrorKind::Other)
    }
    async fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        Err(ErrorKind::Other)
    }
    async fn write_read(
        &mut self,
        _address: u8,
        _write: &[u8],
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        Err(ErrorKind::Other)
    }
    async fn transaction<'a>(
        &mut self,
        _address: u8,
        _ops: &mut [Operation<'a>],
    ) -> Result<(), Self::Error> {
        Err(ErrorKind::Other)
    }
}

// Noop async delay for storage_async
struct NoopAsyncDelay;
impl embedded_hal_async::delay::DelayNs for NoopAsyncDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
    async fn delay_us(&mut self, _us: u32) {}
    async fn delay_ms(&mut self, _ms: u32) {}
}

#[tokio::test]
async fn async_eeprom_basic_ops() {
    // Use a device with 1-byte addr and 8-byte page
    let i2c = AsyncI2cMock::with_byte(0xAB);
    let mut eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c, SlaveAddr::default());

    // write/read single byte
    eeprom.write_byte_async(0x34, 0xCD).await.unwrap();
    let b = eeprom.read_byte_async(0x34).await.unwrap();
    assert_eq!(b, 0xAB);

    // write/read data slice
    let mut buf = [0u8; 3];
    eeprom.read_data_async(0x20, &mut buf).await.unwrap();
    assert_eq!(buf, [0xAB; 3]);

    // current address read
    let b2 = eeprom.read_current_address_async().await.unwrap();
    assert_eq!(b2, 0xAB);

    // page write
    eeprom.write_page_async(0x10, &[1, 2, 3]).await.unwrap();
}

#[tokio::test]
async fn async_storage_ops() {
    let i2c = AsyncI2cMock::with_byte(0x11);
    let eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c, SlaveAddr::default());
    let mut storage = Storage::new_async(eeprom, NoopAsyncDelay);

    // capacity
    assert_eq!(storage.capacity(), 1 << 7);

    // write and read
    storage.write_async(0x00, &[1, 2, 3, 4]).await.unwrap();
    let mut out = [0u8; 4];
    storage.read_async(0x00, &mut out).await.unwrap();
    assert_eq!(out, [0x11; 4]);
}

#[tokio::test]
async fn async_unique_serial_read_onebyte() {
    let i2c = AsyncI2cMock::with_byte(0x5A);
    let mut eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx01_async(i2c, SlaveAddr::default());
    let serial = eeprom.read_unique_serial_async().await.unwrap();
    assert_eq!(serial, [0x5A; 16]);
}

#[tokio::test]
async fn async_unique_serial_read_twobytes() {
    let i2c = AsyncI2cMock::with_byte(0x3C);
    let mut eeprom: Eeprom24x<_, page_size::B32, addr_size::TwoBytes, unique_serial::Yes> =
        Eeprom24x::new_24csx64_async(i2c, SlaveAddr::Alternative(false, true, false));
    let serial = eeprom.read_unique_serial_async().await.unwrap();
    assert_eq!(serial, [0x3C; 16]);
}

#[tokio::test]
async fn async_error_paths_and_validation() {
    // TooMuchData on page write
    let i2c_ok = AsyncI2cMock::with_byte(0);
    let mut eeprom_ok: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c_ok, SlaveAddr::default());
    let too_big = [0u8; 9];
    let err = eeprom_ok
        .write_page_async(0x00, &too_big)
        .await
        .err()
        .unwrap();
    assert!(matches!(err, eeprom24x::Error::TooMuchData));

    // InvalidAddr on read
    let i2c_ok2 = AsyncI2cMock::with_byte(0);
    let mut eeprom_ok2: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c_ok2, SlaveAddr::default());
    let mut buf = [0u8; 1];
    let err2 = eeprom_ok2
        .read_data_async(0x200, &mut buf)
        .await
        .err()
        .unwrap();
    assert!(matches!(err2, eeprom24x::Error::InvalidAddr));

    // I2C error is mapped
    let mut eeprom_fail: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(AsyncI2cMockFail, SlaveAddr::default());
    let res = eeprom_fail.read_byte_async(0x00).await;
    assert!(matches!(res, Err(eeprom24x::Error::I2C(ErrorKind::Other))));
}

// Additional async tests to increase coverage

#[tokio::test]
async fn async_empty_page_write_is_noop() {
    let i2c = AsyncI2cMock::with_byte(0);
    let mut eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c, SlaveAddr::default());
    // Should early-return Ok(())
    eeprom.write_page_async(0x10, &[]).await.unwrap();
}

#[tokio::test]
async fn async_destroy_methods() {
    // Eeprom destroy_async
    let i2c = AsyncI2cMock::with_byte(0);
    let eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c, SlaveAddr::default());
    let _i2c_back = eeprom.destroy_async();

    // Storage destroy_async
    let i2c2 = AsyncI2cMock::with_byte(0);
    let eeprom2: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c2, SlaveAddr::default());
    let storage = Storage::new_async(eeprom2, NoopAsyncDelay);
    let (_bus, _delay) = storage.destroy_async();
}

#[tokio::test]
async fn async_storage_multi_page_and_overflow() {
    let i2c = AsyncI2cMock::with_byte(0xEF);
    let eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(i2c, SlaveAddr::default());
    let mut storage = Storage::new_async(eeprom, NoopAsyncDelay);

    // Write over multiple pages (8 + 8 + 4)
    let big = [0x55u8; 20];
    storage.write_async(0, &big).await.unwrap();

    // Read a portion back
    let mut read_back = [0u8; 5];
    storage.read_async(0, &mut read_back).await.unwrap();
    assert_eq!(read_back, [0xEF; 5]);

    // Overflow check (capacity is 128 for 24x01)
    let too_large = [0u8; 129];
    let err = storage.write_async(0, &too_large).await.err().unwrap();
    assert!(matches!(err, eeprom24x::Error::TooMuchData));
}

#[tokio::test]
async fn async_current_address_read_error_maps() {
    let mut eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(AsyncI2cMockFail, SlaveAddr::default());
    let res = eeprom.read_current_address_async().await;
    assert!(matches!(res, Err(eeprom24x::Error::I2C(ErrorKind::Other))));
}

#[tokio::test]
async fn async_unique_serial_onebyte_all_variants() {
    // CS02 (address_bits = 8)
    let mut dev_cs02: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx02_async(
            AsyncI2cMock::with_byte(0x22),
            SlaveAddr::Alternative(true, false, true),
        );
    let s02 = dev_cs02.read_unique_serial_async().await.unwrap();
    assert_eq!(s02, [0x22; 16]);

    // CS04 (address_bits = 9)
    let mut dev_cs04: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx04_async(
            AsyncI2cMock::with_byte(0x44),
            SlaveAddr::Alternative(false, true, false),
        );
    let s04 = dev_cs04.read_unique_serial_async().await.unwrap();
    assert_eq!(s04, [0x44; 16]);

    // CS08 (address_bits = 10)
    let mut dev_cs08: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx08_async(
            AsyncI2cMock::with_byte(0x88),
            SlaveAddr::Alternative(true, true, false),
        );
    let s08 = dev_cs08.read_unique_serial_async().await.unwrap();
    assert_eq!(s08, [0x88; 16]);

    // CS16 (address_bits = 11)
    let mut dev_cs16: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::Yes> =
        Eeprom24x::new_24csx16_async(
            AsyncI2cMock::with_byte(0x16),
            SlaveAddr::Alternative(false, false, false),
        );
    let s16 = dev_cs16.read_unique_serial_async().await.unwrap();
    assert_eq!(s16, [0x16; 16]);
}

// Cover all async constructors and trait-forwarder methods to improve coverage.
#[tokio::test]
async fn async_all_constructors_and_trait_forwarders() {
    // 1) Constructors across families
    let _d00 = Eeprom24x::new_24x00_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let mut d01: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(AsyncI2cMock::with_byte(0xAA), SlaveAddr::default());
    let _d02 = Eeprom24x::new_24x02_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _de48 = Eeprom24x::new_24x02e48_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _de64 = Eeprom24x::new_24x02e64_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _d025e48 = Eeprom24x::new_24x025e48_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _d025e64 = Eeprom24x::new_24x025e64_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _m01 = Eeprom24x::new_m24x01_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let _m02 = Eeprom24x::new_m24x02_async(AsyncI2cMock::with_byte(0), SlaveAddr::default())
        .destroy_async();
    let d04: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x04_async(AsyncI2cMock::with_byte(0xBB), SlaveAddr::default());
    let d08: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x08_async(AsyncI2cMock::with_byte(0xCC), SlaveAddr::default());
    let d16: Eeprom24x<_, page_size::B16, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x16_async(AsyncI2cMock::with_byte(0xDD), SlaveAddr::default());
    let mut d32: Eeprom24x<_, page_size::B32, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x32_async(AsyncI2cMock::with_byte(0xEE), SlaveAddr::default());
    let d64: Eeprom24x<_, page_size::B32, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x64_async(AsyncI2cMock::with_byte(0x11), SlaveAddr::default());
    let d128: Eeprom24x<_, page_size::B64, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x128_async(AsyncI2cMock::with_byte(0x22), SlaveAddr::default());
    let d256: Eeprom24x<_, page_size::B64, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x256_async(AsyncI2cMock::with_byte(0x33), SlaveAddr::default());
    let d512: Eeprom24x<_, page_size::B128, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24x512_async(AsyncI2cMock::with_byte(0x44), SlaveAddr::default());
    let dm01: Eeprom24x<_, page_size::B256, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24xm01_async(AsyncI2cMock::with_byte(0x55), SlaveAddr::default());
    let dm02: Eeprom24x<_, page_size::B256, addr_size::TwoBytes, unique_serial::No> =
        Eeprom24x::new_24xm02_async(AsyncI2cMock::with_byte(0x66), SlaveAddr::default());

    // 2) Call trait-forwarder methods (Eeprom24xAsyncTrait) to exercise wrapper lines
    // One-byte address device
    Eeprom24xAsyncTrait::write_byte_async(&mut d01, 0x01, 0x7A)
        .await
        .unwrap();
    let _ = Eeprom24xAsyncTrait::read_byte_async(&mut d01, 0x01)
        .await
        .unwrap();
    let mut buf = [0u8; 2];
    Eeprom24xAsyncTrait::read_data_async(&mut d01, 0x01, &mut buf)
        .await
        .unwrap();
    let _ = Eeprom24xAsyncTrait::read_current_address_async(&mut d01)
        .await
        .unwrap();
    Eeprom24xAsyncTrait::write_page_async(&mut d01, 0x00, &[1, 2, 3])
        .await
        .unwrap();
    let _ps = Eeprom24xAsyncTrait::page_size(&d01);

    // Two-byte address device
    Eeprom24xAsyncTrait::write_byte_async(&mut d32, 0x0010, 0x7B)
        .await
        .unwrap();
    let _ = Eeprom24xAsyncTrait::read_byte_async(&mut d32, 0x0010)
        .await
        .unwrap();
    let mut buf2 = [0u8; 3];
    Eeprom24xAsyncTrait::read_data_async(&mut d32, 0x0020, &mut buf2)
        .await
        .unwrap();
    let _ = Eeprom24xAsyncTrait::read_current_address_async(&mut d32)
        .await
        .unwrap();
    Eeprom24xAsyncTrait::write_page_async(&mut d32, 0x0000, &[1, 2, 3, 4])
        .await
        .unwrap();
    let _ps2 = Eeprom24xAsyncTrait::page_size(&d32);

    // Call a couple more to mark those constructors as "used"
    let _ = d04.page_size();
    let _ = d08.page_size();
    let _ = d16.page_size();
    let _ = d64.page_size();
    let _ = d128.page_size();
    let _ = d256.page_size();
    let _ = d512.page_size();
    let _ = dm01.page_size();
    let _ = dm02.page_size();
}

#[tokio::test]
async fn async_24x00_basic_ops_and_bounds() {
    // 24x00 has 4 address bits (16 bytes)
    let mut d00 = Eeprom24x::new_24x00_async(AsyncI2cMock::with_byte(0xAB), SlaveAddr::default());
    d00.write_byte_async(0x0, 0x12).await.unwrap();
    let _ = d00.read_byte_async(0x0).await.unwrap();
    let mut slice = [0u8; 4];
    d00.read_data_async(0x0, &mut slice).await.unwrap();
    // Out of range
    let mut tmp = [0u8; 1];
    let err = d00.read_data_async(0x10, &mut tmp).await.err().unwrap();
    assert!(matches!(err, eeprom24x::Error::InvalidAddr));
}

#[tokio::test]
async fn async_storage_zero_length_write_is_noop() {
    let eeprom: Eeprom24x<_, page_size::B8, addr_size::OneByte, unique_serial::No> =
        Eeprom24x::new_24x01_async(AsyncI2cMock::with_byte(0), SlaveAddr::default());
    let mut storage = Storage::new_async(eeprom, NoopAsyncDelay);
    storage.write_async(0, &[]).await.unwrap();
}
