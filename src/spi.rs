extern crate e310x;
use e310x::QSPI1;

pub fn qspi_xfer(qspi: &QSPI1, payload: u8) -> u8 {
    while qspi.txdata.read().full().bit_is_set() {};

    unsafe {
        qspi.txdata.write(|w| w.bits(payload.into()));
    }

    while qspi.rxdata.read().empty().bit_is_clear() {};

    return qspi.rxdata.read().data().bits();
}
