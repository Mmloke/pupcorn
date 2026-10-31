use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::error::Result;

#[async_trait]
pub trait Screenshotter: Send + Sync {
    /// Capture a screenshot of `url` into `out_path`. Returns Ok(path) on success.
    async fn capture(&self, url: &str, out_path: &Path) -> Result<String>;
}

/// No-op screenshotter (used when no browser is available).
pub struct NoopScreenshotter;
#[async_trait]
impl Screenshotter for NoopScreenshotter {
    async fn capture(&self, _url: &str, _out_path: &Path) -> Result<String> {
        Err(crate::error::PupcornError::Other("no screenshot backend".into()))
    }
}

/// Firefox headless screenshot backend (shells out to firefox).
pub struct FirefoxHeadless {
    pub profile_dir: String,
}
#[async_trait]
impl Screenshotter for FirefoxHeadless {
    async fn capture(&self, url: &str, out_path: &Path) -> Result<String> {
        let status = Command::new("firefox")
            .arg("--headless")
            .arg("--profile")
            .arg(&self.profile_dir)
            .arg("--screenshot")
            .arg(out_path)
            .arg(url)
            .status()
            .map_err(crate::error::PupcornError::Io)?;
        if !status.success() {
            return Err(crate::error::PupcornError::Other(format!(
                "firefox screenshot failed: {}",
                status
            )));
        }
        Ok(out_path.to_string_lossy().to_string())
    }
}

/// Convenience: pick a backend based on availability.
pub fn default_screenshotter() -> Box<dyn Screenshotter> {
    // Prefer Firefox if present.
    if Command::new("firefox").arg("--version").output().is_ok() {
        Box::new(FirefoxHeadless {
            profile_dir: "/tmp/pupcorn-ff-profile".to_string(),
        })
    } else {
        Box::new(NoopScreenshotter)
    }
}