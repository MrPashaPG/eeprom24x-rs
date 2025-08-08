#![cfg(feature = "blocking")]

use eeprom24x::{addr_size, page_size, unique_serial, Eeprom24x, SlaveAddr};
use embedded_hal_mock::eh1::i2c::Mock as I2cMock;

// This test covers destroy() path which was previously uncovered explicitly.
#[test]
fn destroy_returns_bus() {
    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::<_, page_size::B8, addr_size::OneByte, unique_serial::No>::new_24x01(
        i2c,
        SlaveAddr::default(),
    )
    .destroy();
    i2c.done();
}
