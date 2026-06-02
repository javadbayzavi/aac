use std::path::PathBuf;
use std::process::{Command, Stdio};

const REPO_URL: &str = "https://github.com/javadbayzavi/acc.git";

pub fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot find home directory")
        .join(".3t-scaffold")
        .join("repo")
}

pub fn agentic_setup_dir() -> PathBuf {
    cache_dir().join("agentic-setup")
}

/// Clone repo if not present, pull if it is. Returns error message if it fails.
pub fn sync() -> Result<(), String> {
    let cache = cache_dir();

    if cache.join(".git").exists() {
        // Pull latest
        let status = Command::new("git")
            .args(["-C", cache.to_str().unwrap(), "pull", "--ff-only"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("git pull failed: {e}"))?;

        if !status.success() {
            return Err("git pull failed — using cached version".to_string());
        }
    } else {
        // Clone
        std::fs::create_dir_all(&cache).map_err(|e| format!("Failed to create cache dir: {e}"))?;

        let status = Command::new("git")
            .args(["clone", "--depth=1", REPO_URL, cache.to_str().unwrap()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("git clone failed: {e}"))?;

        if !status.success() {
            return Err("git clone failed".to_string());
        }
    }

    Ok(())
}
