// This example requires a Unix-like OS with /dev/i2c-* and linux-embedded-hal.
// It is disabled on non-Unix targets so the crate can build on Windows and others.

#[cfg(unix)]
use core::fmt::Debug;
#[cfg(unix)]
use eeprom24x::{Eeprom24x, Eeprom24xTrait, SlaveAddr};
#[cfg(unix)]
use embedded_hal::delay::DelayNs;
#[cfg(unix)]
use linux_embedded_hal::{Delay, I2cdev};

#[cfg(unix)]
fn run<E: Debug>(eeprom: &mut impl Eeprom24xTrait<Error = E>) {
    let memory_address = 0x1234;
    let data = 0xAB;

    eeprom.write_byte(memory_address, data).unwrap();

    Delay.delay_ms(5u32);

    let read_data = eeprom.read_byte(memory_address).unwrap();

    println!(
        "Read memory address: {}, retrieved content: {}",
        memory_address, &read_data
    );
}

#[cfg(unix)]
fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut eeprom = Eeprom24x::new_24x256(dev, address);

    run(&mut eeprom);

    let _dev = eeprom.destroy(); // Get the I2C device back
}

#[cfg(not(unix))]
fn main() {
    // Example disabled on non-Unix targets.
}
