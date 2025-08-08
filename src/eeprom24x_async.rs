use crate::{addr_size, page_size, unique_serial, Eeprom24x, Error, MultiSizeAddr, SlaveAddr};
use core::marker::PhantomData;
use embedded_hal_async::i2c::I2c as AsyncI2c;
use crate::internal::{build_payload_with_address, validate_page_write};

/// Async common methods
impl<I2C, PS, AS, SN> Eeprom24x<I2C, PS, AS, SN> {
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy_async(self) -> I2C {
        self.i2c
    }
}

impl<I2C, PS, AS, SN> Eeprom24x<I2C, PS, AS, SN>
where
    AS: MultiSizeAddr,
{
    fn get_device_address_async<E>(&self, memory_address: u32) -> Result<u8, Error<E>> {
        if memory_address >= (1 << self.address_bits) {
            return Err(Error::InvalidAddr);
        }
        let addr = self.address.devaddr(
            memory_address,
            self.address_bits,
            AS::ADDRESS_BYTES as u8 * 8,
        );
        Ok(addr)
    }
}

/// Async common methods
impl<I2C, E, PS, AS, SN> Eeprom24x<I2C, PS, AS, SN>
where
    I2C: AsyncI2c<Error = E>,
    AS: MultiSizeAddr,
{
    /// Write a single byte in an address asynchronously.
    ///
    /// After writing a byte, the EEPROM enters an internally-timed write cycle
    /// to the nonvolatile memory.
    /// During this time all inputs are disabled and the EEPROM will not
    /// respond until the write is complete.
    pub async fn write_byte_async(&mut self, address: u32, data: u8) -> Result<(), Error<E>> {
        let devaddr = self.get_device_address_async(address)?;
        let mut payload = [0; 3];
        AS::fill_address(address, &mut payload);
        payload[AS::ADDRESS_BYTES] = data;
        self.i2c
            .write(devaddr, &payload[..=AS::ADDRESS_BYTES])
            .await
            .map_err(Error::I2C)
    }

    /// Read a single byte from an address asynchronously.
    pub async fn read_byte_async(&mut self, address: u32) -> Result<u8, Error<E>> {
        let devaddr = self.get_device_address_async(address)?;
        let mut memaddr = [0; 2];
        AS::fill_address(address, &mut memaddr);
        let mut data = [0; 1];
        self.i2c
            .write_read(devaddr, &memaddr[..AS::ADDRESS_BYTES], &mut data)
            .await
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }

    /// Read starting in an address as many bytes as necessary to fill the data array provided asynchronously.
    pub async fn read_data_async(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error<E>> {
        let devaddr = self.get_device_address_async(address)?;
        let mut memaddr = [0; 2];
        AS::fill_address(address, &mut memaddr);
        self.i2c
            .write_read(devaddr, &memaddr[..AS::ADDRESS_BYTES], data)
            .await
            .map_err(Error::I2C)
    }
}

/// Async specialization for platforms which implement `embedded_hal_async::i2c::I2c`
impl<I2C, E, PS, AS, SN> Eeprom24x<I2C, PS, AS, SN>
where
    I2C: AsyncI2c<Error = E>,
{
    /// Read the contents of the last address accessed during the last read
    /// or write operation, _incremented by one_ asynchronously.
    ///
    /// Note: This may not be available on your platform.
    pub async fn read_current_address_async(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .read(self.address.addr(), &mut data)
            .await
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }
}

/// Async specialization for devices without page access (e.g. 24C00)
impl<I2C, E> Eeprom24x<I2C, page_size::No, addr_size::OneByte, unique_serial::No>
where
    I2C: AsyncI2c<Error = E>,
{
    /// Create a new instance of a 24x00 device (e.g. 24C00) for async use
    pub fn new_24x00_async(i2c: I2C, address: SlaveAddr) -> Self {
        Eeprom24x {
            i2c,
            address,
            address_bits: 4,
            _ps: PhantomData,
            _as: PhantomData,
            _sn: PhantomData,
        }
    }
}

/// Async page write functionality
pub trait AsyncPageWrite<E> {
    fn page_write_async(
        &mut self,
        address: u32,
        data: &[u8],
    ) -> impl core::future::Future<Output = Result<(), Error<E>>>;
    fn page_size(&self) -> usize;
}

macro_rules! impl_create_async {
    ( $dev:expr, $part:expr, $address_bits:expr, $create:ident ) => {
        impl_create_async! {
            @gen [$create, $address_bits,
                concat!("Create a new instance of a ", $dev, " device (e.g. ", $part, ") for async use")]
        }
    };

    (@gen [$create:ident, $address_bits:expr, $doc:expr] ) => {
        #[doc = $doc]
        pub fn $create(i2c: I2C, address: SlaveAddr) -> Self {
            Self::new_async(i2c, address, $address_bits)
        }
    };
}

// This macro could be simplified once https://github.com/rust-lang/rust/issues/42863 is fixed.
macro_rules! impl_for_page_size_async {
    ( $AS:ident, $addr_bytes:expr, $PS:ident, $page_size:expr,
        $( [ $dev:expr, $part:expr, $address_bits:expr, $SN:ident, $create:ident ] ),* ) => {
        impl_for_page_size_async!{
            @gen [$AS, $addr_bytes, $PS, $page_size,
            concat!("Async specialization for devices with a page size of ", stringify!($page_size), " bytes."),
            concat!("Create generic async instance for devices with a page size of ", stringify!($page_size), " bytes."),
            $( [ $dev, $part, $address_bits, $SN, $create ] ),* ]
        }
    };

    (@gen [$AS:ident, $addr_bytes:expr, $PS:ident, $page_size:expr, $doc_impl:expr, $doc_new:expr,
        $( [ $dev:expr, $part:expr, $address_bits:expr, $SN:ident, $create:ident ] ),* ] ) => {

            $(
            impl<I2C, E> Eeprom24x<I2C, page_size::$PS, addr_size::$AS, unique_serial::$SN>
            where
                I2C: AsyncI2c<Error = E>
            {
                impl_create_async!($dev, $part, $address_bits, $create);
            }
            )*

            #[doc = $doc_impl]
            impl<I2C, E, SN> Eeprom24x<I2C, page_size::$PS, addr_size::$AS, SN>
            where
                I2C: AsyncI2c<Error = E>
            {
            #[doc = $doc_new]
            fn new_async(i2c: I2C, address: SlaveAddr, address_bits: u8) -> Self {
                Eeprom24x {
                    i2c,
                    address,
                    address_bits,
                    _ps: PhantomData,
                    _as: PhantomData,
                    _sn: PhantomData,
                }
            }
        }

        impl<I2C, E, AS, SN> Eeprom24x<I2C, page_size::$PS, AS, SN>
        where
            I2C: AsyncI2c<Error = E>,
            AS: MultiSizeAddr,
        {
            /// Write up to a page starting in an address asynchronously.
            ///
            /// The maximum amount of data that can be written depends on the page
            /// size of the device and its overall capacity. If too much data is passed,
            /// the error `Error::TooMuchData` will be returned.
            ///
            /// After writing a byte, the EEPROM enters an internally-timed write cycle
            /// to the nonvolatile memory.
            /// During this time all inputs are disabled and the EEPROM will not
            /// respond until the write is complete.
            pub async fn write_page_async(&mut self, address: u32, data: &[u8]) -> Result<(), Error<E>> {
                validate_page_write::<E>(address, data.len(), $page_size)?;
                if data.is_empty() {
                    return Ok(());
                }
                let devaddr = self.get_device_address_async(address)?;
                const TOTAL: usize = $addr_bytes + $page_size;
                let (payload, tx_len) = build_payload_with_address::<TOTAL>(
                    address,
                    $addr_bytes,
                    data,
                    |a, out| AS::fill_address(a, out),
                );
                self.i2c
                    .write(devaddr, &payload[..tx_len])
                    .await
                    .map_err(Error::I2C)
            }
        }

        impl<I2C, E, AS, SN> AsyncPageWrite<E> for Eeprom24x<I2C, page_size::$PS, AS, SN>
        where
            I2C: AsyncI2c<Error = E>,
            AS: MultiSizeAddr,
        {
            async fn page_write_async(&mut self, address: u32, data: &[u8]) -> Result<(), Error<E>> {
                self.write_page_async(address, data).await
            }

            fn page_size(&self) -> usize {
                $page_size
            }
        }

        impl<I2C, E, AS, SN> crate::Eeprom24xAsyncTrait for Eeprom24x<I2C, page_size::$PS, AS, SN>
        where
            I2C: AsyncI2c<Error = E>,
            AS: MultiSizeAddr
            {
                type Error = E;

                async fn write_byte_async(&mut self, address: u32, data: u8) -> Result<(), Error<Self::Error>>
                {
                    self.write_byte_async(address, data).await
                }

                async fn read_byte_async(&mut self, address: u32) -> Result<u8, Error<Self::Error>>
                {
                    self.read_byte_async(address).await
                }

                async fn read_data_async(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error<Self::Error>>
                {
                    self.read_data_async(address, data).await
                }

                async fn read_current_address_async(&mut self) -> Result<u8, Error<Self::Error>>
                {
                    self.read_current_address_async().await
                }

                async fn write_page_async(&mut self, address: u32, data: &[u8]) -> Result<(), Error<Self::Error>>
                {
                    self.write_page_async(address, &data).await
                }

                fn page_size(&self) -> usize
                {
                    $page_size
                }
            }
    };
}

impl_for_page_size_async!(
    OneByte,
    1,
    B8,
    8,
    ["24x01", "AT24C01", 7, No, new_24x01_async],
    ["24x02", "AT24C02", 8, No, new_24x02_async],
    ["24CSx01", "24CS01", 7, Yes, new_24csx01_async],
    ["24CSx02", "24CS02", 8, Yes, new_24csx02_async],
    ["24x02E48", "24AA02E48", 8, No, new_24x02e48_async],
    ["24x02E64", "24AA02E64", 8, No, new_24x02e64_async]
);
impl_for_page_size_async!(
    OneByte,
    1,
    B16,
    16,
    ["24x04", "AT24C04", 9, No, new_24x04_async],
    ["24x08", "AT24C08", 10, No, new_24x08_async],
    ["24x16", "AT24C16", 11, No, new_24x16_async],
    ["24CSx04", "AT24CS04", 9, Yes, new_24csx04_async],
    ["24CSx08", "AT24CS08", 10, Yes, new_24csx08_async],
    ["24CSx16", "AT24CS16", 11, Yes, new_24csx16_async],
    ["24x025E48", "24AA025E48", 8, No, new_24x025e48_async],
    ["24x025E64", "24AA025E64", 8, No, new_24x025e64_async],
    ["M24C01", "M24C01", 7, No, new_m24x01_async],
    ["M24C02", "M24C02", 8, No, new_m24x02_async]
);
impl_for_page_size_async!(
    TwoBytes,
    2,
    B32,
    32,
    ["24x32", "AT24C32", 12, No, new_24x32_async],
    ["24x64", "AT24C64", 13, No, new_24x64_async],
    ["24CSx32", "AT24CS32", 12, Yes, new_24csx32_async],
    ["24CSx64", "AT24CS64", 13, Yes, new_24csx64_async]
);
impl_for_page_size_async!(
    TwoBytes,
    2,
    B64,
    64,
    ["24x128", "AT24C128", 14, No, new_24x128_async],
    ["24x256", "AT24C256", 15, No, new_24x256_async]
);
impl_for_page_size_async!(
    TwoBytes,
    2,
    B128,
    128,
    ["24x512", "AT24C512", 16, No, new_24x512_async]
);
impl_for_page_size_async!(
    TwoBytes,
    2,
    B256,
    256,
    ["24xM01", "AT24CM01", 17, No, new_24xm01_async],
    ["24xM02", "AT24CM02", 18, No, new_24xm02_async]
);
