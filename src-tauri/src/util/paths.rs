use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn app_data_dir() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "boswell", "tarcie")
        .ok_or_else(|| anyhow::anyhow!("failed to resolve project dirs"))?;
    Ok(proj.data_local_dir().to_path_buf())
}

pub fn queue_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("queue"))
}

pub fn sent_dir() -> Result<PathBuf> {
    Ok(queue_dir()?.join("sent"))
}

pub fn device_id_path() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("device_id.txt"))
}

pub fn logs_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("logs"))
}
