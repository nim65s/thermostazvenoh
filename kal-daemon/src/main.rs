use kal_daemon::{Mode, Schedule, Time};
use log::{debug, info};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut mode = Mode::default();
    let schedule = Schedule::default();
    let mut config = zenoh::Config::default();
    config
        .insert_json5("connect/endpoints", "[\"tcp/127.0.0.1:7447\"]")
        .unwrap();
    let session = zenoh::open(config).await.unwrap();
    let replies = session.get("kal/cmnd/daemon/mode").await.unwrap();
    while let Ok(reply) = replies.recv_async().await {
        if let Ok(payload) = reply.result().unwrap().payload().try_to_string() {
            mode = payload.as_ref().into();
            info!("mode {mode}");
        }
    }
    let mode_subscriber = session
        .declare_subscriber("kal/cmnd/daemon/mode")
        .await
        .unwrap();
    let temperature_subscriber = session
        .declare_subscriber("kal/tele/tasmota_43D8FD/temperature")
        .await
        .unwrap();

    loop {
        tokio::select! {
            reply = mode_subscriber.recv_async() => {
                if let Ok(sample) = reply {
                    if let Ok(payload) = sample.payload().try_to_string() {
                        mode = payload.as_ref().into();
                        info!("mode {mode}");
                    }
                }
            }
            reply = temperature_subscriber.recv_async() => {
                if let Ok(sample) = reply {
                    if let Ok(payload) = sample.payload().try_to_string() {
                        if let Ok(v) = payload.parse::<f64>() {
                            let t = v.into();
                            debug!("received {t}");
                            let h = match mode {
                                Mode::Auto => schedule.auto(Time::now(), t),
                                Mode::On => true,
                                Mode::Off => false,
                            };
                            let p = if h { "On" } else { "Off" };
                            debug!("relay {p}");
                            session.put("kal/cmnd/garage/relay", p).await.unwrap();
                        };
                    };
                }
            }
        };
    }
}
