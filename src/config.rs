use serde::Deserialize;

/// The default value of the listener's hostname.
const DEFAULT_LISTENER_HOSTNAME: &str = "::";

/// The default value of the listener's port.
const DEFAULT_LISTENER_PORT: u16 = 5672;

#[derive(Deserialize)]
pub struct Config {
    pub log: Log,
    pub network: Network,
    pub queue: Queue,
}

#[derive(Deserialize)]
pub struct Log;

#[derive(Deserialize)]
pub struct Network {
    #[serde(default = "Network::default_hostname")]
    pub hostname: String,

    #[serde(default = "Network::default_port")]
    pub port: u16,
}

impl Network {
    fn default_hostname() -> String {
        String::from(DEFAULT_LISTENER_HOSTNAME)
    }

    fn default_port() -> u16 {
        DEFAULT_LISTENER_PORT
    }
}

#[derive(Deserialize)]
pub struct Queue;
