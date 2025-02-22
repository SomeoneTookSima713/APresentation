#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppConfig {
    pub window_size: (u32, u32),
    pub fullscreen: bool
}

#[derive(Debug, thiserror::Error)]
pub enum AppConfigLoadError {
    #[error("input/output error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("error parsing file contents: {0}")]
    ParsingError(#[from] toml::de::Error)
}

impl AppConfig {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, AppConfigLoadError> {
        let file_contents = std::fs::read_to_string(path)?;

        Ok(toml::from_str(&file_contents)?)
    }
    
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::write(path, toml::to_string(&self).unwrap())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_size: (1280, 720),
            fullscreen: false
        }
    }
}