pub mod config;

use config::AppConfig;

pub fn main() {
    let config = AppConfig::builder().build().unwrap();
    println!("{:?}", config.host);
    println!("{:?}", config.port);
    println!("{:?}", config.url);
    println!("{:?}", config.wrap_url);
}
