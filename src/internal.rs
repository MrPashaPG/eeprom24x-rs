//! Internal helpers to keep implementation modular and DRY.
//! These are private to the crate and do not change the public API.

use crate::Error;

/// Validate page write boundaries and sizes.
/// Returns Error::TooMuchData when the operation would overflow a page
/// or the data is larger than the device page size.
pub(crate) fn validate_page_write<E>(
    address: u32,
    data_len: usize,
    page_size: usize,
) -> Result<(), Error<E>> {
    if data_len == 0 {
        return Ok(());
    }
    if data_len > page_size {
        return Err(Error::TooMuchData);
    }
    let page_boundary = address | (page_size as u32 - 1);
    if address + data_len as u32 > page_boundary + 1 {
        return Err(Error::TooMuchData);
    }
    Ok(())
}

/// Build the payload buffer with address bytes followed by the data slice.
/// TOTAL must be addr_bytes + max_page_size for this invocation.
/// Returns the fixed-size payload buffer and the effective length to transmit.
pub(crate) fn build_payload_with_address<const TOTAL: usize>(
    address: u32,
    addr_bytes: usize,
    data: &[u8],
    mut fill_address: impl FnMut(u32, &mut [u8]),
) -> ([u8; TOTAL], usize) {
    let mut payload: [u8; TOTAL] = [0; TOTAL];
    fill_address(address, &mut payload[..addr_bytes]);
    payload[addr_bytes..addr_bytes + data.len()].copy_from_slice(data);
    (payload, addr_bytes + data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_zero_len_ok() {
        assert!(validate_page_write::<()>(0x10, 0, 16).is_ok());
    }

    #[test]
    fn validate_over_page_size() {
        let err = validate_page_write::<()>(0x10, 17, 16).unwrap_err();
        assert!(matches!(err, Error::TooMuchData));
    }

    #[test]
    fn validate_crosses_boundary() {
        let err = validate_page_write::<()>(0x0F, 2, 16).unwrap_err();
        assert!(matches!(err, Error::TooMuchData));
    }

    #[test]
    fn validate_equal_to_boundary_ok() {
        assert!(validate_page_write::<()>(0x10, 16, 16).is_ok());
    }

    #[test]
    fn validate_near_end_within_ok() {
        assert!(validate_page_write::<()>(0x1E, 1, 16).is_ok());
    }

    #[test]
    fn build_payload_fills_address_and_data() {
        const TOTAL: usize = 2 + 32;
        let (payload, len) =
            build_payload_with_address::<TOTAL>(0x1234, 2, &[0xAA, 0xBB], |addr, out| {
                out[0] = (addr >> 8) as u8;
                out[1] = addr as u8;
            });
        assert_eq!(len, 2 + 2);
        assert_eq!(&payload[..4], &[0x12, 0x34, 0xAA, 0xBB]);
    }
}
