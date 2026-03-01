# 设计规范 (Design Specification)

**项目名称**: 全双工音频处理程序 (trans)  
**版本**: 0.1.0  
**最后更新**: 2026-03-01

---

## 目录

1. [系统概述](#1-系统概述)
2. [系统架构](#2-系统架构)
3. [模块设计](#3-模块设计)
4. [接口定义](#4-接口定义)
5. [数据结构设计](#5-数据结构设计)
6. [并发模型](#6-并发模型)
7. [错误处理](#7-错误处理)
8. [配置设计](#8-配置设计)

---

## 1. 系统概述

### 1.1 项目目标

开发一个基于 Rust 的全双工音频处理程序，用于对视频会议/直播软件的输入和输出音频进行实时处理。

### 1.2 核心功能

- **全双工音频处理**: 同时处理麦克风输入和扬声器输出
- **实时低延迟处理**: 支持可配置的缓冲区大小以平衡延迟和 CPU 占用
- **可扩展处理器架构**: 支持动态添加音频处理器
- **智能设备管理**: 自动检测和配置虚拟音频设备

### 1.3 使用场景

- 视频会议软件（Zoom、Teams、腾讯会议等）
- 直播软件（OBS、Streamlabs 等）
- 音频录制和处理

---

## 2. 系统架构

### 2.1 音频流架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        输入流（你说话）                          │
│                                                                  │
│   物理麦克风 ──→ [处理器链] ──→ CABLE-A Input ──→ CABLE-A Output │
│                                                      │           │
│                                                      ↓           │
│                                                会议软件输入       │
├─────────────────────────────────────────────────────────────────┤
│                       输出流（对方说话）                         │
│                                                                  │
│   会议软件输出 ──→ CABLE Output ──→ [处理器链] ──→ 物理扬声器    │
│                                                      │           │
│                                                      ↓           │
│                                                   你听到          │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流图

```
输入流数据流:
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ 物理麦克风    │───→│ AudioStream  │───→│ ProcessorChain│───→│ CABLE-A Input│
│ (InputDevice)│    │ (InputStream)│    │ (process())  │    │ (OutputDevice)│
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                                                              │
                                                              │ 内部管道
                                                              ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ 会议软件输入  │←───│ CABLE-A Output│←──│ crossbeam    │←───│ data_sender  │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘

输出流数据流:
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ 会议软件输出  │───→│ CABLE Output │───→│ AudioStream  │───→│ ProcessorChain│
└──────────────┘    └──────────────┘    │ (InputStream)│    │ (process())  │
                                        └──────────────┘    └──────────────┘
                                                                   │
                                                                   ↓
                                        ┌──────────────┐    ┌──────────────┐
                                        │ 物理扬声器    │←───│ AudioStream  │
                                        │ (OutputDevice)│   │ (OutputStream)│
                                        └──────────────┘    └──────────────┘
```

### 2.3 组件依赖关系

```
                    ┌─────────────┐
                    │   main.rs   │
                    │  (CLI + 协调)│
                    └──────┬──────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
           ↓               ↓               ↓
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │  audio_io.rs │ │ processor.rs │ │  config.rs   │
    │ (音频流管理) │ │ (处理器链)   │ │ (配置管理)   │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           │               │               │
           │               │               │
           ↓               ↓               ↓
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │    cpal      │ │    anyhow    │ │    toml      │
    │ (音频 I/O)   │ │  (错误处理)  │ │ (配置解析)   │
    └──────────────┘ └──────────────┘ └──────────────┘
           │
           ↓
    ┌──────────────┐
    │crossbeam-chan│
    │ (线程通信)   │
    └──────────────┘
```

---

## 3. 模块设计

### 3.1 `audio_io` 模块

**职责**: 管理音频设备的输入输出流

**核心类型**:
- `AudioStream`: 音频流结构体
- `StreamConfig`: 流配置结构体

**主要函数**:
```rust
pub fn create_duplex_stream(
    input_name: &str,
    output_name: &str,
    sample_rate: u32,
    buffer_size: u32,
    processor: Arc<Mutex<ProcessorChain>>,
    is_input_direction: bool,
) -> Result<Self>
```

**依赖**:
- `cpal`: 音频设备访问
- `crossbeam-channel`: 线程间数据传递
- `processor::ProcessorChain`: 音频处理

### 3.2 `processor` 模块

**职责**: 定义音频处理器接口和处理器链

**核心类型**:
- `AudioProcessor` (trait): 音频处理器接口
- `ProcessorChain`: 处理器链
- `PassThroughProcessor`: 直通处理器
- `GainProcessor`: 增益处理器
- `NoiseGateProcessor`: 噪音门处理器

**处理器执行顺序**:
```
原始音频 → Processor 1 → Processor 2 → ... → Processor N → 处理后音频
```

### 3.3 `config` 模块

**职责**: 管理配置文件加载和保存

**核心类型**:
- `AudioConfig`: 音频配置结构体

**主要函数**:
```rust
pub fn load_or_default() -> Result<Self>
pub fn save(&self) -> Result<()>
```

### 3.4 `main.rs`

**职责**: 程序入口、CLI 命令处理、流协调

**CLI 命令**:
- `run`: 运行音频处理程序
- `config`: 交互式配置向导
- `check`: 检查音频设备
- `list-devices`: 列出所有设备
- `device-info`: 显示设备详细信息

---

## 4. 接口定义

### 4.1 `AudioProcessor` Trait

```rust
pub trait AudioProcessor: Send + Sync {
    /// 处理音频数据，原地修改 buffer
    /// 
    /// # 参数
    /// - `buffer`: 音频数据缓冲区，f32 数组，范围 [-1.0, 1.0]
    /// 
    /// # 返回
    /// - `Ok(())`: 处理成功
    /// - `Err(e)`: 处理失败
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;

    /// 获取处理器名称，用于日志和调试
    fn name(&self) -> &str;
}
```

**设计要求**:
1. 所有处理器必须实现 `Send + Sync`，因为它们在音频回调线程中执行
2. `process` 方法必须是实时的，不能阻塞或分配内存
3. 处理器应该原地修改 buffer，避免额外分配

### 4.2 `AudioStream` 公共接口

```rust
impl AudioStream {
    /// 创建全双工音频流
    pub fn create_duplex_stream(
        input_name: &str,
        output_name: &str,
        sample_rate: u32,
        buffer_size: u32,
        processor: Arc<Mutex<ProcessorChain>>,
        is_input_direction: bool,
    ) -> Result<Self>;
}
```

### 4.3 `ProcessorChain` 公共接口

```rust
impl ProcessorChain {
    /// 创建新的处理器链
    pub fn new() -> Self;

    /// 添加处理器到链尾
    pub fn add_processor(&mut self, processor: Box<dyn AudioProcessor>);

    /// 执行所有处理器
    pub fn process(&mut self, buffer: &mut [f32]) -> Result<()>;
}
```

### 4.4 `AudioConfig` 公共接口

```rust
impl AudioConfig {
    /// 从配置文件加载，如果不存在则创建默认配置
    pub fn load_or_default() -> Result<Self>;

    /// 保存配置到文件
    pub fn save(&self) -> Result<()>;
}
```

---

## 5. 数据结构设计

### 5.1 `AudioConfig`

```rust
pub struct AudioConfig {
    /// 物理输入设备名称（麦克风）
    pub input_device_name: String,
    
    /// 虚拟输入设备名称（CABLE-A Input，用于输入流）
    pub vbcable_input_name: String,
    
    /// 虚拟输出设备名称（CABLE Output，用于输出流）
    pub vbcable_output_name: String,
    
    /// 物理输出设备名称（扬声器）
    pub output_device_name: String,
    
    /// 采样率 (Hz)，默认 48000
    pub sample_rate: u32,
    
    /// 缓冲区大小（帧数），默认 512
    pub buffer_size: u32,
}
```

### 5.2 `AudioStream`

```rust
pub struct AudioStream {
    /// 输入设备
    input_device: Device,
    
    /// 输出设备
    output_device: Device,
    
    /// 采样率
    sample_rate: u32,
    
    /// 缓冲区大小
    buffer_size: u32,
    
    /// 输入流
    input_stream: cpal::Stream,
    
    /// 输出流
    output_stream: cpal::Stream,
}
```

### 5.3 `StreamConfig`

```rust
pub struct StreamConfig {
    /// 采样率 (Hz)
    pub sample_rate: u32,
    
    /// 缓冲区大小（帧数）
    pub buffer_size: u32,
}
```

### 5.4 `ProcessorChain`

```rust
pub struct ProcessorChain {
    /// 处理器列表，按执行顺序存储
    processors: Vec<Box<dyn AudioProcessor>>,
}
```

---

## 6. 并发模型

### 6.1 线程架构

```
主线程 (main)
    │
    ├── 配置加载
    ├── 处理器链初始化
    └── 音频流创建
            │
            ↓
    ┌───────────────────┐
    │  cpal 音频线程     │ (由 cpal 库管理)
    │                   │
    │  输入回调线程 ────→│──→ process() ──→ 发送数据
    │                   │
    │  输出回调线程 ←────│←── 接收数据 ──→ 播放
    └───────────────────┘
```

### 6.2 数据传递机制

使用 `crossbeam-channel` 在线程间传递音频数据：

```rust
// 创建有界通道
let (data_sender, data_receiver) = crossbeam_channel::bounded::<Vec<f32>>(1024);

// 输入回调：发送处理后的数据
data_sender.send(buffer)?;

// 输出回调：接收数据播放
if let Ok(buffer) = data_receiver.try_recv() {
    data[..copy_len].copy_from_slice(&buffer[..copy_len]);
}
```

### 6.3 同步机制

- `Arc<Mutex<ProcessorChain>>`: 处理器链的线程安全共享
- `crossbeam-channel`: 无锁通道用于音频数据传递
- 音频回调中避免阻塞操作

### 6.4 线程安全要求

1. 所有 `AudioProcessor` 实现必须是 `Send + Sync`
2. 处理器状态修改必须通过 `Mutex` 保护
3. 音频回调中不能使用阻塞操作
4. 通道操作使用 `try_recv` 避免阻塞

---

## 7. 错误处理

### 7.1 错误类型

使用 `anyhow::Result` 作为统一错误类型：

```rust
use anyhow::{Context, Result};
```

### 7.2 错误处理策略

```rust
// 设备查找失败
Err(anyhow::anyhow!("未找到音频设备：{}", name))

// 配置加载失败
.context("读取配置文件失败")?

// 音频流创建失败
.context("获取输入设备支持配置失败")?
```

### 7.3 错误日志

```rust
use log::{error, warn, info};

// 错误级别
error!("处理音频数据失败：{}", e);

// 警告级别
warn!("未找到包含 '{}' 的设备", name);

// 信息级别
info!("音频流已启动：{} -> {}", input_name, output_name);
```

---

## 8. 配置设计

### 8.1 配置文件格式

使用 TOML 格式：

```toml
# 音频设备配置
input_device_name = "麦克风"
vbcable_input_name = "CABLE-A Input"
vbcable_output_name = "CABLE Output"
output_device_name = "扬声器"

# 音频参数
sample_rate = 48000
buffer_size = 512
```

### 8.2 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `input_device_name` | String | "麦克风" | 物理麦克风设备名称 |
| `vbcable_input_name` | String | "CABLE-A Input" | 虚拟输入设备（输入流） |
| `vbcable_output_name` | String | "CABLE Output" | 虚拟输出设备（输出流） |
| `output_device_name` | String | "扬声器" | 物理扬声器设备名称 |
| `sample_rate` | u32 | 48000 | 采样率 (Hz) |
| `buffer_size` | u32 | 512 | 缓冲区大小（帧） |

### 8.3 配置加载流程

```
程序启动
    │
    ↓
检查 config.toml 是否存在
    │
    ├── 存在 ──→ 加载并解析 ──→ 验证配置 ──→ 使用配置
    │
    └── 不存在 ──→ 启动配置向导 ──→ 生成配置 ──→ 保存并加载
```

---

## 附录

### A. 设备命名约定

- **物理设备**: 使用系统设备名称（如"麦克风"、"扬声器"）
- **虚拟设备**: 必须包含 "CABLE" 标识
  - 输入流虚拟设备：建议命名为 "CABLE-A Input/Output"
  - 输出流虚拟设备：建议命名为 "CABLE Input/Output"

### B. 性能参数建议

| 场景 | `buffer_size` | `sample_rate` | 说明 |
|------|---------------|---------------|------|
| 低延迟 | 256 | 48000 | 适合实时通话 |
| 平衡 | 512 | 48000 | 默认设置 |
| 低 CPU | 1024 | 44100 | 适合后台运行 |

### C. 相关文件

- `src/main.rs` - 主程序入口
- `src/audio_io.rs` - 音频 I/O 实现
- `src/processor.rs` - 处理器定义
- `src/config.rs` - 配置管理
- `config.toml` - 配置文件
- `config.toml.example` - 配置模板
