mod config;

use config::Config;

fn main() {
    let config = Config::from_config_file();

    println!("Hello, world!");
}
