use aht20_async::Aht20;
use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;

use crate::kalval::{KAL_CHAN, KalVal};

#[embassy_executor::task]
pub async fn aht20_task(mut aht: Aht20<I2c<'static, esp_hal::Async>, embassy_time::Delay>) {
    let sender = KAL_CHAN.sender();
    let send = async |h: aht20_async::Humidity, t: aht20_async::Temperature| {
        sender.send(KalVal::Humidity(h.rh())).await;
        sender.send(KalVal::Temperature(t.celsius())).await;
    };

    loop {
        info!("Read aht20 Humidity & Temperature");

        match aht.read().await {
            Ok((h, t)) => send(h, t).await,
            Err(_e) => error!("can't read aht20 data"),
        }

        Timer::after(Duration::from_secs(5 * 60)).await;
    }
}
