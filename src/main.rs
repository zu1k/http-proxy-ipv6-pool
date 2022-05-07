mod proxy;

use log::{error, LevelFilter};
use proxy::start_proxy;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_target(false)
        .parse_default_env()
        .init();

    let bind_addr = match "0.0.0.0:51080".parse() {
        Ok(b) => b,
        Err(e) => {
            error!("bind address not valid: {}", e);
            return;
        }
    };
    if let Err(e) = start_proxy(bind_addr).await {
        error!("{}", e);
    }
}
