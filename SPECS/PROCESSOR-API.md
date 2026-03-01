# 音频处理器 API 规范 (Processor API Specification)

**项目名称**: 全双工音频处理程序 (trans)  
**版本**: 0.1.0  
**最后更新**: 2026-03-01

---

## 目录

1. [概述](#1-概述)
2. [AudioProcessor Trait](#2-audioprocessor-trait)
3. [内置处理器](#3-内置处理器)
4. [自定义处理器开发](#4-自定义处理器开发)
5. [处理器链](#5-处理器链)
6. [最佳实践](#6-最佳实践)

---

## 1. 概述

### 1.1 处理器架构

```
音频数据流:
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ 原始音频数据 │───→│ Processor 1  │───→│ Processor 2  │───→│ 处理后音频   │
│  buffer[]    │    │  (Gain)      │    │  (NoiseGate) │    │  buffer[]    │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
```

### 1.2 设计原则

1. **单一职责**: 每个处理器只负责一种音频效果
2. **链式组合**: 多个处理器可以组合成处理器链
3. **实时处理**: 处理器必须在音频回调中实时执行
4. **无状态优先**: 处理器应尽量保持无状态

---

## 2. AudioProcessor Trait

### 2.1 Trait 定义

```rust
use anyhow::Result;

/// 音频处理器接口
/// 
/// 所有音频处理器必须实现此 trait。处理器在音频回调线程中执行，
/// 因此必须是线程安全的（Send + Sync）。
pub trait AudioProcessor: Send + Sync {
    /// 处理音频数据，原地修改 buffer
    /// 
    /// # 参数
    /// * `buffer` - 音频数据缓冲区，f32 数组，范围 [-1.0, 1.0]
    /// 
    /// # 返回
    /// * `Ok(())` - 处理成功
    /// * `Err(e)` - 处理失败
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;

    /// 获取处理器名称，用于日志和调试
    fn name(&self) -> &str;
}
```

### 2.2 约束说明

#### Send + Sync

```rust
// 处理器必须是 Send + Sync，因为它会在音频回调线程中执行
pub trait AudioProcessor: Send + Sync {
    // ...
}
```

**原因**:
- `Send`: 处理器可以被移动到音频线程
- `Sync`: 处理器可以被多个线程共享访问（通过 `Arc<Mutex<>>`）

#### 原地修改

```rust
// ✅ 正确：原地修改 buffer
fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
    for sample in buffer.iter_mut() {
        *sample *= 2.0;
    }
    Ok(())
}

// ❌ 错误：创建新数组，效率低下
fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
    let new_buffer: Vec<f32> = buffer.iter().map(|s| s * 2.0).collect();
    // ... 这样会导致额外的分配
}
```

### 2.3 音频数据格式

```rust
// 音频数据格式
buffer: &mut [f32]

// 样本值范围：[-1.0, 1.0]
// -1.0 = 最小振幅（负向最大）
//  0.0 = 静音
//  1.0 = 最大振幅（正向最大）

// 示例：50% 音量的正弦波
let sample = 0.5 * (2.0 * std::f32::consts::PI * frequency * t).sin();
```

**重要**:
- 所有样本值必须在 `[-1.0, 1.0]` 范围内
- 超出范围会导致削波（clipping）失真
- 处理器应该使用 `.clamp(-1.0, 1.0)` 限制输出

---

## 3. 内置处理器

### 3.1 PassThroughProcessor（直通处理器）

**功能**: 不做任何处理，直接传递音频数据

**用途**: 
- 默认处理器
- 测试和调试
- 作为自定义处理器的基础模板

```rust
pub struct PassThroughProcessor;

impl AudioProcessor for PassThroughProcessor {
    fn process(&mut self, _buffer: &mut [f32]) -> Result<()> {
        // 直通，不做任何处理
        Ok(())
    }

    fn name(&self) -> &str {
        "直通处理器"
    }
}
```

**使用示例**:
```rust
let mut chain = ProcessorChain::new();
chain.add_processor(Box::new(PassThroughProcessor));
```

### 3.2 GainProcessor（音量增益处理器）

**功能**: 放大或衰减音频信号

**参数**:
- `gain: f32` - 增益倍数
  - `gain > 1.0`: 放大
  - `gain = 1.0`: 不变
  - `gain < 1.0`: 衰减
  - `gain = 0.0`: 静音

```rust
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
            // 应用增益并限制范围，防止削波
            *sample = (*sample * self.gain).clamp(-1.0, 1.0);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "音量增益处理器"
    }
}
```

**使用示例**:
```rust
// 放大 2 倍
let mut chain = ProcessorChain::new();
chain.add_processor(Box::new(GainProcessor::new(2.0)));

// 降低到 50%
chain.add_processor(Box::new(GainProcessor::new(0.5)));
```

### 3.3 NoiseGateProcessor（噪音门处理器）

**功能**: 静音低于阈值的音频，消除背景噪音

**参数**:
- `threshold: f32` - 阈值 (0.0-1.0)
  - 低于阈值的音频会被静音
  - 高于阈值的音频保持不变

```rust
pub struct NoiseGateProcessor {
    threshold: f32,
}

impl NoiseGateProcessor {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl AudioProcessor for NoiseGateProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        let threshold_sq = self.threshold * self.threshold;
        for sample in buffer.iter_mut() {
            let sample_val = *sample;
            // 如果样本的平方小于阈值平方，则静音
            if sample_val * sample_val < threshold_sq {
                *sample = 0.0;
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "噪音门处理器"
    }
}
```

**使用示例**:
```rust
// 静音低于 10% 阈值的音频
let mut chain = ProcessorChain::new();
chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
```

---

## 4. 自定义处理器开发

### 4.1 开发模板

```rust
use anyhow::Result;
use trans::processor::AudioProcessor;

/// 自定义处理器名称
pub struct MyCustomProcessor {
    // 处理器参数
    param1: f32,
    param2: f32,
}

impl MyCustomProcessor {
    /// 创建新处理器
    pub fn new(param1: f32, param2: f32) -> Self {
        Self { param1, param2 }
    }
}

impl AudioProcessor for MyCustomProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        // 在这里实现音频处理逻辑
        for sample in buffer.iter_mut() {
            // 修改 sample 值
            *sample = /* 处理逻辑 */;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "自定义处理器名称"
    }
}
```

### 4.2 开发步骤（TDD）

#### 步骤 1: 编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_processor_modifies_audio() {
        // Arrange
        let mut processor = MyCustomProcessor::new(1.0, 0.5);
        let mut buffer = vec![0.5, -0.5, 0.0];
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        // 验证 buffer 被正确修改
        assert_ne!(buffer[0], 0.5);
    }

    #[test]
    fn test_my_processor_clips_output() {
        // Arrange
        let mut processor = MyCustomProcessor::new(3.0, 0.5);
        let mut buffer = vec![0.9];  // 处理后可能超过 1.0
        
        // Act
        processor.process(&mut buffer).unwrap();
        
        // Assert
        // 验证输出被限制在 [-1.0, 1.0]
        assert!(buffer[0] <= 1.0);
        assert!(buffer[0] >= -1.0);
    }
}
```

#### 步骤 2: 实现处理器

```rust
pub struct MyCustomProcessor {
    param1: f32,
    param2: f32,
}

impl MyCustomProcessor {
    pub fn new(param1: f32, param2: f32) -> Self {
        Self { param1, param2 }
    }
}

impl AudioProcessor for MyCustomProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for sample in buffer.iter_mut() {
            // 示例：应用参数处理
            *sample = (*sample * self.param1 + self.param2).clamp(-1.0, 1.0);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "自定义处理器"
    }
}
```

#### 步骤 3: 运行测试

```bash
cargo test test_my_processor
```

### 4.3 示例：压缩器处理器

```rust
/// 音频压缩器 - 降低动态范围
pub struct CompressorProcessor {
    threshold: f32,  // 压缩阈值
    ratio: f32,      // 压缩比
    attack: f32,     // 启动时间（样本数）
    release: f32,    // 释放时间（样本数）
    gain: f32,       // 输出增益
}

impl CompressorProcessor {
    pub fn new(threshold: f32, ratio: f32, attack: f32, release: f32, gain: f32) -> Self {
        Self { threshold, ratio, attack, release, gain }
    }
}

impl AudioProcessor for CompressorProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for sample in buffer.iter_mut() {
            let input_level = sample.abs();
            
            if input_level > self.threshold {
                // 超过阈值，应用压缩
                let excess = input_level - self.threshold;
                let reduced = self.threshold + excess / self.ratio;
                *sample = sample.signum() * reduced * self.gain;
            }
            
            // 限制输出范围
            *sample = sample.clamp(-1.0, 1.0);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "压缩器处理器"
    }
}
```

### 4.4 示例：均衡器处理器

```rust
/// 简单的高通滤波器（高通均衡器）
pub struct HighPassFilterProcessor {
    cutoff_freq: f32,  // 截止频率 (Hz)
    sample_rate: f32,  // 采样率 (Hz)
    prev_in: f32,      // 上一个输入样本
    prev_out: f32,     // 上一个输出样本
}

impl HighPassFilterProcessor {
    pub fn new(cutoff_freq: f32, sample_rate: f32) -> Self {
        Self {
            cutoff_freq,
            sample_rate,
            prev_in: 0.0,
            prev_out: 0.0,
        }
    }
}

impl AudioProcessor for HighPassFilterProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        // 计算滤波器系数
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff_freq);
        let dt = 1.0 / self.sample_rate;
        let alpha = rc / (rc + dt);

        for sample in buffer.iter_mut() {
            // 一阶高通滤波器
            let new_out = alpha * (self.prev_out + *sample - self.prev_in);
            self.prev_in = *sample;
            self.prev_out = new_out;
            *sample = new_out;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "高通滤波器"
    }
}
```

---

## 5. 处理器链

### 5.1 ProcessorChain 结构

```rust
pub struct ProcessorChain {
    processors: Vec<Box<dyn AudioProcessor>>,
}

impl ProcessorChain {
    /// 创建新的处理器链
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// 添加处理器到链尾
    pub fn add_processor(&mut self, processor: Box<dyn AudioProcessor>) {
        self.processors.push(processor);
    }

    /// 执行所有处理器（按添加顺序）
    pub fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for processor in &mut self.processors {
            processor.process(buffer)?;
        }
        Ok(())
    }
}
```

### 5.2 执行顺序

```rust
let mut chain = ProcessorChain::new();
chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));  // 1. 噪音门
chain.add_processor(Box::new(GainProcessor::new(1.5)));       // 2. 增益
chain.add_processor(Box::new(CompressorProcessor::new(/*...*/))); // 3. 压缩

// 执行顺序：噪音门 → 增益 → 压缩
chain.process(&mut buffer)?;
```

### 5.3 使用示例

```rust
use trans::processor::{ProcessorChain, GainProcessor, NoiseGateProcessor};
use std::sync::{Arc, Mutex};

// 创建处理器链
let mut chain = ProcessorChain::new();
chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
chain.add_processor(Box::new(GainProcessor::new(1.5)));

// 包装为线程安全
let processor = Arc::new(Mutex::new(chain));

// 传递给音频流
let stream = AudioStream::create_duplex_stream(
    &input_name,
    &output_name,
    sample_rate,
    buffer_size,
    processor,
    true,
)?;
```

---

## 6. 最佳实践

### 6.1 性能优化

```rust
// ✅ 好的做法：避免在回调中分配内存
impl AudioProcessor for MyProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        // 原地修改，不分配新内存
        for sample in buffer.iter_mut() {
            *sample *= 2.0;
        }
        Ok(())
    }
}

// ❌ 不好的做法：在回调中分配
impl AudioProcessor for MyProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        // 每次都分配新 Vec，效率低下
        let new_data: Vec<f32> = buffer.iter().map(|s| s * 2.0).collect();
        buffer.copy_from_slice(&new_data);
        Ok(())
    }
}
```

### 6.2 状态管理

```rust
// ✅ 无状态处理器（推荐）
pub struct GainProcessor {
    gain: f32,  // 只读参数
}

// ✅ 有状态处理器（需要时）
pub struct DelayProcessor {
    delay_buffer: Vec<f32>,  // 延迟缓冲区
    write_pos: usize,
    read_pos: usize,
}

// ❌ 避免：在 process 中修改参数
pub struct BadProcessor {
    gain: f32,
}

impl AudioProcessor for BadProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        self.gain += 0.1;  // 每次调用都改变参数，不可预测
        // ...
    }
}
```

### 6.3 错误处理

```rust
// ✅ 正确的错误处理
impl AudioProcessor for MyProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());  // 空 buffer 不是错误
        }
        
        // 处理逻辑...
        Ok(())
    }
}

// ❌ 避免：在 process 中 panic
impl AudioProcessor for MyProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        let first = buffer[0];  // 可能 panic，如果 buffer 为空
        // ...
    }
}
```

### 6.4 线程安全

```rust
// ✅ 使用 Arc<Mutex<>> 共享处理器
use std::sync::{Arc, Mutex};

let processor = Arc::new(Mutex::new(ProcessorChain::new()));

// 在音频流中使用
let stream = AudioStream::create_duplex_stream(
    // ...
    processor.clone(),  // 克隆 Arc 引用
    // ...
)?;
```

### 6.5 调试技巧

```rust
// 添加调试输出
use log::debug;

impl AudioProcessor for MyProcessor {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        debug!("处理前：min={}, max={}", 
            buffer.iter().cloned().fold(f32::INFINITY, f32::min),
            buffer.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
        );
        
        // 处理逻辑...
        
        debug!("处理后：min={}, max={}", 
            buffer.iter().cloned().fold(f32::INFINITY, f32::min),
            buffer.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
        );
        
        Ok(())
    }
}
```

---

## 附录

### A. 处理器开发检查清单

在提交新处理器前，确保：

- [ ] 实现了 `AudioProcessor` trait
- [ ] 添加了 `Send + Sync` 约束
- [ ] 编写了单元测试（TDD）
- [ ] 测试覆盖了正常路径
- [ ] 测试覆盖了边界条件
- [ ] 输出被限制在 `[-1.0, 1.0]`
- [ ] 没有在 `process` 中分配内存
- [ ] 添加了文档注释
- [ ] 实现了 `name()` 方法

### B. 常用音频处理算法

| 效果 | 算法 | 复杂度 |
|------|------|--------|
| 增益 | 乘法 | O(n) |
| 噪音门 | 阈值比较 | O(n) |
| 压缩器 | 动态范围控制 | O(n) |
| 限幅器 | 硬/软削波 | O(n) |
| 高通滤波 | IIR/FIR 滤波 | O(n) |
| 低通滤波 | IIR/FIR 滤波 | O(n) |
| 延迟 | 环形缓冲区 | O(n) |
| 混响 | 多延迟线 | O(n*m) |

### C. 相关文件

- `src/processor.rs` - 处理器实现
- `SPECS/DESIGN.md` - 设计规范
- `SPECS/DEVELOPMENT.md` - 开发规范
- `SPECS/TESTING-GUIDE.md` - 测试指南
