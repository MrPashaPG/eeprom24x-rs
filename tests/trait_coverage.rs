#![cfg(feature = "blocking")]

use eeprom24x::{Eeprom24x, Eeprom24xTrait, SlaveAddr};
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTrans};

mod common;
use crate::common::{destroy, new_24x16, DEV_ADDR};

fn exercise_trait<E: core::fmt::Debug>(dev: &mut impl Eeprom24xTrait<Error = E>) {
    // read_byte
    let _ = dev.read_byte(0x12).unwrap();
    // read_data
    let mut buf = [0u8; 3];
    dev.read_data(0x10, &mut buf).unwrap();
    // write_page
    dev.write_page(0x34, &[0xAB, 0xCD]).unwrap();
    // read_current_address
    let _ = dev.read_current_address().unwrap();
    // page_size
    let _ = dev.page_size();
}

#[test]
fn cover_trait_forwarders() {
    let trans = [
        // read_byte at 0x12
        I2cTrans::write_read(DEV_ADDR, vec![0x12], vec![0x77]),
        // read_data at 0x10 for 3 bytes
        I2cTrans::write_read(DEV_ADDR, vec![0x10], vec![0x01, 0x02, 0x03]),
        // write_page at 0x34 with 2 bytes
        I2cTrans::write(DEV_ADDR, vec![0x34, 0xAB, 0xCD]),
        // read_current_address
        I2cTrans::read(DEV_ADDR, vec![0xEE]),
    ];
    let mut eeprom = new_24x16(&trans);
    exercise_trait(&mut eeprom);
    destroy(eeprom);
}

#[test]
fn construct_eui_variants_blocking() {
    // These cover the extra constructors for EUI variants
    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::new_24x02e48(i2c, SlaveAddr::default()).destroy();
    i2c.done();

    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::new_24x02e64(i2c, SlaveAddr::default()).destroy();
    i2c.done();

    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::new_24x025e48(i2c, SlaveAddr::default()).destroy();
    i2c.done();

    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::new_24x025e64(i2c, SlaveAddr::default()).destroy();
    i2c.done();
}
