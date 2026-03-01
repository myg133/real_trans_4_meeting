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
            let content = fs::read_to_string(config_path).context("读取配置文件失败")?;

            let config: AudioConfig = toml::from_str(&content).context("解析配置文件失败")?;

            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("序列化配置失败")?;

        fs::write("config.toml", content).context("写入配置文件失败")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_values() {
        let config = AudioConfig::default();

        assert_eq!(config.input_device_name, "麦克风");
        assert_eq!(config.vbcable_input_name, "CABLE-A Input");
        assert_eq!(config.vbcable_output_name, "CABLE Output");
        assert_eq!(config.output_device_name, "扬声器");
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
    }

    #[test]
    #[serial]
    fn test_load_or_default_creates_default_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let config = AudioConfig::load_or_default().unwrap();

        assert_eq!(config.input_device_name, "麦克风");
        assert!(temp_dir.path().join("config.toml").exists());

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_load_or_default_loads_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
input_device_name = "Test Mic"
vbcable_input_name = "Test Input"
vbcable_output_name = "Test Output"
output_device_name = "Test Speaker"
sample_rate = 44100
buffer_size = 256
"#;
        fs::write(&config_path, config_content).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let config = AudioConfig::load_or_default().unwrap();

        assert_eq!(config.input_device_name, "Test Mic");
        assert_eq!(config.vbcable_input_name, "Test Input");
        assert_eq!(config.vbcable_output_name, "Test Output");
        assert_eq!(config.output_device_name, "Test Speaker");
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.buffer_size, 256);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_save_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let config = AudioConfig {
            input_device_name: "My Mic".to_string(),
            vbcable_input_name: "CABLE-A Input".to_string(),
            vbcable_output_name: "CABLE Output".to_string(),
            output_device_name: "My Speaker".to_string(),
            sample_rate: 96000,
            buffer_size: 128,
        };

        config.save().unwrap();

        let saved_content = fs::read_to_string("config.toml").unwrap();
        assert!(saved_content.contains("My Mic"));
        assert!(saved_content.contains("96000"));
        assert!(saved_content.contains("128"));

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let original = AudioConfig {
            input_device_name: "Custom Mic".to_string(),
            vbcable_input_name: "VB-A Input".to_string(),
            vbcable_output_name: "VB-A Output".to_string(),
            output_device_name: "Custom Speaker".to_string(),
            sample_rate: 44100,
            buffer_size: 256,
        };

        original.save().unwrap();
        let loaded = AudioConfig::load_or_default().unwrap();

        assert_eq!(loaded.input_device_name, "Custom Mic");
        assert_eq!(loaded.vbcable_input_name, "VB-A Input");
        assert_eq!(loaded.vbcable_output_name, "VB-A Output");
        assert_eq!(loaded.output_device_name, "Custom Speaker");
        assert_eq!(loaded.sample_rate, 44100);
        assert_eq!(loaded.buffer_size, 256);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_clone() {
        let config = AudioConfig::default();
        let cloned = config.clone();

        assert_eq!(config.input_device_name, cloned.input_device_name);
        assert_eq!(config.sample_rate, cloned.sample_rate);
    }
}
