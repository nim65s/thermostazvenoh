use aht20_async::Aht20;
use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;

use crate::HUMI_TEMP;

#[embassy_executor::task]
pub async fn aht20_task(mut aht: Aht20<I2c<'static, esp_hal::Async>, embassy_time::Delay>) {
    loop {
        info!("Read H T");
        match aht.read().await {
            Ok((humidity, temperature)) => HUMI_TEMP.signal((humidity.rh(), temperature.celsius())),
            Err(_e) => error!("can't read aht20 data"),
        }
        Timer::after(Duration::from_secs(5 * 60)).await;
    }
}
