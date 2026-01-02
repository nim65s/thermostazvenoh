use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::togglable::Togglable;

include!(concat!(env!("OUT_DIR"), "/keyexprs.rs"));

pub static KAL_CHAN: Channel<CriticalSectionRawMutex, KalVal, 3> = Channel::new();

#[derive(Debug, defmt::Format)]
pub enum KalVal {
    Hello,
    Led(Togglable),
    Relay(Togglable),
    Temperature(f32),
    Humidity(f32),
}

impl KalVal {
    pub fn as_string(&self) -> Result<heapless::String<10>, crate::error::Error> {
        Ok(match self {
            Self::Hello => heapless::String::try_from("1")?,
            Self::Relay(v) | Self::Led(v) => heapless::String::try_from(v.as_str())?,
            Self::Temperature(v) | Self::Humidity(v) => heapless::format!("{:.2}", v)?,
        })
    }
    pub fn as_cmnd_keyexpr(&self) -> &'static zenoh_nostd::keyexpr {
        match self {
            Self::Hello => CMND_HELLO,
            Self::Led(_) => CMND_LED,
            Self::Relay(_) => CMND_RELAY,
            Self::Temperature(_) => CMND_TEMPERATURE,
            Self::Humidity(_) => CMND_HUMIDITY,
        }
    }
    pub fn as_tele_keyexpr(&self) -> &'static zenoh_nostd::keyexpr {
        match self {
            Self::Hello => TELE_HELLO,
            Self::Led(_) => TELE_LED,
            Self::Relay(_) => TELE_RELAY,
            Self::Temperature(_) => TELE_TEMPERATURE,
            Self::Humidity(_) => TELE_HUMIDITY,
        }
    }
}
