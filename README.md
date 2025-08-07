# Rust 24x EEPROM Driver

[![crates.io](https://img.shields.io/crates/v/eeprom24x.svg)](https://crates.io/crates/eeprom24x)
[![Docs](https://docs.rs/eeprom24x/badge.svg)](https://docs.rs/eeprom24x)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.60+-blue.svg)
[![Build Status](https://github.com/eldruin/eeprom24x-rs/workflows/Build/badge.svg)](https://github.com/eldruin/eeprom24x-rs/actions?query=workflow%3ABuild)
[![Coverage Status](https://coveralls.io/repos/eldruin/eeprom24x-rs/badge.svg?branch=master)](https://coveralls.io/r/eldruin/eeprom24x-rs?branch=master)

This is a platform agnostic Rust driver for the 24x series serial EEPROM,
based on the [`embedded-hal`] and [`embedded-hal-async`] traits.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
[`embedded-hal-async`]: https://github.com/rust-embedded/embedded-hal/tree/master/embedded-hal-async

This driver allows you to:

- Read a single byte from a memory address. See: `read_byte()`.
- Read a byte array starting on a memory address. See: `read_data()`.
- Read the current memory address (please read notes). See: `read_current_address()`.
- Write a byte to a memory address. See: `write_byte()`.
- Write a byte array (up to a memory page) to a memory address. See: `write_page()`.
- Read `CSx`-variant devices' factory-programmed unique serial. See: `read_unique_serial()`.
- Use the device in generic code via the `Eeprom24xTrait`.
- **Async operations** when the `async` feature is enabled. See: `Eeprom24xAsyncTrait`.

Can be used at least with the devices listed below.

[Introductory blog post](https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/)

## The devices

These devices provides a number of bits of serial electrically erasable and
programmable read only memory (EEPROM) organized as a number of words of 8 bits
each. The devices' cascadable feature allows up to 8 devices to share a common
2-wire bus. The devices are optimized for use in many industrial and commercial
applications where low power and low voltage operation are essential.

| Device | Memory bits | 8-bit words | Page size | Datasheet  |
|-------:|------------:|------------:|----------:|:-----------|
|  24x00 |    128 bits |          16 |       N/A | [24C00]    |
|  24x01 |      1 Kbit |         128 |   8 bytes | [AT24C01]  |
| M24x01 |      1 Kbit |         128 |  16 bytes | [M24C01]   |
|  24x02 |      2 Kbit |         256 |   8 bytes | [AT24C02]  |
| M24x02 |      2 Kbit |         256 |  16 bytes | [M24C02]   |
|  24x04 |      4 Kbit |         512 |  16 bytes | [AT24C04]  |
|  24x08 |      8 Kbit |       1,024 |  16 bytes | [AT24C08]  |
|  24x16 |     16 Kbit |       2,048 |  16 bytes | [AT24C16]  |
|  24x32 |     32 Kbit |       4,096 |  32 bytes | [AT24C32]  |
|  24x64 |     64 Kbit |       8,192 |  32 bytes | [AT24C64]  |
| 24x128 |    128 Kbit |      16,384 |  64 bytes | [AT24C128] |
| 24x256 |    256 Kbit |      32,768 |  64 bytes | [AT24C256] |
| 24x512 |    512 Kbit |      65,536 | 128 bytes | [AT24C512] |
| 24xM01 |      1 Mbit |     131,072 | 256 bytes | [AT24CM01] |
| 24xM02 |      2 Mbit |     262,144 | 256 bytes | [AT24CM02] |

[24C00]: https://ww1.microchip.com/downloads/en/DeviceDoc/24AA00-24LC00-24C00-Data-Sheet-20001178J.pdf
[AT24C01]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8871F-SEEPROM-AT24C01D-02D-Datasheet.pdf
[M24C01]: https://www.st.com/resource/en/datasheet/m24c01-r.pdf
[AT24C02]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8871F-SEEPROM-AT24C01D-02D-Datasheet.pdf
[M24C02]: https://www.st.com/resource/en/datasheet/m24c02-r.pdf
[AT24C04]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8896E-SEEPROM-AT24C04D-Datasheet.pdf
[AT24C08]: https://ww1.microchip.com/downloads/en/DeviceDoc/AT24C08D-I2C-Compatible-2-Wire-Serial-EEPROM-20006022A.pdf
[AT24C16]: https://ww1.microchip.com/downloads/en/DeviceDoc/20005858A.pdf
[AT24C32]: https://ww1.microchip.com/downloads/en/devicedoc/doc0336.pdf
[AT24C64]: https://ww1.microchip.com/downloads/en/devicedoc/doc0336.pdf
[AT24C128]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8734-SEEPROM-AT24C128C-Datasheet.pdf
[AT24C256]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8568-SEEPROM-AT24C256C-Datasheet.pdf
[AT24C512]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8720-SEEPROM-AT24C512C-Datasheet.pdf
[AT24CM01]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8812-SEEPROM-AT24CM01-Datasheet.pdf
[AT24CM02]: https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-8828-SEEPROM-AT24CM02-Datasheet.pdf

## Usage

To use this driver, import this crate and an `embedded_hal` implementation,
then instantiate the appropriate device.
In the following examples an instance of the device AT24C256 will be created
as an example. Other devices can be created with similar methods like:
`Eeprom24x::new_24x64(...)`.

Please find additional examples using hardware in this repository: [driver-examples]

[driver-examples]: https://github.com/eldruin/driver-examples

```rust
use eeprom24x::{Eeprom24x, SlaveAddr};
use embedded_hal::blocking::delay::DelayMs;
use linux_embedded_hal::{Delay, I2cdev};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut eeprom = Eeprom24x::new_24x256(dev, address);
    let memory_address = 0x1234;
    let data = 0xAB;

    eeprom.write_byte(memory_address, data).unwrap();

    Delay.delay_ms(5u16);

    let read_data = eeprom.read_byte(memory_address).unwrap();

    println!(
        "Read memory address: {}, retrieved content: {}",
        memory_address, &read_data
    );

    let _dev = eeprom.destroy(); // Get the I2C device back
}
```

### Async Usage with Embassy

For async usage, enable the `async` feature:

```toml
[dependencies]
eeprom24x = { version = "0.8.0", features = ["async"] }
```

#### Example with `I2cDevice`
#### Using with esp-hal embassy-embedded-hal I2cDevice

```rust
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    peripherals::Peripherals,
    time::Rate,
};
use static_cell::StaticCell;

type I2c0 = Mutex<NoopRawMutex, I2c<'static, esp_hal::Async>>;

async fn async_example(peripherals: Peripherals) {
    let i2c0_init = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9)
    .into_async();

    static I2C0: StaticCell<I2c0> = StaticCell::new();
    let i2c0 = I2C0.init(Mutex::new(i2c0_init));

    let i2c_device = I2cDevice::new(i2c0);
    let address = SlaveAddr::default();
    let mut eeprom = Eeprom24x::new_24x256_async(i2c_device, address);
    let memory_address = 0x1234;
    let data = 0xAB;

    eeprom.write_byte_async(memory_address, data).await.unwrap();

    Timer::after_millis(5).await;

    let read_data = eeprom.read_byte_async(memory_address).await.unwrap();

    println!(
        "Read memory address: {}, retrieved content: {}",
        memory_address, &read_data
    );

    let _dev = eeprom.destroy_async(); // Get the I2C device back
}
```

#### Example with `I2cDeviceWithConfig`

```rust
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    peripherals::Peripherals,
    time::Rate,
};
use static_cell::StaticCell;

type I2c0 = Mutex<NoopRawMutex, I2c<'static, esp_hal::Async>>;

async fn async_example(peripherals: Peripherals) {
    let i2c0_init = I2c::new(
        peripherals.I2C0,
        I2cConfig::default(),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9)
    .into_async();

    static I2C0: StaticCell<I2c0> = StaticCell::new();
    let i2c0 = I2C0.init(Mutex::new(i2c0_init));

    let config = I2cConfig::default().with_frequency(Rate::from_khz(100)); // 100kHz
    let i2c_device = I2cDeviceWithConfig::new(i2c0, config);
    let address = SlaveAddr::default();
    let mut eeprom = Eeprom24x::new_24x256_async(i2c_device, address);
    let memory_address = 0x1234;
    let data = 0xAB;

    eeprom.write_byte_async(memory_address, data).await.unwrap();

    Timer::after_millis(5).await;

    let read_data = eeprom.read_byte_async(memory_address).await.unwrap();

    println!(
        "Read memory address: {}, retrieved content: {}",
        memory_address, &read_data
    );

    let _dev = eeprom.destroy_async(); // Get the I2C device back
}
```

## Features

### Async Support

This crate supports both blocking and async I2C operations:

- **`blocking`** (default): Uses `embedded-hal` for synchronous/blocking I2C operations
- **`async`**: Uses `embedded-hal-async` for asynchronous I2C operations

You can use only async support by adding the `async` feature to your `Cargo.toml`:

```toml
[dependencies]
eeprom24x = { version = "0.8", default-features = false, features = ["async"] }
```

Or use both blocking and async features together:

```toml
[dependencies]
eeprom24x = { version = "0.8", features = ["async"] }
```

Or use only blocking:

```toml
eeprom24x = { version = "0.8" }
```

### defmt-03

To enable [defmt](https://crates.io/crates/defmt) (version `0.3.x`) support, when specifying the dependency on `eeprom24x`, add the feature "`defmt-03`".

```toml
[dependencies]
eeprom24x = { version = "0.8.0", features = ["defmt-03"] }
```

## Support

For questions, issues, feature requests, and other changes, please file an
[issue in the github project](https://github.com/eldruin/eeprom24x-rs/issues).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
   <http://opensource.org/licenses/MIT>)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
