use super::Result;
use std::{process::Stdio, path::PathBuf};
use tokio::{process::Command, sync::Mutex};

pub struct GhidraServer {
    // We store it in a mutex so that only one request can access
    // the admin script.
    admin_path: Mutex<PathBuf>,
}

impl GhidraServer {
    pub fn from_dir(dir: PathBuf) -> Result<Self> {
        if !dir.exists() {
            return Err("ghidra server directory does not exist.".into());
        }
        let admin_path = dir.join("svrAdmin");
        if !admin_path.exists() {
            return Err("ghidra server admin script does not exist.".into());
        }

        Ok(Self {
            admin_path: Mutex::new(admin_path),
        })
    }

    pub async fn add_user(&self, user: &str) -> Result<()> {
        let path = self.admin_path.lock().await;
        let output = Command::new(&*path)
            .current_dir(path.parent().unwrap())
            .stderr(Stdio::piped())
            .args(&["-add", user])
            .output().await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("{}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}
