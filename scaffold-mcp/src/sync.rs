use std::path::PathBuf;
use std::process::{Command, Stdio};

const REPO_URL: &str = "https://github.com/javadbayzavi/aac.git";

pub fn cache_dir() -> PathBuf {
    // Fall back to the temp dir rather than panicking if the home directory
    // can't be determined (e.g. HOME unset). sync then clones there and tools
    // still work, just without a persistent cache across runs.
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".scaffold")
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
            .arg("-C")
            .arg(&cache)
            .args(["pull", "--ff-only"])
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
            .args(["clone", "--depth=1", REPO_URL])
            .arg(&cache)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir_resolves_to_scaffold_repo() {
        // Must not panic regardless of whether a home dir is found.
        let dir = cache_dir();
        assert!(dir.ends_with("repo"));
        assert!(agentic_setup_dir().starts_with(&dir));
    }
}
