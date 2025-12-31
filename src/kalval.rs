use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

include!(concat!(env!("OUT_DIR"), "/keyexprs.rs"));

pub static KAL_CHAN: Channel<CriticalSectionRawMutex, KalVal, 3> = Channel::new();

#[derive(Debug, defmt::Format)]
pub enum KeyExprType {
    Command,
    Telemetry,
}

#[derive(Debug, defmt::Format)]
pub enum KalVal {
    Led(Option<bool>),
    Relay(Option<bool>),
    Temperature(Option<f32>),
    Humidity(Option<f32>),
    DewPoint(Option<f32>),
}

impl KalVal {
    pub fn as_string(&self) -> Result<heapless::String<30>, core::fmt::Error> {
        match self {
            Self::Relay(Some(v)) | Self::Led(Some(v)) => heapless::format!("{}", v),
            Self::Temperature(Some(v)) | Self::Humidity(Some(v)) | Self::DewPoint(Some(v)) => {
                heapless::format!("{}", v)
            }
            _ => unreachable!(),
        }
    }
    pub fn as_keyexpr(&self, ke_type: &KeyExprType) -> &'static zenoh_nostd::keyexpr {
        match (ke_type, self) {
            (KeyExprType::Command, Self::Led(_)) => CMND_LED,
            (KeyExprType::Command, Self::Relay(_)) => CMND_RELAY,
            (KeyExprType::Command, Self::Temperature(_)) => CMND_TEMPERATURE,
            (KeyExprType::Command, Self::Humidity(_)) => CMND_HUMIDITY,
            (KeyExprType::Command, Self::DewPoint(_)) => CMND_DEWPOINT,
            (KeyExprType::Telemetry, Self::Led(_)) => TELE_LED,
            (KeyExprType::Telemetry, Self::Relay(_)) => TELE_RELAY,
            (KeyExprType::Telemetry, Self::Temperature(_)) => TELE_TEMPERATURE,
            (KeyExprType::Telemetry, Self::Humidity(_)) => TELE_HUMIDITY,
            (KeyExprType::Telemetry, Self::DewPoint(_)) => TELE_DEWPOINT,
        }
    }
}
