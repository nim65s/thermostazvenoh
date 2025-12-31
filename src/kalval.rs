use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::togglable::Togglable;

include!(concat!(env!("OUT_DIR"), "/keyexprs.rs"));

pub static KAL_CHAN: Channel<CriticalSectionRawMutex, KalVal, 3> = Channel::new();

#[derive(Debug, defmt::Format)]
pub enum KeyExprType {
    Command,
    Telemetry,
}

#[derive(Debug, defmt::Format)]
pub enum KalVal {
    Led(Togglable),
    Relay(Togglable),
    Temperature(f32),
    Humidity(f32),
    DewPoint(f32),
}

impl KalVal {
    pub fn as_string(&self) -> Result<heapless::String<10>, crate::error::Error> {
        Ok(match self {
            Self::Relay(v) | Self::Led(v) => heapless::String::try_from(v.as_str())?,
            Self::Temperature(v) | Self::Humidity(v) | Self::DewPoint(v) => {
                heapless::format!("{:.2}", v)?
            }
        })
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
