use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub show_home_on_startup: bool,
}

const CONFIG_FILE_NAME: &'static str = "config.json";

pub fn load() -> Config {
    let file = File::open(PathBuf::from(CONFIG_FILE_NAME));
    let config: Config = match file {
        Ok(file) => {
            serde_json::from_reader(file).unwrap()
        }
        Err(_) => {
            Config::default()
        }
    };
    config
}

pub fn save(config_reference: &Config) {
    let content: String = serde_json::to_string(config_reference).unwrap();

    let mut file = File::create(PathBuf::from(CONFIG_FILE_NAME)).unwrap();
    file.write(content.as_bytes()).unwrap();
}
