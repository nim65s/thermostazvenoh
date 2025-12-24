#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::fmt::Write;
use core::num::NonZeroU32;

use aht20_async::Aht20;
use defmt::{error, info};
use embassy_executor::Spawner;
// use embassy_futures::select;
use embassy_net::{Runner, StackResources};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::{clock::CpuClock, ram, rng::Rng, timer::timg::TimerGroup};
use esp_println as _;
use esp_radio::{
    Controller,
    wifi::{
        ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
    },
};
use getrandom::register_custom_getrandom;
use heapless::String;
use zenoh_embassy::PlatformEmbassy;
use zenoh_nostd::{EndPoint, Session, keyexpr, zsubscriber};

extern crate alloc;

use thermostazvenoh::relay::{RELAY_LEVEL, relay_cmnd_callback, relay_cmnd_sub_task, relay_task};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
const CONNECT: &str = env!("CONNECT");
static RNG: Mutex<CriticalSectionRawMutex, Option<Rng>> = Mutex::new(None);

register_custom_getrandom!(getrandom_custom);
const MY_CUSTOM_ERROR_CODE: u32 = getrandom::Error::CUSTOM_START + 42;
pub fn getrandom_custom(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    unsafe {
        RNG.lock_mut(|rng_opt| {
            let code = NonZeroU32::new(MY_CUSTOM_ERROR_CODE).unwrap();
            let rng = rng_opt.as_mut().ok_or(getrandom::Error::from(code))?;
            rng.read(bytes);
            Ok(())
        })
    }
}

async fn gozenoh(
    spawner: embassy_executor::Spawner,
    mut aht: Aht20<I2c<'static, esp_hal::Async>, embassy_time::Delay>,
    stack: embassy_net::Stack<'static>,
) -> zenoh_nostd::ZResult<()> {
    let endpoint = EndPoint::try_from(CONNECT).unwrap();
    let zconfig = zenoh_nostd::zconfig!(
            PlatformEmbassy: (spawner, PlatformEmbassy { stack: stack}),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2,
            MAX_QUERYABLES: 2
    );

    let mut session = zenoh_nostd::open!(zconfig, endpoint);

    let ke_relay_cmnd_sub = keyexpr::new("cmnd/thermostazvenoh/RELAY").unwrap();
    let async_sub = session
        .declare_subscriber(
            ke_relay_cmnd_sub,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await
        .unwrap();
    session
        .get(ke_relay_cmnd_sub, relay_cmnd_callback)
        .send()
        .await
        .unwrap();
    spawner.spawn(relay_cmnd_sub_task(async_sub)).unwrap();

    loop {
        zenoh_put(&mut aht, &mut session).await;
        Timer::after(Duration::from_secs(6 * 50)).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let esp_radio_ctrl = &*mk_static!(Controller<'static>, esp_radio::init().unwrap());

    let (controller, interfaces) =
        esp_radio::wifi::new(&esp_radio_ctrl, peripherals.WIFI, Default::default()).unwrap();

    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    unsafe {
        RNG.lock_mut(|rng_opt| {
            *rng_opt = Some(rng);
        });
    }

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    loop {
        if stack.is_link_up() {
            info!("link up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("configure AHT20");
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(esp_hal::time::Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO10)
    .with_scl(peripherals.GPIO8);
    let aht = Aht20::new(i2c.into_async(), embassy_time::Delay)
        .await
        .expect("failed to init aht");

    info!("configure relay");
    let relay = Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default());
    spawner.spawn(relay_task(relay)).ok();

    if let Err(e) = gozenoh(spawner, aht, stack).await {
        error!("Error in gozenoh: {:?}", e);
    }
    Timer::after(Duration::from_secs(3)).await;
    esp_hal::system::software_reset()
}

async fn zenoh_put<'a>(
    aht: &mut Aht20<I2c<'a, esp_hal::Async>, embassy_time::Delay>,
    session: &mut Session<PlatformEmbassy>,
) {
    let mut msg: String<64> = String::new();
    let ke_pub = keyexpr::new("tele/thermostazvenoh/SENSOR").unwrap();

    info!("Read H T");

    let Ok((humidity, temperature)) = aht.read().await else {
        error!("aht: fail to read");
        return;
    };
    let level = RELAY_LEVEL.wait().await;
    let Ok(_) = write!(
        msg,
        "{{\"AHT20\":{{\"Temperature\": {:.2}, \"Humidity\": {:.2}}}, \"RELAY\": {:?}}}",
        temperature.celsius(),
        humidity.rh(),
        level,
    ) else {
        error!("write! measurement failed!");
        return;
    };

    info!("data: {}", msg);

    let Ok(_) = session.put(ke_pub, &msg.clone().into_bytes()).await else {
        error!("cant put");
        return;
    };
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");

            info!("Scan");
            let scan_config = ScanConfig::default().with_max(10);
            let result = controller
                .scan_with_config_async(scan_config)
                .await
                .unwrap();
            for ap in result {
                info!("{:?}", ap);
            }
        }
        info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                error!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
