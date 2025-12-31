#[derive(Debug, defmt::Format)]
pub enum Togglable {
    On,
    Off,
    Toggle,
}

impl From<esp_hal::gpio::Level> for Togglable {
    fn from(value: esp_hal::gpio::Level) -> Self {
        match value {
            esp_hal::gpio::Level::High => Self::On,
            esp_hal::gpio::Level::Low => Self::Off,
        }
    }
}

impl From<bool> for Togglable {
    fn from(value: bool) -> Self {
        match value {
            true => Self::On,
            false => Self::Off,
        }
    }
}

impl From<Option<bool>> for Togglable {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::On,
            Some(false) => Self::Off,
            None => Self::Toggle,
        }
    }
}

impl<'a> From<&'a [u8]> for Togglable {
    fn from(value: &'a [u8]) -> Self {
        match value {
            b"ON" | b"On" | b"on" | b"TRUE" | b"True" | b"true" | b"1" => Self::On,
            b"OFF" | b"Off" | b"off" | b"FALSE" | b"False" | b"false" | b"0" => Self::Off,
            _ => Self::Toggle,
        }
    }
}

impl Togglable {
    pub fn as_str(&self) -> &str {
        match self {
            Togglable::On => "true",
            Togglable::Off => "false",
            Togglable::Toggle => "toggle",
        }
    }
}

impl Default for Togglable {
    fn default() -> Self {
        Self::Off
    }
}
