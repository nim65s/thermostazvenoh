use crate::relay::{RelayCmnd, RelayCmndError};
use embassy_sync::channel::TrySendError;

#[derive(Debug, thiserror::Error, defmt::Format)]
pub enum Error<'a> {
    #[error("relay command error: {0:?}")]
    RelayCmnd(RelayCmndError<'a>),

    #[error("zenoh reply cant be sent to relay: {0:?}")]
    ChannelTrySend(TrySendError<RelayCmnd>),
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
