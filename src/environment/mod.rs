mod config;

use serde_json::json;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use config::*;

#[derive(Debug)]
pub struct Environment {
    pub project_name: String,
    pub project_uuid: String,

    pub work_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub data_dir: PathBuf,

    pub config: Config,
}

pub type EnvironmentRef = Arc<Environment>;

unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

fn get_work_dir() -> Result<PathBuf> {
    // if work dir is specified in command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cwd = std::env::current_dir()?;
        let custom_path = Path::new(args[1].as_str()).to_path_buf();
        let full_path = cwd.join(custom_path);

        // if custom_path a directory and exists, return it
        if full_path.is_dir() {
            return Ok(full_path);
        }

        // if custom_path is not a directory, return error
        let err = Error::new(
            ErrorKind::Other,
            "Custom path is not a directory or not exists",
        );
        return Err(err);
    }

    // if work dir is not specified in command line arguments,
    // return executable dir path as default work dir
    let executable_path = std::env::current_exe()?;
    // executable parent always exists
    let default_work_dir = executable_path.parent().unwrap().to_path_buf();
    Ok(default_work_dir)
}

fn get_or_create_config(work_dir: &Path) -> Result<Config> {
    let config_path = work_dir.join("tauri-lite.json");
    let config_exists = config_path.exists();

    if !config_exists {
        std::fs::write(
            &config_path,
            json!({
                "name": "tauri-lite-project",
                "uuid": uuid::Uuid::new_v4().to_string(),
            })
            .to_string(),
        )?;
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut config = serde_json::from_str::<Config>(&content)?;

    // if config uuid is not exists, create a new one and write back to config file.
    if config.uuid.is_none() {
        config.uuid = Some(uuid::Uuid::new_v4().to_string());
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())?;
    }

    Ok(config)
}

pub fn init() -> Result<Arc<Environment>> {
    let work_dir = get_work_dir()?;

    let config = get_or_create_config(&work_dir)?;
    let project_name = config.name.clone();
    let project_uuid = config.uuid.clone().unwrap();

    let temp_dir = std::env::temp_dir();

    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| Error::new(ErrorKind::Other, "BaseDirs not found"))?;
    let data_dir = base_dirs
        .data_dir()
        .join(project_name.clone() + "." + &project_uuid);

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)?;
    }

    std::env::set_current_dir(&work_dir)?;

    Ok(Arc::new(Environment {
        project_name,
        project_uuid,
        work_dir,
        temp_dir,
        data_dir,
        config,
    }))
}
