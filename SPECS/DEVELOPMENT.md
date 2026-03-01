# 开发规范 (Development Specification)

**项目名称**: 全双工音频处理程序 (trans)  
**版本**: 0.1.0  
**最后更新**: 2026-03-01

---

## 目录

1. [TDD 开发流程](#1-tdd-开发流程)
2. [测试规范](#2-测试规范)
3. [代码规范](#3-代码规范)
4. [提交规范](#4-提交规范)
5. [构建和验证](#5-构建和验证)
6. [开发环境](#6-开发环境)

---

## 1. TDD 开发流程

### 1.1 核心原则

**测试先行 (Test-First)**: 所有功能开发必须遵循 TDD 流程，禁止先写实现后补测试。

### 1.2 Red-Green-Refactor 循环

```
┌─────────────────────────────────────────────────────────────┐
│                    TDD 开发循环                              │
│                                                              │
│   ┌──────────┐     ┌──────────┐     ┌──────────┐           │
│   │   RED    │ ──→ │  GREEN   │ ──→ │ REFACTOR │           │
│   │ 写失败测试│     │ 写实现代码│     │ 重构优化   │           │
│   └──────────┘     └──────────┘     └──────────┘           │
│        ↑                                      │             │
│        │                                      │             │
│        └──────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 开发步骤

#### 步骤 1: RED - 编写失败的测试

```rust
// 1. 先写测试，定义期望行为
#[test]
fn test_gain_processor_amplifies_audio() {
    let mut processor = GainProcessor::new(2.0);
    let mut buffer = vec![0.5, -0.5, 0.3];
    
    processor.process(&mut buffer).unwrap();
    
    // 期望：音频被放大 2 倍
    assert!((buffer[0] - 1.0).abs() < 0.001);
    assert!((buffer[1] - (-1.0)).abs() < 0.001);
    assert!((buffer[2] - 0.6).abs() < 0.001);
}
```

**要求**:
- 测试必须编译失败或运行失败
- 测试应该只测试一个行为
- 测试命名应该描述期望行为

#### 步骤 2: GREEN - 编写最小实现

```rust
// 2. 编写刚好能让测试通过的代码
pub struct GainProcessor {
    gain: f32,
}

impl GainProcessor {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

impl AudioProcessor for GainProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for sample in buffer.iter_mut() {
            *sample = *sample * self.gain;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "音量增益处理器"
    }
}
```

**要求**:
- 只实现让测试通过所需的最小代码
- 不要过度设计
- 允许代码不够优雅

#### 步骤 3: REFACTOR - 重构优化

```rust
// 3. 重构代码，保持测试通过
impl AudioProcessor for GainProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for sample in buffer.iter_mut() {
            // 添加限幅保护，防止削波
            *sample = (*sample * self.gain).clamp(-1.0, 1.0);
        }
        Ok(())
    }
    // ...
}
```

**要求**:
- 重构后必须运行所有相关测试
- 消除重复代码
- 提高代码可读性
- 不改变外部行为

### 1.4 开发检查清单

在开始实现任何功能前，必须完成：

- [ ] 已编写测试用例
- [ ] 测试用例编译失败或运行失败（RED）
- [ ] 测试覆盖了正常路径
- [ ] 测试覆盖了边界条件
- [ ] 测试覆盖了错误处理

在实现完成后，必须完成：

- [ ] 所有测试通过（GREEN）
- [ ] 代码已重构（REFACTOR）
- [ ] 没有重复代码
- [ ] 代码符合命名规范
- [ ] 已添加必要的文档注释

---

## 2. 测试规范

### 2.1 测试文件组织

```
src/
├── audio_io.rs
├── audio_io_tests.rs    # 音频 I/O 测试
├── processor.rs
├── processor_tests.rs  # 处理器测试
├── config.rs
└── config_tests.rs     # 配置测试
```

或在对应模块内使用 `#[cfg(test)]`:

```rust
// processor.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxx() {
        // ...
    }
}
```

### 2.2 测试命名规范

```rust
// 格式：test_<功能>_<场景>_<期望结果>

#[test]
fn test_gain_processor_amplifies_audio() {}

#[test]
fn test_gain_processor_clips_values_above_1() {}

#[test]
fn test_noise_gate_mutes_below_threshold() {}

#[test]
fn test_processor_chain_executes_in_order() {}
```

### 2.3 测试覆盖率要求

**最低覆盖率要求**:

| 模块 | 行覆盖率 | 分支覆盖率 |
|------|----------|------------|
| `processor` | 100% | 100% |
| `config` | 90% | 90% |
| `audio_io` | 80% | 75% |
| `main` | 70% | 60% |

**检查命令**:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 2.4 单元测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 测试处理器基本功能
    #[test]
    fn test_passthrough_processor_does_not_modify_buffer() {
        // Arrange
        let mut processor = PassThroughProcessor;
        let mut buffer = vec![0.5, -0.3, 0.8];
        let expected = buffer.clone();
        
        // Act
        let result = processor.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(buffer, expected);
    }

    // 测试边界条件
    #[test]
    fn test_gain_processor_clips_values_above_1() {
        // Arrange
        let mut processor = GainProcessor::new(3.0);
        let mut buffer = vec![0.5];  // 0.5 * 3 = 1.5 → 应该被限制到 1.0
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        assert!((buffer[0] - 1.0).abs() < 0.001);
    }

    // 测试空输入
    #[test]
    fn test_processor_handles_empty_buffer() {
        // Arrange
        let mut processor = PassThroughProcessor;
        let mut buffer: Vec<f32> = vec![];
        
        // Act
        let result = processor.process(&mut buffer);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 2.5 集成测试规范

```rust
// tests/integration_test.rs

use trans::{AudioConfig, ProcessorChain, GainProcessor};

#[test]
fn test_full_audio_pipeline() {
    // 测试完整的音频处理流程
    // ...
}

#[test]
fn test_config_load_and_save() {
    // 测试配置的加载和保存
    // ...
}
```

### 2.6 测试辅助函数

```rust
// 测试辅助：生成测试音频数据
fn generate_test_tone(frequency: f32, sample_rate: u32, duration: usize) -> Vec<f32> {
    (0..duration)
        .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate).sin())
        .collect()
}

// 测试辅助：比较两个浮点数组
fn assert_float_eq(actual: &[f32], expected: &[f32], epsilon: f32) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!((a - e).abs() < epsilon, "期望: {}, 实际: {}", e, a);
    }
}
```

---

## 3. 代码规范

### 3.1 Rust 代码风格

遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) 和 `rustfmt` 默认配置。

**格式化命令**:
```bash
cargo fmt
```

### 3.2 命名约定

```rust
// 类型：PascalCase
pub struct AudioConfig;
pub trait AudioProcessor;
pub enum Commands;

// 函数和变量：snake_case
pub fn load_or_default() -> Result<Self>;
let sample_rate = 48000;

// 常量：SCREAMING_SNAKE_CASE
const DEFAULT_SAMPLE_RATE: u32 = 48000;

// 泛型类型参数：PascalCase
fn process<T: AudioProcessor>(processor: &mut T) {}

// 测试模块：snake_case + tests 后缀
#[cfg(test)]
mod tests {
    // ...
}
```

### 3.3 函数设计

```rust
// ✅ 好的设计：函数短小，职责单一
pub fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
    for processor in &mut self.processors {
        processor.process(buffer)?;
    }
    Ok(())
}

// ❌ 不好的设计：函数过长，职责过多
pub fn process_and_save_and_log(&mut self, buffer: &mut [f32]) {
    // 处理...
    // 保存...
    // 日志...
}
```

**要求**:
- 函数长度不超过 50 行
- 每个函数只做一件事
- 参数不超过 5 个

### 3.4 错误处理

```rust
use anyhow::{Context, Result};

// ✅ 使用 Context 提供错误上下文
pub fn load_config() -> Result<Self> {
    let content = std::fs::read_to_string("config.toml")
        .context("读取配置文件失败")?;
    
    let config: AudioConfig = toml::from_str(&content)
        .context("解析配置文件失败")?;
    
    Ok(config)
}

// ✅ 使用 ? 操作符传播错误
pub fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
    for processor in &mut self.processors {
        processor.process(buffer)?;  // 传播错误
    }
    Ok(())
}

// ❌ 避免 unwrap()
let config = load_config().unwrap();  // 可能 panic

// ✅ 使用 ? 或 expect()
let config = load_config()?;
let config = load_config().expect("配置加载失败，请检查 config.toml");
```

### 3.5 文档注释

```rust
/// 音频处理器接口
/// 
/// 所有音频处理器必须实现此 trait。处理器在音频回调线程中执行，
/// 因此必须是线程安全的（Send + Sync）。
/// 
/// # 示例
/// 
/// ```
/// use trans::processor::{AudioProcessor, PassThroughProcessor};
/// 
/// let mut processor = PassThroughProcessor;
/// let mut buffer = vec![0.5, -0.5];
/// processor.process(&mut buffer).unwrap();
/// ```
/// 
/// # 注意事项
/// 
/// - `process` 方法必须是实时的，不能阻塞或分配内存
/// - 处理器应该原地修改 buffer，避免额外分配
pub trait AudioProcessor: Send + Sync {
    /// 处理音频数据，原地修改 buffer
    /// 
    /// # 参数
    /// 
    /// * `buffer` - 音频数据缓冲区，f32 数组，范围 [-1.0, 1.0]
    /// 
    /// # 返回
    /// 
    /// * `Ok(())` - 处理成功
    /// * `Err(e)` - 处理失败
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;

    /// 获取处理器名称，用于日志和调试
    fn name(&self) -> &str;
}
```

### 3.6 日志规范

```rust
use log::{debug, info, warn, error};

// 信息级别：正常流程
info!("音频流已启动：{} -> {}", input_name, output_name);

// 调试级别：详细调试信息
debug!("处理音频缓冲区，大小：{}", buffer.len());

// 警告级别：可恢复的问题
warn!("未找到设备 '{}'，使用默认设备", name);

// 错误级别：严重问题
error!("处理音频数据失败：{}", e);
```

---

## 4. 提交规范

### 4.1 Commit Message 格式

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 4.2 Type 类型

| Type | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更新 |
| `style` | 代码格式（不影响代码运行） |
| `refactor` | 重构（不是新功能或修复） |
| `test` | 添加或修改测试 |
| `chore` | 构建过程或辅助工具变动 |

### 4.3 提交示例

```bash
# 新功能
git commit -m "feat(processor): 添加低通滤波器处理器"

# Bug 修复
git commit -m "fix(audio_io): 修复缓冲区溢出问题"

# 测试
git commit -m "test(processor): 为 GainProcessor 添加边界测试"

# 重构
git commit -m "refactor(config): 简化配置加载逻辑"

# 文档
git commit -m "docs(README): 更新安装说明"
```

### 4.4 分支管理

```
main          - 主分支，稳定版本
develop       - 开发分支
feature/*     - 功能分支
bugfix/*      - 修复分支
```

**分支命名**:
```bash
git checkout -b feature/add-equalizer-processor
git checkout -b bugfix/fix-buffer-overflow
```

---

## 5. 构建和验证

### 5.1 开发命令

```bash
# 检查编译错误（快速）
cargo check

# 格式化代码
cargo fmt

# 运行所有测试
cargo test

# 运行特定测试
cargo test test_gain_processor

# 生成测试覆盖率报告
cargo tarpaulin --out Html

# 构建开发版本
cargo build

# 构建发布版本（优化）
cargo build --release

# 运行程序
cargo run -- run

# 运行特定二进制文件
cargo run --bin list_devices
```

### 5.2 CI/CD 检查清单

在提交代码前，必须通过以下检查：

```bash
# 1. 代码格式化
cargo fmt -- --check

# 2. 编译检查
cargo check

# 3. 运行所有测试
cargo test

# 4. 构建发布版本
cargo build --release
```

### 5.3 性能验证

```bash
# 使用 release 模式测试性能
cargo build --release
time ./target/release/trans.exe run

# 监控 CPU 使用率
# Windows: 任务管理器
# Linux: top 或 htop
```

---

## 6. 开发环境

### 6.1 必需工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.93.0+ | 编译环境 |
| cargo |  bundled | 包管理 |
| rustfmt | bundled | 代码格式化 |

### 6.2 推荐工具

| 工具 | 用途 |
|------|------|
| VS Code + rust-analyzer | IDE |
| cargo-watch | 文件变化自动编译 |
| cargo-tarpaulin | 测试覆盖率 |
| cargo-clippy | 代码 lint |

### 6.3 虚拟音频设备

**Windows**:
- VB-Cable A: https://vb-audio.com/Cable/
- VB-Cable B: https://vb-audio.com/Cable/

**安装后验证**:
```bash
trans.exe check
```

### 6.4 IDE 配置

**VS Code settings.json**:
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

---

## 附录

### A. TDD 开发模板

```rust
// 1. 先写测试（RED）
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        // Act
        // Assert
    }
}

// 2. 实现功能（GREEN）
pub fn new_feature() -> Result<()> {
    // 最小实现
}

// 3. 重构优化（REFACTOR）
```

### B. 快速参考

```bash
# TDD 循环
cargo test          # RED: 测试失败
# 写实现...
cargo test          # GREEN: 测试通过
# 重构...
cargo test          # 确保测试仍通过

# 提交前检查
cargo fmt -- --check && cargo check && cargo test && cargo build --release
```

### C. 相关文件

- `SPECS/DESIGN.md` - 设计规范
- `SPECS/PROCESSOR-API.md` - 处理器 API 规范
- `SPECS/TESTING-GUIDE.md` - 测试指南
- `MEMORY.md` - 项目记忆文档
- `README.md` - 项目说明
