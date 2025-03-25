use std::{
    io::Write,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use log::info;

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum HyperMode {
    Hybrid = 0,
    Override = 1,
}

pub static CONF: LazyLock<Mutex<KeyboardHookConf>> = LazyLock::new(|| Default::default());

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardHookConf {
    pub hyper_mode: HyperMode,
    pub the_key: u16,
    pub use_meh_key: bool,
}

impl Default for KeyboardHookConf {
    fn default() -> Self {
        Self {
            hyper_mode: HyperMode::Hybrid,
            the_key: 0x14,
            use_meh_key: false,
        }
    }
}

pub fn get_config_folder_path() -> PathBuf {
    let mut home_dir_result = std::env::var("USERPROFILE");
    if home_dir_result.is_err() {
        home_dir_result = std::env::var("HOME");
    }
    let path = match home_dir_result {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => PathBuf::new(),
    };
    let path = path.join(PathBuf::from_iter(
        [".config", "hyper-key"].iter(),
    ));
    _ = std::fs::DirBuilder::new().recursive(true).create(&path);
    path
}

fn get_config_file_path() -> PathBuf {
    get_config_folder_path().join("config.json")
}


pub fn read() {
    let path = get_config_file_path();
    let conf = {
        if !path.exists() {
            let conf = KeyboardHookConf::default();
            let conf_str = serde_json::to_string(&conf).unwrap();
            let file_result = std::fs::File::create(path);
            if let Ok(mut file) = file_result {
                _ = file.write_all(conf_str.as_bytes());
            }
            conf
        } else {
            let conf_result = read_conf_from_file(path);
            let conf = match conf_result {
                Ok(conf) => conf,
                Err(_) => KeyboardHookConf::default(),
            };
            conf
        }
    };
    *CONF.lock().unwrap() = conf;
}

fn read_conf_from_file(path: std::path::PathBuf) -> anyhow::Result<KeyboardHookConf> {
    let file = std::fs::File::open(path)?;
    let conf: KeyboardHookConf = serde_json::from_reader(file)?;
    info!("{:?}", &conf);
    anyhow::Ok(conf)
}

pub fn write_conf(conf: KeyboardHookConf, also_set_conf: bool) -> anyhow::Result<()> {
    if also_set_conf {
        match CONF.lock() {
            Ok(mut mut_conf) => *mut_conf = conf,
            Err(err) => anyhow::bail!("Failed to lock conf: {:?}", err),
        }
    }
    let path = get_config_file_path();
    let conf_str = serde_json::to_string_pretty(&conf)?;
    let mut file = if path.exists() {
        std::fs::OpenOptions::new().truncate(true).write(true).open(path)?
    } else {
        std::fs::File::create(path)?
    };

    file.write_all(conf_str.as_bytes())?;

    anyhow::Ok(())
}

pub fn write() -> anyhow::Result<()> {
    let conf_result = CONF.lock();
    if conf_result.is_err() {
        anyhow::bail!("Failed to lock conf")
    }
    let conf = conf_result.unwrap().clone();
    return write_conf(conf, false);
}
