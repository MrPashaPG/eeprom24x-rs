#![cfg(feature = "blocking")]

use embedded_hal_mock::eh1::i2c::Mock as I2cMock;
use eeprom24x::{Eeprom24x, SlaveAddr, page_size, addr_size, unique_serial};

// This test covers destroy() path which was previously uncovered explicitly.
#[test]
fn destroy_returns_bus() {
    let i2c = I2cMock::new(&[]);
    let mut i2c = Eeprom24x::<_, page_size::B8, addr_size::OneByte, unique_serial::No>::new_24x01(i2c, SlaveAddr::default()).destroy();
    i2c.done();
}
