#![no_std]
#![feature(const_option_ops)]

#[cfg(feature = "aht20")]
pub mod aht20;

#[cfg(feature = "shtc3")]
pub mod shtc3;

pub mod error;
pub mod kalval;
pub mod network;
pub mod relay;
pub mod togglable;
