use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;
use shtcx::{self, LowPower, PowerMode};

use crate::{
    error::Error,
    kalval::{KAL_CHAN, KalVal},
};

#[embassy_executor::task]
pub async fn shtc3_task(mut sht: shtcx::ShtC3<I2c<'static, esp_hal::Blocking>>) {
    let sender = KAL_CHAN.sender();
    let send = async |h: shtcx::Humidity, t: shtcx::Temperature| {
        sender.send(KalVal::Humidity(h.as_percent())).await;
        sender
            .send(KalVal::Temperature(t.as_degrees_celsius()))
            .await;
    };

    sht.start_wakeup().ok();
    Timer::after(Duration::from_millis(1)).await;
    match sht.device_identifier() {
        Ok(device_id) => info!("ShtC3 Device ID : {:#02x}", device_id),
        Err(e) => error!("can't get ShtC3 device id i2c: {:?}", Error::from(e)),
    }

    loop {
        info!("Read shtc3 Humidity & Temperature");

        sht.start_wakeup().ok();
        Timer::after(Duration::from_millis(1)).await;
        sht.start_measurement(PowerMode::NormalMode).ok();
        Timer::after(Duration::from_millis(13)).await;
        match sht.get_measurement_result() {
            Ok(m) => send(m.humidity, m.temperature).await,
            Err(_e) => error!("can't read shtc3 data"),
        }
        sht.sleep().ok();

        Timer::after(Duration::from_secs(5 * 60)).await;
    }
}
