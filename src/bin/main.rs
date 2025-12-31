#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#[cfg(all(feature = "aht20", feature = "shtc3"))]
compile_error!("feature \"aht20\" and feature \"shtc3\" cannot be enabled at the same time");

use core::num::NonZeroU32;

#[cfg(feature = "aht20")]
use aht20_async::Aht20;
use defmt::{error, info};
use embassy_executor::Spawner;
// use embassy_futures::select;
use embassy_net::StackResources;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::{clock::CpuClock, peripherals::Peripherals, ram, rng::Rng, timer::timg::TimerGroup};
use esp_println as _;
use esp_radio::Controller;
use getrandom::register_custom_getrandom;
use zenoh_embassy::PlatformEmbassy;
use zenoh_nostd::{EndPoint, Session, zsubscriber};

extern crate alloc;

#[cfg(feature = "aht20")]
use kal::aht20::aht20_task;
use kal::error::Error;
use kal::kalval::{KAL_CHAN, KalVal, KeyExprType};
use kal::network::{connection, net_task};
use kal::relay::{relay_cmnd_callback, relay_cmnd_sub_task, relay_task};
#[cfg(feature = "shtc3")]
use kal::shtc3::shtc3_task;

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

const CONNECT: Option<&str> = option_env!("CONNECT");
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

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    if let Err(e) = real_main(peripherals, spawner).await {
        error!("init error: {:?}", e);
    }

    Timer::after(Duration::from_secs(5)).await;
    esp_hal::system::software_reset()
}

async fn real_main<'a>(peripherals: Peripherals, spawner: Spawner) -> Result<(), Error<'a>> {
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    info!("configure radio");
    let esp_radio_ctrl = &*mk_static!(Controller<'static>, esp_radio::init()?);

    let (controller, interfaces) =
        esp_radio::wifi::new(&esp_radio_ctrl, peripherals.WIFI, Default::default())?;

    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    unsafe {
        RNG.lock_mut(|rng_opt| {
            *rng_opt = Some(rng);
        });
    }

    info!("configure network stack");
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    info!("Waiting for link...");
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

    info!("configure i2c");
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(esp_hal::time::Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO10)
    .with_scl(peripherals.GPIO8);

    #[cfg(feature = "aht20")]
    {
        info!("configure AHT20");
        let aht20 = Aht20::new(i2c.into_async(), embassy_time::Delay)
            .await
            .expect("failed to init aht");
        spawner.spawn(aht20_task(aht20)).ok();
    }

    #[cfg(feature = "shtc3")]
    {
        let shtc3 = shtcx::shtc3(i2c);
        spawner.spawn(shtc3_task(shtc3)).ok();
    }

    info!("configure relay");
    let relay = Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default());
    spawner.spawn(relay_task(relay)).ok();

    info!("configure zenoh");
    let endpoint = EndPoint::try_from(CONNECT.unwrap_or("tcp/10.74.47.1:7447"))?;
    let zconfig = zenoh_nostd::zconfig!(
            PlatformEmbassy: (spawner, PlatformEmbassy { stack: stack}),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2,
            MAX_QUERYABLES: 2
    );

    let mut session = zenoh_nostd::open!(zconfig, endpoint);

    info!("configure relay cmnd subscriber");
    let ke_cmnd_relay = KalVal::Relay(None).as_keyexpr(&KeyExprType::Command);
    let async_sub = session
        .declare_subscriber(
            ke_cmnd_relay,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await?;
    session
        .get(ke_cmnd_relay, relay_cmnd_callback)
        .send()
        .await?;
    spawner.spawn(relay_cmnd_sub_task(async_sub)).ok();

    info!("initialization done, starting main loop");
    loop {
        if let Err(e) = main_loop(&mut session).await {
            error!("main loop error: {:?}", e);
        }
    }
}

async fn main_loop<'a>(session: &mut Session<PlatformEmbassy>) -> Result<(), Error<'a>> {
    let kalval = KAL_CHAN.receive().await;
    session
        .put(
            kalval.as_keyexpr(&KeyExprType::Telemetry),
            &kalval.as_string()?.into_bytes(),
        )
        .await?;

    Ok(())
}
