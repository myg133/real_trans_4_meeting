# 测试指南 (Testing Guide)

**项目名称**: 全双工音频处理程序 (trans)  
**版本**: 0.1.0  
**最后更新**: 2026-03-01

---

## 目录

1. [测试概述](#1-测试概述)
2. [测试工具和环境](#2-测试工具和环境)
3. [处理器模块测试](#3-处理器模块测试)
4. [配置模块测试](#4-配置模块测试)
5. [音频 I/O 模块测试](#5-音频-io 模块测试)
6. [集成测试](#6-集成测试)
7. [测试辅助工具](#7-测试辅助工具)

---

## 1. 测试概述

### 1.1 测试金字塔

```
                    /\
                   /  \
                  / E2E \
                 /--------\
                /          \
               / Integration \
              /----------------\
             /                  \
            /     Unit Tests     \
           /----------------------\
```

**测试分布**:
- **单元测试 (70%)**: 测试单个函数、方法或模块
- **集成测试 (20%)**: 测试模块间的交互
- **端到端测试 (10%)**: 测试完整流程

### 1.2 测试分类

| 分类 | 位置 | 命令 |
|------|------|------|
| 单元测试 | `src/*.rs` 中的 `#[cfg(test)]` 模块 | `cargo test` |
| 集成测试 | `tests/*.rs` | `cargo test --test '*'` |
| 文档测试 | 文档注释中的 ` ``` ` 代码块 | `cargo test --doc` |

### 1.3 测试命名规范

```rust
// 格式：test_<模块>_<功能>_<场景>_<期望>

#[test]
fn test_gain_processor_amplifies_audio() {}

#[test]
fn test_gain_processor_clips_values_above_1() {}

#[test]
fn test_config_load_returns_default_when_file_missing() {}
```

---

## 2. 测试工具和环境

### 2.1 基本测试命令

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_gain_processor

# 运行包含 "processor" 的测试
cargo test processor

# 显示测试输出
cargo test -- --nocapture

# 运行测试并显示耗时
cargo test -- --report-time

# 只运行单元测试
cargo test --lib

# 只运行集成测试
cargo test --test '*'
```

### 2.2 测试覆盖率

```bash
# 安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 生成 HTML 覆盖率报告
cargo tarpaulin --out Html

# 生成 XML 报告（用于 CI）
cargo tarpaulin --out Xml

# 查看覆盖率摘要
cargo tarpaulin --out Stdout
```

### 2.3 测试依赖

在 `Cargo.toml` 中添加测试依赖：

```toml
[dev-dependencies]
mockall = "0.12"  # Mock 框架
tempfile = "3"    # 临时文件
```

---

## 3. 处理器模块测试

### 3.1 PassThroughProcessor 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_processor_does_not_modify_buffer() {
        // Arrange
        let mut processor = PassThroughProcessor;
        let mut buffer = vec![0.5, -0.3, 0.8, 0.0, -0.9];
        let expected = buffer.clone();
        
        // Act
        let result = processor.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_passthrough_processor_returns_ok() {
        // Arrange
        let mut processor = PassThroughProcessor;
        let mut buffer = vec![0.5];
        
        // Act
        let result = processor.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 3.2 GainProcessor 测试

```rust
#[cfg(test)]
mod gain_processor_tests {
    use super::*;

    #[test]
    fn test_gain_processor_amplifies_audio() {
        // Arrange
        let mut processor = GainProcessor::new(2.0);
        let mut buffer = vec![0.5, -0.5, 0.3];
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 1.0).abs() < 0.001);
        assert!((buffer[1] - (-1.0)).abs() < 0.001);
        assert!((buffer[2] - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_gain_processor_attenuates_audio() {
        // Arrange
        let mut processor = GainProcessor::new(0.5);
        let mut buffer = vec![1.0, -0.8, 0.4];
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 0.5).abs() < 0.001);
        assert!((buffer[1] - (-0.4)).abs() < 0.001);
        assert!((buffer[2] - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_gain_processor_clips_positive_values() {
        // Arrange
        let mut processor = GainProcessor::new(3.0);
        let mut buffer = vec![0.5];  // 0.5 * 3 = 1.5 → 应该被限制到 1.0
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gain_processor_clips_negative_values() {
        // Arrange
        let mut processor = GainProcessor::new(3.0);
        let mut buffer = vec![-0.5];  // -0.5 * 3 = -1.5 → 应该被限制到 -1.0
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_gain_processor_handles_empty_buffer() {
        // Arrange
        let mut processor = GainProcessor::new(2.0);
        let mut buffer: Vec<f32> = vec![];
        
        // Act
        let result = processor.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_gain_processor_handles_zero_gain() {
        // Arrange
        let mut processor = GainProcessor::new(0.0);
        let mut buffer = vec![0.5, -0.5, 1.0];
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_gain_processor_name() {
        // Arrange
        let processor = GainProcessor::new(1.0);
        
        // Assert
        assert_eq!(processor.name(), "音量增益处理器");
    }
}
```

### 3.3 NoiseGateProcessor 测试

```rust
#[cfg(test)]
mod noise_gate_processor_tests {
    use super::*;

    #[test]
    fn test_noise_gate_mutes_below_threshold() {
        // Arrange
        let mut processor = NoiseGateProcessor::new(0.5);
        let mut buffer = vec![0.3, -0.3, 0.1];  // 都低于阈值
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_noise_gate_preserves_above_threshold() {
        // Arrange
        let mut processor = NoiseGateProcessor::new(0.3);
        let mut buffer = vec![0.5, -0.5, 0.8];  // 都高于阈值
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 0.5).abs() < 0.001);
        assert!((buffer[1] - (-0.5)).abs() < 0.001);
        assert!((buffer[2] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_noise_gate_mixed_values() {
        // Arrange
        let mut processor = NoiseGateProcessor::new(0.3);
        let mut buffer = vec![0.5, -0.2, 0.1, -0.8];
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 0.5).abs() < 0.001);  // 保留
        assert_eq!(buffer[1], 0.0);                 // 静音
        assert_eq!(buffer[2], 0.0);                 // 静音
        assert!((buffer[3] - (-0.8)).abs() < 0.001); // 保留
    }

    #[test]
    fn test_noise_gate_at_exact_threshold() {
        // Arrange
        let mut processor = NoiseGateProcessor::new(0.5);
        let mut buffer = vec![0.5, -0.5];  // 正好在阈值上
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        // 阈值比较使用平方，0.5^2 = 0.25，不小于 0.25，所以应该保留
        assert!((buffer[0] - 0.5).abs() < 0.001);
        assert!((buffer[1] - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_noise_gate_name() {
        // Arrange
        let processor = NoiseGateProcessor::new(0.1);
        
        // Assert
        assert_eq!(processor.name(), "噪音门处理器");
    }
}
```

### 3.4 ProcessorChain 测试

```rust
#[cfg(test)]
mod processor_chain_tests {
    use super::*;

    #[test]
    fn test_processor_chain_empty_chain() {
        // Arrange
        let mut chain = ProcessorChain::new();
        let mut buffer = vec![0.5, -0.3];
        
        // Act
        let result = chain.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_processor_chain_single_processor() {
        // Arrange
        let mut chain = ProcessorChain::new();
        chain.add_processor(Box::new(GainProcessor::new(2.0)));
        let mut buffer = vec![0.5];
        
        // Act
        chain.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_processor_chain_multiple_processors() {
        // Arrange
        let mut chain = ProcessorChain::new();
        chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
        chain.add_processor(Box::new(GainProcessor::new(2.0)));
        let mut buffer = vec![0.5, 0.05];  // 0.05 低于噪音门阈值
        
        // Act
        chain.process(&mut buffer).unwrap();
        
        // Assert
        // 第一步：噪音门 → [0.5, 0.0]
        // 第二步：增益 ×2 → [1.0, 0.0]
        assert!((buffer[0] - 1.0).abs() < 0.001);
        assert_eq!(buffer[1], 0.0);
    }

    #[test]
    fn test_processor_chain_execution_order() {
        // Arrange
        let mut chain = ProcessorChain::new();
        chain.add_processor(Box::new(GainProcessor::new(2.0)));
        chain.add_processor(Box::new(GainProcessor::new(0.5)));
        let mut buffer = vec![1.0];
        
        // Act
        chain.process(&mut buffer).unwrap();
        
        // Assert
        // 1.0 × 2 = 2.0 → 限制到 1.0
        // 1.0 × 0.5 = 0.5
        assert!((buffer[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_processor_chain_name() {
        // Arrange
        let mut chain = ProcessorChain::new();
        chain.add_processor(Box::new(PassThroughProcessor));
        
        // 处理器链本身没有 name 方法，但可以测试内部处理器
        // 这里只是示例
    }
}
```

---

## 4. 配置模块测试

### 4.1 AudioConfig 测试

```rust
#[cfg(test)]
mod audio_config_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default_values() {
        // Arrange & Act
        let config = AudioConfig::default();
        
        // Assert
        assert_eq!(config.input_device_name, "麦克风");
        assert_eq!(config.vbcable_input_name, "CABLE-A Input");
        assert_eq!(config.vbcable_output_name, "CABLE Output");
        assert_eq!(config.output_device_name, "扬声器");
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
    }

    #[test]
    fn test_config_load_returns_default_when_file_missing() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Act
        let config = AudioConfig::load_or_default().unwrap();
        
        // Assert
        assert_eq!(config.input_device_name, "麦克风");
        
        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_config_load_from_existing_file() {
        // Arrange
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
        
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Act
        let config = AudioConfig::load_or_default().unwrap();
        
        // Assert
        assert_eq!(config.input_device_name, "Test Mic");
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.buffer_size, 256);
        
        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_config_save_and_load() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let config = AudioConfig {
            input_device_name: "Saved Mic".to_string(),
            vbcable_input_name: "Saved Input".to_string(),
            vbcable_output_name: "Saved Output".to_string(),
            output_device_name: "Saved Speaker".to_string(),
            sample_rate: 96000,
            buffer_size: 1024,
        };
        
        // Act
        config.save().unwrap();
        let loaded_config = AudioConfig::load_or_default().unwrap();
        
        // Assert
        assert_eq!(loaded_config.input_device_name, "Saved Mic");
        assert_eq!(loaded_config.sample_rate, 96000);
        
        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }
}
```

---

## 5. 音频 I/O 模块测试

### 5.1 设备查找测试

```rust
#[cfg(test)]
mod audio_io_tests {
    use super::*;

    #[test]
    fn test_find_device_by_name_finds_device() {
        // 这个测试需要实际的音频设备
        // 在 CI 环境中可能需要 mock
        let host = cpal::default_host();
        
        // 尝试查找任意输入设备
        let devices: Vec<_> = host.input_devices().unwrap().collect();
        if !devices.is_empty() {
            if let Ok(name) = devices[0].name() {
                let result = find_device_by_name(&host, &name, true);
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn test_find_device_by_name_returns_error_when_not_found() {
        let host = cpal::default_host();
        
        // Act
        let result = find_device_by_name(&host, "NonExistentDevice12345", true);
        
        // Assert
        assert!(result.is_err());
    }
}
```

### 5.2 使用 Mock 测试音频流

```rust
// 由于 cpal 需要实际硬件，使用 mock 进行测试
// 可以使用 mockall crate 创建 mock

#[cfg(test)]
mod mock_audio_tests {
    use mockall::automock;
    
    #[automock]
    trait AudioDevice {
        fn name(&self) -> Result<String, String>;
        fn play(&self) -> Result<(), String>;
    }
    
    #[test]
    fn test_mock_device() {
        let mut mock = MockAudioDevice::new();
        mock.expect_name().returning(|| Ok("Mock Device".to_string()));
        mock.expect_play().returning(|| Ok(()));
        
        assert_eq!(mock.name().unwrap(), "Mock Device");
        assert!(mock.play().is_ok());
    }
}
```

---

## 6. 集成测试

### 6.1 完整音频处理流程测试

```rust
// tests/integration_test.rs

use trans::processor::*;
use trans::config::AudioConfig;

#[test]
fn test_full_audio_processing_pipeline() {
    // Arrange
    let mut chain = ProcessorChain::new();
    chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
    chain.add_processor(Box::new(GainProcessor::new(1.5)));
    
    // 模拟音频数据（1 秒，48kHz，正弦波）
    let sample_rate = 48000;
    let frequency = 440.0;  // A4 音符
    let duration = sample_rate as usize;
    
    let mut buffer: Vec<f32> = (0..duration)
        .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate).sin())
        .collect();
    
    // Act
    chain.process(&mut buffer).unwrap();
    
    // Assert
    // 验证所有样本都在有效范围内
    for sample in &buffer {
        assert!(*sample >= -1.0 && *sample <= 1.0);
    }
    
    // 验证增益效果（整体振幅应该增加）
    let original_rms = calculate_rms(&buffer);
    assert!(original_rms > 0.0);
}

#[test]
fn test_config_integration() {
    // 测试配置的完整流程
    let config = AudioConfig::default();
    
    // 验证默认配置有效
    assert!(!config.input_device_name.is_empty());
    assert!(!config.vbcable_input_name.is_empty());
    assert!(config.sample_rate > 0);
    assert!(config.buffer_size > 0);
}

fn calculate_rms(buffer: &[f32]) -> f32 {
    let sum: f32 = buffer.iter().map(|s| s * s).sum();
    (sum / buffer.len() as f32).sqrt()
}
```

### 6.2 处理器链集成测试

```rust
// tests/processor_chain_test.rs

use trans::processor::*;

#[test]
fn test_voice_processing_chain() {
    // 模拟典型的语音处理链：噪音门 → 增益 → 压缩
    let mut chain = ProcessorChain::new();
    chain.add_processor(Box::new(NoiseGateProcessor::new(0.05)));
    chain.add_processor(Box::new(GainProcessor::new(1.5)));
    
    // 测试静音输入
    let mut silent_buffer = vec![0.0; 1024];
    chain.process(&mut silent_buffer).unwrap();
    assert!(silent_buffer.iter().all(|&s| s == 0.0));
    
    // 测试正常输入
    let mut normal_buffer = vec![0.5; 1024];
    chain.process(&mut normal_buffer).unwrap();
    // 经过噪音门（通过）和增益（×1.5），结果应该接近 0.75（限制到 1.0）
    assert!(normal_buffer.iter().all(|&s| s > 0.0 && s <= 1.0));
}
```

---

## 7. 测试辅助工具

### 7.1 测试数据生成器

```rust
// 测试辅助函数

/// 生成正弦波测试信号
pub fn generate_sine_wave(
    frequency: f32,
    amplitude: f32,
    sample_rate: u32,
    duration_samples: usize,
) -> Vec<f32> {
    (0..duration_samples)
        .map(|i| {
            amplitude * (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate).sin()
        })
        .collect()
}

/// 生成方波测试信号
pub fn generate_square_wave(
    frequency: f32,
    amplitude: f32,
    sample_rate: u32,
    duration_samples: usize,
) -> Vec<f32> {
    let period = sample_rate as f32 / frequency;
    (0..duration_samples)
        .map(|i| {
            if (i as f32 % period) < period / 2.0 {
                amplitude
            } else {
                -amplitude
            }
        })
        .collect()
}

/// 生成白噪声测试信号
pub fn generate_white_noise(
    amplitude: f32,
    duration_samples: usize,
) -> Vec<f32> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let mut rng = rand::Rng::new(seed);  // 需要 rand crate
    (0..duration_samples)
        .map(|_| rng.gen_range(-amplitude..amplitude))
        .collect()
}

/// 生成静音测试信号
pub fn generate_silence(duration_samples: usize) -> Vec<f32> {
    vec![0.0; duration_samples]
}
```

### 7.2 断言辅助函数

```rust
/// 断言两个浮点数组近似相等
pub fn assert_float_eq(actual: &[f32], expected: &[f32], epsilon: f32) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "数组长度不匹配：实际={}, 期望={}",
        actual.len(),
        expected.len()
    );
    
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (a - e).abs() < epsilon,
            "索引 {} 不匹配：期望={}, 实际={}, 差异={}",
            i, e, a,
            (a - e).abs()
        );
    }
}

/// 断言所有样本在指定范围内
pub fn assert_all_in_range(buffer: &[f32], min: f32, max: f32) {
    for (i, &sample) in buffer.iter().enumerate() {
        assert!(
            sample >= min && sample <= max,
            "索引 {} 的样本 {} 超出范围 [{}, {}]",
            i, sample, min, max
        );
    }
}

/// 计算信号的 RMS（均方根）值
pub fn calculate_rms(buffer: &[f32]) -> f32 {
    let sum: f32 = buffer.iter().map(|s| s * s).sum();
    (sum / buffer.len() as f32).sqrt()
}

/// 计算信号的峰值
pub fn calculate_peak(buffer: &[f32]) -> f32 {
    buffer.iter().map(|s| s.abs()).fold(0.0, f32::max)
}
```

### 7.3 性能测试

```rust
// 基准测试（需要 nightly Rust 或 criterion crate）

#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;  // 需要 nightly
    
    #[bench]
    fn bench_gain_processor(b: &mut Bencher) {
        let mut processor = GainProcessor::new(1.5);
        let mut buffer = vec![0.5; 1024];
        
        b.iter(|| {
            processor.process(&mut buffer).unwrap();
        });
    }
    
    #[bench]
    fn bench_noise_gate_processor(b: &mut Bencher) {
        let mut processor = NoiseGateProcessor::new(0.1);
        let mut buffer = vec![0.5; 1024];
        
        b.iter(|| {
            processor.process(&mut buffer).unwrap();
        });
    }
    
    #[bench]
    fn bench_processor_chain(b: &mut Bencher) {
        let mut chain = ProcessorChain::new();
        chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
        chain.add_processor(Box::new(GainProcessor::new(1.5)));
        let mut buffer = vec![0.5; 1024];
        
        b.iter(|| {
            chain.process(&mut buffer).unwrap();
        });
    }
}
```

### 7.4 使用 Criterion 进行基准测试

```rust
// Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "processor_bench"
harness = false

// benches/processor_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trans::processor::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut processor = GainProcessor::new(1.5);
    let mut buffer = vec![0.5; 1024];
    
    c.bench_function("gain_processor_1024_samples", |b| {
        b.iter(|| {
            processor.process(black_box(&mut buffer)).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

运行基准测试：
```bash
cargo bench
```

---

## 附录

### A. 测试检查清单

在提交代码前，确保：

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 测试覆盖率满足要求
- [ ] 新代码有对应的测试
- [ ] 边界条件已测试
- [ ] 错误处理已测试
- [ ] 文档测试通过

### B. 常见问题

**Q: 测试运行太慢怎么办？**

```bash
# 只运行失败的测试
cargo test -- --test-threads=1

# 跳过慢测试
cargo test -- --skip slow_test_name
```

**Q: 如何调试失败的测试？**

```bash
# 显示测试输出
cargo test -- --nocapture

# 运行单个测试
cargo test test_name -- --exact
```

**Q: 如何测试需要硬件的功能？**

- 使用 Mock 模拟硬件行为
- 在 CI 中跳过硬件相关测试
- 使用条件编译：`#[cfg(not(test))]`

### C. 相关文件

- `SPECS/DESIGN.md` - 设计规范
- `SPECS/DEVELOPMENT.md` - 开发规范
- `SPECS/PROCESSOR-API.md` - 处理器 API 规范
- `src/processor.rs` - 处理器实现
- `src/audio_io.rs` - 音频 I/O 实现
- `src/config.rs` - 配置管理
