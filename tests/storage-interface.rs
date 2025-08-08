#![cfg(feature = "blocking")]

use eeprom24x::{Eeprom24x, Error, Storage};
use embedded_hal_mock::eh1::{
    delay::NoopDelay,
    i2c::{Mock as I2cMock, Transaction as I2cTrans},
};
use embedded_storage::{ReadStorage, Storage as _};
mod common;
use crate::common::{
    destroy, new_24csx01, new_24csx02, new_24csx04, new_24csx08, new_24csx16, new_24csx32,
    new_24csx64, new_24x00, new_24x01, new_24x02, new_24x04, new_24x08, new_24x128, new_24x16,
    new_24x256, new_24x32, new_24x512, new_24x64, new_24xm01, new_24xm02, new_m24x01, new_m24x02,
    DEV_ADDR,
};

fn storage_new<PS, AS, SN>(
    eeprom: Eeprom24x<I2cMock, PS, AS, SN>,
) -> Storage<I2cMock, PS, AS, SN, NoopDelay> {
    Storage::new(eeprom, NoopDelay)
}

// Added: ensure Storage::destroy() is covered and returns both parts
#[test]
fn can_destroy_storage_and_retrieve_parts() {
    let storage = storage_new(new_24x01(&[]));
    let (mut i2c, _delay) = storage.destroy();
    // Verify mock consumed successfully
    i2c.done();
}

macro_rules! can_query_capacity {
    ($name:ident, $create:ident, $capacity:expr) => {
        #[test]
        fn $name() {
            let storage = storage_new($create(&[]));
            let capacity = storage.capacity();
            assert_eq!(capacity, $capacity);
            destroy(storage.eeprom);
        }
    };
}
for_all_ics_with_capacity!(can_query_capacity);

macro_rules! can_read_byte_1byte_addr {
    ($name:ident, $create:ident) => {
        #[test]
        fn $name() {
            let trans = [I2cTrans::write_read(DEV_ADDR, vec![0xF], vec![0xAB])];
            let mut storage = storage_new($create(&trans));
            let mut data = [0u8; 1];
            storage.read(0xF, &mut data).unwrap();
            assert_eq!(0xAB, data[0]);
            destroy(storage.eeprom);
        }
    };
}
for_all_ics_with_1b_addr!(can_read_byte_1byte_addr);

macro_rules! can_read_byte_2byte_addr {
    ($name:ident, $create:ident) => {
        #[test]
        fn $name() {
            let trans = [I2cTrans::write_read(DEV_ADDR, vec![0xF, 0x34], vec![0xAB])];
            let mut storage = storage_new($create(&trans));
            let mut data = [0u8; 1];
            storage.read(0xF34, &mut data).unwrap();
            assert_eq!(0xAB, data[0]);
            destroy(storage.eeprom);
        }
    };
}
for_all_ics_with_2b_addr!(can_read_byte_2byte_addr);

macro_rules! can_write_array_1byte_addr {
    ($name:ident, $create:ident, $_page_size:expr) => {
        #[test]
        fn $name() {
            let trans = [I2cTrans::write(DEV_ADDR, vec![0x34, 0xAB, 0xCD, 0xEF])];
            let mut storage = storage_new($create(&trans));
            storage.write(0x34, &[0xAB, 0xCD, 0xEF]).unwrap();
            destroy(storage.eeprom);
        }
    };
}
for_all_ics_with_1b_addr_and_page_size!(can_write_array_1byte_addr);

macro_rules! can_write_array_2byte_addr {
    ($name:ident, $create:ident, $_page_size:expr) => {
        #[test]
        fn $name() {
            let trans = [I2cTrans::write(DEV_ADDR, vec![0xF, 0x34, 0xAB, 0xCD, 0xEF])];
            let mut storage = storage_new($create(&trans));
            storage.write(0xF34, &[0xAB, 0xCD, 0xEF]).unwrap();
            destroy(storage.eeprom);
        }
    };
}
for_all_ics_with_2b_addr_and_page_size!(can_write_array_2byte_addr);

macro_rules! cannot_write_too_much_data {
    ($name:ident, $create:ident, $capacity:expr) => {
        #[test]
        fn $name() {
            let mut storage = storage_new($create(&[]));
            match storage.write(0x34, &[0xAB; 1 + $capacity]) {
                Err(Error::TooMuchData) => (),
                _ => panic!("Error::TooMuchData not returned."),
            }
            destroy(storage.eeprom);
        }
    };
}
for_all_writestorage_ics_with_capacity!(cannot_write_too_much_data);

// New: zero-length write should be a no-op
#[test]
fn zero_length_write_is_noop() {
    use crate::common::new_24x01;
    let mut storage = storage_new(new_24x01(&[]));
    storage.write(0x00, &[]).unwrap();
    destroy(storage.eeprom);
}

// New: multi-page write across three pages on 1-byte address device
#[test]
fn write_across_multiple_pages_one_byte_addr() {
    use crate::common::DEV_ADDR;
    // 24x01: page=8; write 20 bytes -> 8 + 8 + 4
    let big = [0x55u8; 20];
    let mut expected = Vec::new();
    // First page @ 0x00
    expected.push(I2cTrans::write(DEV_ADDR, {
        let mut v = Vec::with_capacity(1 + 8);
        v.push(0x00);
        v.extend_from_slice(&big[0..8]);
        v
    }));
    // Second page @ 0x08
    expected.push(I2cTrans::write(DEV_ADDR, {
        let mut v = Vec::with_capacity(1 + 8);
        v.push(0x08);
        v.extend_from_slice(&big[8..16]);
        v
    }));
    // Third page @ 0x10 (4 bytes)
    expected.push(I2cTrans::write(DEV_ADDR, {
        let mut v = Vec::with_capacity(1 + 4);
        v.push(0x10);
        v.extend_from_slice(&big[16..20]);
        v
    }));

    let mut storage = storage_new(new_24x01(&expected));
    embedded_storage::Storage::write(&mut storage, 0, &big).unwrap();
    destroy(storage.eeprom);
}

// New: multi-page write across two pages on 2-byte address device
#[test]
fn write_across_multiple_pages_two_byte_addr() {
    use crate::common::{new_24x32, DEV_ADDR};
    // 24x32: page=32; write 33 bytes -> 32 + 1
    let big = [0xAAu8; 33];
    let mut expected = Vec::new();
    // First page @ 0x0000
    expected.push(I2cTrans::write(DEV_ADDR, {
        let mut v = Vec::with_capacity(2 + 32);
        v.push(0x00); // high
        v.push(0x00); // low
        v.extend_from_slice(&big[0..32]);
        v
    }));
    // Second page @ 0x0020
    expected.push(I2cTrans::write(DEV_ADDR, {
        let mut v = Vec::with_capacity(2 + 1);
        v.push(0x00); // high
        v.push(0x20); // low
        v.extend_from_slice(&big[32..33]);
        v
    }));

    let mut storage = storage_new(new_24x32(&expected));
    embedded_storage::Storage::write(&mut storage, 0, &big).unwrap();
    destroy(storage.eeprom);
}
