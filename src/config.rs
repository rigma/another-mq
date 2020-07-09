use log::Level;
use serde::{
    de::{self, Deserializer, Unexpected, Visitor},
    Deserialize,
};
use std::{
    convert::Into,
    env, fmt, fs,
    net::{IpAddr, Ipv4Addr},
    path::Path,
    process::Command,
};

/// The default value of the listener's hostname.
const DEFAULT_LISTENER_HOSTNAME: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

/// The default value of the listener's port.
const DEFAULT_LISTENER_PORT: u16 = 5672;

/// This data structure is holding the configuration defined by the user of `another-mq`. This 
/// configuration is loaded from a TOML file which can be edited by the user to fit its needs.
///
/// # Location of configuration file
///
/// The configuration file of `another-mq` is called `another-mq.toml`, and it's stored in different
/// locations depending on the OS you're using.
///
/// | Platform   | Default configuration file path                   |
/// | ---------- | ------------------------------------------------- |
/// | Windows    | `%APPDATA\another-mq\another-mq.toml`             |
/// | MacOS      | `$(brew --prefix)/etc/another-mq/another-mq.toml` |
/// | Linux/Unix | `$ANOTHERMQ_HOME/etc/another-mq/another-mq.toml`  |
///
/// > By default, the `$ANOTHERMQ_HOME` variable is empty.
/// >
/// > For MacOS platform, if `brew` is not installed, the default configuration file path will be the same as
/// > as for Linux/Unix.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// The log namespace.
    pub log: Log,

    /// The network namespace.
    pub network: Network,

    /// The queue namespace.
    pub queue: Queue,
}

impl Config {
    /// Loads the configuration from an arbitrary TOML file specified by the user.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let raw = match fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(_) => {
                return Self::default();
            }
        };

        match toml::from_str(&raw) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Loads the configuration from the default TOML configuration file.
    ///
    /// If the configuration could not be loaded by the application, a default instance of the
    /// `Config` structure will be returned instead.
    #[cfg(target_os = "windows")]
    pub fn from_config_file() -> Self {
        let config_path = env::var("APPDATA")
            .expect("%APPDATA% environment variable is not defined on your system!");
        let config_path = config_path + "/another-mq/another-mq.toml";

        let raw = match fs::read_to_string(config_path) {
            Ok(raw) => raw,
            Err(_) => {
                return Self::default();
            }
        };

        match toml::from_str(&raw) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Loads the configuration from the default TOML configuration file.
    ///
    /// If the configuration could not be loaded by the application, a default instance of the
    /// `Config` structure will be returned instead.
    #[cfg(target_os = "macos")]
    pub fn from_config_file() -> Self {
        let install_prefix = Command::new("brew").arg("--prefix").output();
        let install_prefix = match install_prefix {
            Ok(output) => String::from_utf8(output.stdout).unwrap(),
            Err(_) => env::var("ANOTHERMQ_HOME").unwrap_or_else(|_| "".into()),
        };

        let config_path = install_prefix + "/etc/another-mq/another-mq.toml";

        let raw = match fs::read_to_string(config_path) {
            Ok(raw) => raw,
            Err(_) => {
                return Self::default();
            }
        };

        match toml::from_str(&raw) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Loads the configuration from the default TOML configuration file.
    ///
    /// If the configuration could not be loaded by the application, a default instance of the
    /// `Config` structure will be returned instead.
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    pub fn from_config_file() -> Self {
        let home_prefix = env::var("ANOTHERMQ_HOME").unwrap_or_else(|_| "".into());
        let config_path = home_prefix + "/etc/another-mq/another-mq.toml";

        let raw = match fs::read_to_string(config_path) {
            Ok(raw) => raw,
            Err(_) => {
                return Self::default();
            }
        };

        match toml::from_str(&raw) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log: Log::default(),
            network: Network::default(),
            queue: Queue::default(),
        }
    }
}

/// The log namespace of the configuration. By default, log entries are emitted
/// on the standard output of the application. But a logfile or a syslog server
/// can also be used to collect log entries.
#[derive(Clone, Debug, Deserialize)]
pub struct Log {
    /// The minimum level of an entry to be added to the application log.
    #[serde(default = "Log::default_level")]
    pub level: Level,

    /// The path to the logfile of the application.
    pub file: Option<String>,

    /// The syslog configuration of the application.
    pub syslog: Option<Syslog>,
}

impl Log {
    fn default_level() -> Level {
        Level::Info
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            file: None,
            syslog: None,
        }
    }
}

/// The syslog configuration of the application log.
#[derive(Clone, Debug, Deserialize)]
pub struct Syslog {
    pub host: Option<IpAddr>,
    pub port: Option<u16>,
    pub protocol: SyslogProtocol,
    pub facility: SyslogFacility,
    pub process: String,
}

impl Default for Syslog {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            protocol: SyslogProtocol::Rfc3164,
            facility: SyslogFacility::User,
            process: String::new(),
        }
    }
}

/// The syslog protocol to use.
#[derive(Copy, Clone, Debug)]
pub enum SyslogProtocol {
    Rfc3164,
    Rfc5424,
}

impl<'de> Deserialize<'de> for SyslogProtocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SyslogProtocolVisitor;

        impl<'de> Visitor<'de> for SyslogProtocolVisitor {
            type Value = SyslogProtocol;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting rfc3164 or rfc5424")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "rfc3164" | "RFC3164" => Ok(SyslogProtocol::Rfc3164),
                    "rfc5424" | "RFC5424" => Ok(SyslogProtocol::Rfc5424),
                    _ => Err(de::Error::invalid_value(
                        Unexpected::Str(value),
                        &"Unknown syslog protocol!",
                    )),
                }
            }
        }

        deserializer.deserialize_enum(
            "SyslogProtocol",
            &["rfc3164", "RFC3164", "rfc5324", "RFC5424"],
            SyslogProtocolVisitor,
        )
    }
}

/// The syslog facility to use.
#[derive(Copy, Clone, Debug)]
pub enum SyslogFacility {
    Kern,
    User,
    Mail,
    Daemon,
    Auth,
    Syslog,
    Lpr,
    News,
    Uucp,
    Cron,
    AuthPriv,
    Ftp,
    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
}

impl Into<syslog::Facility> for SyslogFacility {
    fn into(self) -> syslog::Facility {
        use syslog::Facility;

        match self {
            Self::Kern => Facility::LOG_KERN,
            Self::User => Facility::LOG_USER,
            Self::Mail => Facility::LOG_MAIL,
            Self::Daemon => Facility::LOG_DAEMON,
            Self::Auth => Facility::LOG_AUTH,
            Self::Syslog => Facility::LOG_SYSLOG,
            Self::Lpr => Facility::LOG_LPR,
            Self::News => Facility::LOG_NEWS,
            Self::Uucp => Facility::LOG_UUCP,
            Self::Cron => Facility::LOG_CRON,
            Self::AuthPriv => Facility::LOG_AUTHPRIV,
            Self::Ftp => Facility::LOG_FTP,
            Self::Local0 => Facility::LOG_LOCAL0,
            Self::Local1 => Facility::LOG_LOCAL1,
            Self::Local2 => Facility::LOG_LOCAL2,
            Self::Local3 => Facility::LOG_LOCAL3,
            Self::Local4 => Facility::LOG_LOCAL4,
            Self::Local5 => Facility::LOG_LOCAL5,
            Self::Local6 => Facility::LOG_LOCAL6,
            Self::Local7 => Facility::LOG_LOCAL7,
        }
    }
}

impl<'de> Deserialize<'de> for SyslogFacility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SyslogFacilityVisitor;

        impl<'de> Visitor<'de> for SyslogFacilityVisitor {
            type Value = SyslogFacility;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a syslog facility")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "kern" => Ok(Self::Value::Kern),
                    "user" => Ok(Self::Value::User),
                    "mail" => Ok(Self::Value::Mail),
                    "daemon" => Ok(Self::Value::Daemon),
                    "auth" => Ok(Self::Value::Auth),
                    "syslog" => Ok(Self::Value::Syslog),
                    "lpr" => Ok(Self::Value::Lpr),
                    "news" => Ok(Self::Value::News),
                    "uucp" => Ok(Self::Value::Uucp),
                    "cron" => Ok(Self::Value::Cron),
                    "authpriv" => Ok(Self::Value::AuthPriv),
                    "ftp" => Ok(Self::Value::Ftp),
                    "local0" => Ok(Self::Value::Local0),
                    "local1" => Ok(Self::Value::Local1),
                    "local2" => Ok(Self::Value::Local2),
                    "local3" => Ok(Self::Value::Local3),
                    "local4" => Ok(Self::Value::Local4),
                    "local5" => Ok(Self::Value::Local5),
                    "local6" => Ok(Self::Value::Local6),
                    "local7" => Ok(Self::Value::Local7),
                    _ => Err(de::Error::invalid_value(
                        Unexpected::Str(value),
                        &"Unknown syslog facility!",
                    )),
                }
            }
        }

        deserializer.deserialize_enum(
            "SyslogFacility",
            &[
                "kern", "user", "mail", "daemon", "auth", "syslog", "lpr", "news", "uucp", "cron",
                "authpriv", "ftp", "local0", "local1", "local2", "local3", "local4", "local5",
                "local6", "local7",
            ],
            SyslogFacilityVisitor,
        )
    }
}

/// The network namespace of the application's configuration. It's here where the
/// hostname and the port used for instance.
#[derive(Clone, Debug, Deserialize)]
pub struct Network {
    /// The hostname that the application should use to open its sockets.
    #[serde(default = "Network::default_hostname")]
    pub hostname: IpAddr,

    /// The port to use to open the application's sockets.
    #[serde(default = "Network::default_port")]
    pub port: u16,
}

impl Network {
    fn default_hostname() -> IpAddr {
        DEFAULT_LISTENER_HOSTNAME
    }

    fn default_port() -> u16 {
        DEFAULT_LISTENER_PORT
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            hostname: Self::default_hostname(),
            port: Self::default_port(),
        }
    }
}

/// The queue namespace of the application's configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Queue;
