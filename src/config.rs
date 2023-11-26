use std::fs::{File, OpenOptions};
use serde::Deserialize;
use std::io::{self, Read, Write};

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_TOML: &str = "[main]\n\
                            verbose = false\n\
                            \n\
                            [source]\n\
                            host = \"0.0.0.0\"\n\
                            port = 8080\n\
                            protocol = \"tcp\"\n\
                            \n\
                            [[server]]\n\
                            host = \"0.0.0.0\"\n\
                            port = 10001\n\
                            \n\
                            [[server]]\n\
                            host = \"0.0.0.0\"\n\
                            port = 10002\n\
                            ";

#[derive(Debug, Deserialize)]
pub struct Config {
    #[allow(dead_code)]
    pub main: Main,
    #[allow(dead_code)]
    pub source: Source,
    #[allow(dead_code)]
    pub server: Vec<Server>,
}

#[derive(Debug, Deserialize)]
pub struct Main {
    pub verbose: bool,
}

#[derive(Debug, Deserialize)]
pub struct Source {
    pub host: String,
    pub port: u16,
    pub protocol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

/* ask on stackoverflow
impl Config {
    pub fn get_src(self) -> bool {
        self::Main::verbose
    }
}
*/

pub fn read_config() -> Config {
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(CONFIG_FILE) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening or creating config file: {e}");
                panic!("No config file");
            }
        };
    let mut c = String::new();
    file.read_to_string(&mut c)
        .expect("Failed to read file");

    let content = if c.is_empty() {
        file.write_all(DEFAULT_TOML.as_bytes())
            .expect("Failed to write config toml ");
        DEFAULT_TOML
    } else {
        &c
    }.to_string();

    let config: Config = toml::from_str(&content)
        .expect("Failed to deserialize");

    config
}
