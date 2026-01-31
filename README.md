# 全双工音频处理程序

基于 Rust 开发的全双工音频处理工具，支持对视频会议/直播软件的输入和输出音频进行实时处理。

## 功能特性

- ✅ **全双工处理**：同时处理麦克风输入和扬声器输出
- ✅ **实时处理**：低延迟音频流处理
- ✅ **插件架构**：可扩展的音频处理器
- ✅ **智能配置**：交互式配置向导，自动检测设备
- ✅ **系统默认**：默认选中系统当前使用的音频设备
- ✅ **设备过滤**：自动过滤虚拟设备，只显示真实物理设备
- ✅ **多设备支持**：支持多个虚拟音频设备
- ✅ **彩色输出**：清晰的彩色日志和提示

## 系统要求

- **操作系统**：Windows 10/11
- **虚拟音频设备**：VB-Cable A + VB-Cable B（至少需要 2 个）
  - 下载地址：https://vb-audio.com/Cable/
- **Rust 版本**：1.93.0 或更高（仅开发时需要）

## 快速开始

### 1. 安装虚拟音频设备

从 https://vb-audio.com/Cable/ 下载并安装：
- VB-Cable A
- VB-Cable B

### 2. 编译程序

```bash
cargo build --release
```

### 3. 首次运行（自动配置）

```bash
.\target\release\trans.exe
```

首次运行时会自动启动配置向导，引导你完成设备配置。

### 4. 配置会议软件

在视频会议软件中（如 Zoom、Teams、腾讯会议等）：

- **输入设备（麦克风）**：选择 `CABLE-A Input (VB-Audio Cable A)`
- **输出设备（扬声器）**：选择 `CABLE Output (VB-Audio Virtual Cable)`

## 命令行参考

```bash
# 查看帮助
trans.exe --help

# 运行音频处理程序
trans.exe run
# 或直接运行
trans.exe

# 交互式配置向导
trans.exe config

# 检查音频设备
trans.exe check

# 列出所有音频设备
trans.exe list-devices

# 显示设备详细信息
trans.exe device-info
```

## 工作原理

```
输入流（你说话）:
  物理麦克风 → 程序处理 → CABLE-A Input → CABLE-A Output → 会议软件

输出流（对方说话）:
  会议软件 → CABLE Output → 程序处理 → 物理扬声器 → 你听到
```

## 配置文件

配置文件位于 `config.toml`，包含以下设置：

```toml
# 物理输入设备（麦克风）
input_device_name = "麦克风"

# 虚拟设备 A（用于输入流）
vbcable_input_name = "CABLE-A Input"

# 虚拟设备 Output（用于输出流）
vbcable_output_name = "CABLE Output"

# 物理输出设备（扬声器）
output_device_name = "扬声器"

# 音频参数
sample_rate = 48000  # 采样率 (Hz)
buffer_size = 512    # 缓冲区大小 (帧)
```

## 音频处理器

程序内置了多种音频处理器：

- **PassThroughProcessor**：直通处理器（不做任何处理）
- **GainProcessor**：音量增益处理器
- **NoiseGateProcessor**：噪音门处理器

### 添加自定义处理器

在 `src/processor.rs` 中实现 `AudioProcessor` trait：

```rust
pub trait AudioProcessor: Send + Sync {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;
    fn name(&self) -> &str;
}
```

然后在 `main.rs` 中添加到处理器链：

```rust
let mut processor = ProcessorChain::new();
processor.add_processor(Box::new(GainProcessor::new(1.5)));
```

## 配置向导功能

- ✅ 自动检测虚拟音频设备
- ✅ 智能过滤虚拟设备，只显示真实物理设备
- ✅ 默认选中系统当前使用的音频设备
- ✅ 支持多虚拟设备选择，自动排除已选设备
- ✅ 如果虚拟设备不足，提示用户安装 VB-Cable

## 适用于

- 视频会议软件：Zoom、Teams、腾讯会议、飞书会议等
- 直播软件：OBS、Streamlabs 等
- 音频录制软件
- 任何需要音频处理的应用

## 故障排除

### 没有声音输出

1. 检查虚拟音频设备是否正确安装
2. 运行 `trans.exe check` 检查设备列表
3. 确认会议软件的输入/输出设备设置正确
4. 检查配置文件 `config.toml`

### 延迟过高

尝试减小 `buffer_size` 值：
```toml
buffer_size = 256  # 更低的延迟
```

### 编译错误

确保安装了必要的依赖：
```bash
cargo build --release
```

## 许可证

本项目仅供学习和个人使用。

## 致谢

- [cpal](https://github.com/RustAudio/cpal) - Rust 音频 I/O 库
- [VB-Audio](https://vb-audio.com/) - 虚拟音频设备