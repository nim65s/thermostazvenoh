use crate::error::Error;
use defmt::{error, info};
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output};
use zenoh_nostd::ZReply;

use crate::kalval::{KAL_CHAN, KalVal};
use crate::togglable::Togglable;

static LED_SIGNAL: Signal<CriticalSectionRawMutex, Togglable> = Signal::new();
pub fn led_query_cb(reply: &ZReply) {
    if let Err(e) = handle_reply(reply) {
        error!("Query callback error: {:?}", e);
    }
}

fn handle_reply<'a>(reply: &ZReply<'a>) -> Result<(), Error> {
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
            LED_SIGNAL.signal(payload.into());
        }
    }
    Ok(())
}

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    let sender = KAL_CHAN.sender();
    let send = async |level: Level| sender.send(KalVal::Led(level.into())).await;

    let mut level = Level::Low;
    send(level).await;

    loop {
        match select(LED_SIGNAL.wait(), Timer::after(Duration::from_secs(5 * 60))).await {
            Either::Second(()) => {}
            Either::First(val) => {
                level = match val {
                    Togglable::On => Level::High,
                    Togglable::Off => Level::Low,
                    Togglable::Toggle => !level,
                };
                info!("led {}", level);
                led.set_level(level);
            }
        }
        send(level).await;
    }
}

#[embassy_executor::task]
pub async fn led_sub_task(subscriber: zenoh_nostd::ZSubscriber<32, 128>) {
    loop {
        match subscriber.recv().await {
            Ok(sample) => LED_SIGNAL.signal(sample.payload().into()),
            Err(e) => error!("Invalid sample: {:?}", e),
        }
    }
}
