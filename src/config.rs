use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::net::IpAddr;
use std::time::Duration;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub stats_interval: Duration,
    pub enable_stats_logger: bool,
    pub http_stats_port: u16,
}

impl AppConfig {
    // The slightly weird type signature here is to facilitate testing and future flexibility
    // Right now we send in an ::std::env::Vars but we can use a vec of tuples for testing
    // And ultimately swap in any iterable key/value stream
    pub fn new<T>(environ: &mut T) -> AppConfig
    where
        for<'a> &'a mut T: IntoIterator<Item = (String, String)>,
    {
        let vars_map: HashMap<std::string::String, std::string::String> =
            HashMap::from_iter(environ.into_iter());
        let listen_addr = vars_map
            .get("LISTEN_IP")
            .unwrap_or(&"127.0.0.1".to_string())
            .parse::<IpAddr>()
            .unwrap();
        let listen_port = vars_map
            .get("LISTEN_PORT")
            .unwrap_or(&"3030".to_string())
            .parse::<u16>()
            .unwrap();
        let db_url = vars_map
            .get("DATABASE_URL")
            .expect("DATABASE_URL must be set")
            .to_string();
        let max_conns = vars_map
            .get("DATABASE_MAX_CONNS")
            .unwrap_or(&"20".to_string())
            .parse::<u32>()
            .unwrap();
        let stats_seconds = vars_map
            .get("STATS_INTERVAL")
            .unwrap_or(&"20".to_string())
            .parse::<u64>()
            .unwrap();
        let stats_interval = Duration::from_secs(stats_seconds);
        let enable_stats_logger = vars_map
            .get("ENABLE_STATS_LOGGER")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let http_stats_port = vars_map
            .get("HTTP_STATS_PORT")
            .unwrap_or(&"3031".to_string())
            .parse::<u16>()
            .unwrap();
        AppConfig {
            db_url,
            max_conns,
            listen_addr,
            listen_port,
            stats_interval,
            enable_stats_logger,
            http_stats_port,
        }
    }
}

#[test]
pub fn test_ipv4_get_config() {
    let expected = AppConfig {
        db_url: "pgsql://user:pass@host/db".to_string(),
        max_conns: 999,
        listen_addr: "5.4.3.2".to_string().parse().unwrap(),
        listen_port: 4321,
        stats_interval: Duration::from_secs(888),
        enable_stats_logger: false,
        http_stats_port: 4322,
    };
    let mut mock_env = HashMap::new();
    mock_env.insert(
        "DATABASE_URL".to_string(),
        "pgsql://user:pass@host/db".to_string(),
    );
    mock_env.insert("DATABASE_MAX_CONNS".to_string(), "999".to_string());
    mock_env.insert("STATS_INTERVAL".to_string(), "888".to_string());
    mock_env.insert("LISTEN_PORT".to_string(), "4321".to_string());
    mock_env.insert("LISTEN_IP".to_string(), "5.4.3.2".to_string());
    mock_env.insert("ENABLE_STATS_LOGGER".to_string(), "false".to_string());
    mock_env.insert("HTTP_STATS_PORT".to_string(), "4322".to_string());
    let config = AppConfig::new(&mut mock_env.into_iter());
    assert_eq!(expected, config);
}

#[test]
pub fn test_ipv6_get_config() {
    let expected = AppConfig {
        db_url: "pgsql://user:pass@host/db".to_string(),
        max_conns: 999,
        listen_addr: "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
            .to_string()
            .parse()
            .unwrap(),
        listen_port: 4321,
        stats_interval: Duration::from_secs(888),
        enable_stats_logger: false,
        http_stats_port: 4322,
    };
    let mut mock_env = HashMap::new();
    mock_env.insert(
        "DATABASE_URL".to_string(),
        "pgsql://user:pass@host/db".to_string(),
    );
    mock_env.insert("DATABASE_MAX_CONNS".to_string(), "999".to_string());
    mock_env.insert("STATS_INTERVAL".to_string(), "888".to_string());
    mock_env.insert("LISTEN_PORT".to_string(), "4321".to_string());
    mock_env.insert(
        "LISTEN_IP".to_string(),
        "2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string(),
    );
    mock_env.insert("ENABLE_STATS_LOGGER".to_string(), "false".to_string());
    mock_env.insert("HTTP_STATS_PORT".to_string(), "4322".to_string());
    let config = AppConfig::new(&mut mock_env.into_iter());
    assert_eq!(expected, config);
}

//TODO Add tests which verify panics on missing config params
