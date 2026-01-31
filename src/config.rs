use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub input_device_name: String,
    pub vbcable_input_name: String,
    pub vbcable_output_name: String,
    pub output_device_name: String,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device_name: "麦克风".to_string(),
            vbcable_input_name: "CABLE-A Input".to_string(),
            vbcable_output_name: "CABLE Output".to_string(),
            output_device_name: "扬声器".to_string(),
            sample_rate: 48000,
            buffer_size: 512,
        }
    }
}

impl AudioConfig {
    pub fn load_or_default() -> Result<Self> {
        let config_path = Path::new("config.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .context("读取配置文件失败")?;
            
            let config: AudioConfig = toml::from_str(&content)
                .context("解析配置文件失败")?;
            
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("序列化配置失败")?;
        
        fs::write("config.toml", content)
            .context("写入配置文件失败")?;
        
        Ok(())
    }
}