use crate::relay::{RelayCmnd, RelayCmndError};
use embassy_sync::channel::TrySendError;

#[derive(Debug, thiserror::Error, defmt::Format)]
pub enum Error<'a> {
    #[error("relay command error: {0:?}")]
    RelayCmnd(RelayCmndError<'a>),

    #[error("zenoh reply cant be sent to relay: {0:?}")]
    ChannelTrySend(TrySendError<RelayCmnd>),

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

    #[error("core::fmt::Write error")]
    Write,
}

impl<'a> From<RelayCmndError<'a>> for Error<'a> {
    fn from(e: RelayCmndError<'a>) -> Self {
        Error::RelayCmnd(e)
    }
}

impl From<TrySendError<RelayCmnd>> for Error<'_> {
    fn from(e: TrySendError<RelayCmnd>) -> Self {
        Error::ChannelTrySend(e)
    }
}

impl From<esp_radio::InitializationError> for Error<'_> {
    fn from(e: esp_radio::InitializationError) -> Self {
        Error::EspRadioInit(e)
    }
}

impl From<esp_radio::wifi::WifiError> for Error<'_> {
    fn from(e: esp_radio::wifi::WifiError) -> Self {
        Error::EspRadioWifi(e)
    }
}

impl From<zenoh_nostd::ZError> for Error<'_> {
    fn from(e: zenoh_nostd::ZError) -> Self {
        Error::ZError(e)
    }
}

impl From<zenoh_nostd::ZProtocolError> for Error<'_> {
    fn from(e: zenoh_nostd::ZProtocolError) -> Self {
        Error::ZProtocol(e)
    }
}

impl From<zenoh_nostd::ZKeyExprError> for Error<'_> {
    fn from(e: zenoh_nostd::ZKeyExprError) -> Self {
        Error::ZKeyExpr(e)
    }
}

// impl From<aht20_async::Error<esp_hal::i2c::master::Error>> for Error<'_> {
//     fn from(e: aht20_async::Error<esp_hal::i2c::master::Error>) -> Self {
//         Error::AHT20(e)
//     }
// }
