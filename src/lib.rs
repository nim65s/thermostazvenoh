#![no_std]
#![feature(const_option_ops)]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub static HUMI_TEMP: Signal<CriticalSectionRawMutex, (f32, f32)> = Signal::new();
pub static RELAY_LEVEL: Signal<CriticalSectionRawMutex, esp_hal::gpio::Level> = Signal::new();

#[cfg(feature = "aht20")]
pub mod aht20;

#[cfg(feature = "shtc3")]
pub mod shtc3;

pub mod error;
pub mod network;
pub mod relay;
