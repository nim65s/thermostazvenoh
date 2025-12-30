use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;
use shtcx::{self, LowPower, PowerMode};

use crate::{HUMI_TEMP, error::Error};

#[embassy_executor::task]
pub async fn shtc3_task(mut sht: shtcx::ShtC3<I2c<'static, esp_hal::Blocking>>) {
    sht.start_wakeup().ok();
    Timer::after(Duration::from_millis(1)).await;
    match sht.device_identifier() {
        Ok(device_id) => info!("ShtC3 Device ID : {:#02x}", device_id),
        Err(e) => error!("can't get ShtC3 device id i2c: {:?}", Error::from(e)),
    }
    loop {
        info!("Read shtc3 Humidity & Temperature");

        sht.start_measurement(PowerMode::NormalMode).ok();
        Timer::after(Duration::from_millis(13)).await;
        match sht.get_measurement_result() {
            Ok(m) => {
                HUMI_TEMP.signal((m.humidity.as_percent(), m.temperature.as_degrees_celsius()))
            }
            Err(_e) => error!("can't read shtc3 data"),
        }
        sht.sleep().ok();
        Timer::after(Duration::from_secs(5 * 60)).await;
        sht.start_wakeup().ok();
        Timer::after(Duration::from_millis(1)).await;
    }
}
