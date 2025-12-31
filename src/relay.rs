use crate::error::Error;
use defmt::{Format, error, info};
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output};
use zenoh_nostd::ZReply;

use crate::kalval::{KAL_CHAN, KalVal};

static RELAY_CMND: Channel<CriticalSectionRawMutex, RelayCmnd, 5> = Channel::new();

#[derive(Debug, Format)]
pub enum RelayCmnd {
    On,
    Off,
    Toggle,
}

#[derive(Debug, thiserror::Error, Format)]
pub enum RelayCmndError<'a> {
    #[error("invalid payload: {0:?}")]
    InvalidPayload(&'a [u8]),
}

impl<'a> TryFrom<&'a [u8]> for RelayCmnd {
    type Error = RelayCmndError<'a>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"ON" => Ok(Self::On),
            b"OFF" => Ok(Self::Off),
            b"TOGGLE" => Ok(Self::Toggle),
            _ => Err(Self::Error::InvalidPayload(value)),
        }
    }
}

pub fn relay_cmnd_callback(reply: &ZReply) {
    if let Err(e) = handle_reply(reply) {
        error!("Query callback error: {:?}", e);
    }
}

fn handle_reply<'a>(reply: &ZReply<'a>) -> Result<(), Error<'a>> {
    let (payload, keyexpr) = match reply {
        ZReply::Ok(r) | ZReply::Err(r) => (r.payload(), r.keyexpr().as_str()),
    };
    let payload_str = match core::str::from_utf8(payload) {
        Ok(p) => p,
        Err(_e) => "<Utf8Error>",
    };
    match reply {
        ZReply::Err(_) => defmt::error!(
            "[Query] Received ERR Reply ('{}': '{}')",
            keyexpr,
            payload_str
        ),
        ZReply::Ok(_) => {
            defmt::info!(
                "[Query] Received Ok Reply ('{}': '{}')",
                keyexpr,
                payload_str
            );
            RELAY_CMND.sender().try_send(payload.try_into()?)?;
        }
    }
    Ok(())
}

#[embassy_executor::task]
pub async fn relay_task(mut relay: Output<'static>) {
    let sender = KAL_CHAN.sender();
    let send = async |level: Level| sender.send(KalVal::Relay(Some(level.into()))).await;

    let mut level = Level::Low;
    send(level).await;

    let receiver = RELAY_CMND.receiver();
    loop {
        match select(
            receiver.receive(),
            Timer::after(Duration::from_secs(5 * 60)),
        )
        .await
        {
            Either::Second(()) => {}
            Either::First(val) => {
                level = match val {
                    RelayCmnd::On => Level::High,
                    RelayCmnd::Off => Level::Low,
                    RelayCmnd::Toggle => !level,
                };
                info!("relay {}", level);
                relay.set_level(level);
            }
        }
        send(level).await;
    }
}

#[embassy_executor::task]
pub async fn relay_cmnd_sub_task(subscriber: zenoh_nostd::ZSubscriber<32, 128>) {
    let sender = RELAY_CMND.sender();
    let send = async |cmnd: RelayCmnd| {
        sender.send(cmnd).await;
    };

    loop {
        match subscriber.recv().await {
            Ok(sample) => match sample.payload().try_into() {
                Ok(cmnd) => send(cmnd).await,
                Err(e) => error!("Can't decode sample: {:?}", e),
            },
            Err(e) => error!("Invalid sample: {:?}", e),
        }
    }
}
