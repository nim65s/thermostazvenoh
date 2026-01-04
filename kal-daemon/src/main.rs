use kal_daemon::Daemon;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut daemon = Daemon::init().await;
    loop {
        daemon.select().await;
    }
}
