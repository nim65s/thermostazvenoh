#[derive(Debug, thiserror::Error, defmt::Format)]
pub enum Error {
    #[error("esp radio initialization failed: {0:?}")]
    EspRadioInit(esp_radio::InitializationError),

    #[error("esp radio wifi error: {0:?}")]
    EspRadioWifi(esp_radio::wifi::WifiError),

    #[error("zenoh error: {0:?}")]
    ZError(zenoh_nostd::ZError),

    #[error("zenoh protocol error: {0:?}")]
    ZProtocol(zenoh_nostd::ZProtocolError),

    #[error("zenoh keyexpr error: {0:?}")]
    ZKeyExpr(zenoh_nostd::ZKeyExprError),
    //
    // #[error("aht20 error: {0:?}")]
    // AHT20(aht20_async::Error<esp_hal::i2c::master::Error>),
    #[error("aht20 error TODO")]
    AHT20,

    #[error("shtcx Crc error")]
    ShtCxCrc,

    #[error("I2c error TODO")]
    I2c(esp_hal::i2c::master::Error),

    #[error("core::fmt::Error error")]
    Format,

    #[error("core::fmt::Write error")]
    Write,

    #[error("string capacity error: {0:?}")]
    Capacity(heapless::CapacityError),
}

impl From<esp_radio::InitializationError> for Error {
    fn from(e: esp_radio::InitializationError) -> Self {
        Error::EspRadioInit(e)
    }
}

impl From<esp_radio::wifi::WifiError> for Error {
    fn from(e: esp_radio::wifi::WifiError) -> Self {
        Error::EspRadioWifi(e)
    }
}

impl From<zenoh_nostd::ZError> for Error {
    fn from(e: zenoh_nostd::ZError) -> Self {
        Error::ZError(e)
    }
}

impl From<zenoh_nostd::ZProtocolError> for Error {
    fn from(e: zenoh_nostd::ZProtocolError) -> Self {
        Error::ZProtocol(e)
    }
}

impl From<zenoh_nostd::ZKeyExprError> for Error {
    fn from(e: zenoh_nostd::ZKeyExprError) -> Self {
        Error::ZKeyExpr(e)
    }
}

// impl From<aht20_async::Error<esp_hal::i2c::master::Error>> for Error<'_> {
//     fn from(e: aht20_async::Error<esp_hal::i2c::master::Error>) -> Self {
//         Error::AHT20(e)
//     }
// }

impl From<heapless::CapacityError> for Error {
    fn from(e: heapless::CapacityError) -> Self {
        Error::Capacity(e)
    }
}

impl From<core::fmt::Error> for Error {
    fn from(_e: core::fmt::Error) -> Self {
        Error::Format
    }
}

#[cfg(feature = "shtc3")]
impl From<shtcx::Error<esp_hal::i2c::master::Error>> for Error {
    fn from(e: shtcx::Error<esp_hal::i2c::master::Error>) -> Self {
        match e {
            shtcx::Error::I2c(e) => Error::I2c(e),
            shtcx::Error::Crc => Error::ShtCxCrc,
        }
    }
}
