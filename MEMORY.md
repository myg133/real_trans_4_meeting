# 项目记忆文档

## 项目概述

**项目名称**：全双工音频处理程序  
**开发语言**：Rust  
**主要功能**：对视频会议/直播软件的输入和输出音频进行实时处理

## 核心架构

### 音频流架构

```
┌─────────────────────────────────────────────────────────────────┐
│ 输入流（你说话）:                                                │
│   物理麦克风 → 处理器链 → CABLE-A Input → CABLE-A Output → 会   │
│   议软件                                                         │
│                                                                 │
│ 输出流（对方说话）:                                              │
│   会议软件 → CABLE Output → 处理器链 → 物理扬声器 → 你听到       │
└─────────────────────────────────────────────────────────────────┘
```

### 代码结构

```
src/
├── main.rs              # 主程序入口，CLI 命令处理
├── audio_io.rs          # 音频输入输出处理
├── processor.rs         # 音频处理器定义
├── config.rs            # 配置文件管理
└── bin/
    ├── list_devices.rs  # 列出设备工具
    ├── device_info.rs   # 设备详细信息工具
    └── check_config.rs  # 配置检查工具
```

## 关键技术点

### 1. 音频流处理

**输入流**：
- 从物理麦克风读取音频数据
- 通过处理器链处理
- 写入 CABLE-A Input（虚拟扬声器）
- 通过内部管道传到 CABLE-A Output（虚拟麦克风）
- 会议软件从 CABLE-A Output 读取

**输出流**：
- 从 CABLE Output（虚拟扬声器）读取
- 通过处理器链处理
- 写入物理扬声器
- 用户听到处理后的声音

### 2. 处理器架构

**AudioProcessor trait**：
```rust
pub trait AudioProcessor: Send + Sync {
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;
    fn name(&self) -> &str;
}
```

**ProcessorChain**：
- 管理多个处理器的链式执行
- 按顺序处理音频数据
- 支持动态添加处理器

### 3. 设备管理

**设备检测**：
- 自动检测虚拟音频设备
- 过滤虚拟设备，只显示真实物理设备
- 检测系统默认音频设备

**设备选择**：
- 交互式选择界面
- 默认选中系统默认设备
- 支持多虚拟设备选择
- 自动排除已选设备

### 4. 配置管理

**配置文件**：`config.toml`

**配置向导**：
- 首次运行自动启动
- 检测虚拟设备数量
- 智能设备推荐
- 自动生成配置文件

## 依赖库

- **cpal**：音频 I/O 处理
- **crossbeam-channel**：跨线程音频数据传递
- **anyhow**：错误处理
- **serde** + **toml**：配置文件序列化
- **clap**：命令行参数解析
- **dialoguer**：交互式配置界面
- **env_logger** + **log**：日志记录
- **colored**：彩色输出

## 已知问题和解决方案

### 1. CABLE 设备命名混淆

**问题**：CABLE Input 在输出设备列表中，CABLE Output 在输入设备列表中

**原因**：VB-Audio Virtual Cable 的工作原理
- CABLE Input：虚拟扬声器（输出设备）
- CABLE Output：虚拟麦克风（输入设备）

**解决方案**：
- 在配置文件中明确说明每个设备的作用
- 在日志中清晰标注设备用途
- 使用彩色高亮显示重要信息

### 2. 单向虚拟设备限制

**问题**：单个 VB-Cable 只能单向传输

**解决方案**：
- 需要至少 2 个独立的 VB-Cable
- 使用 CABLE-A 用于输入流
- 使用 CABLE 用于输出流
- 配置向导会检测并提示用户

### 3. 配置向导需要真实终端

**问题**：dialoguer 库需要真实终端环境

**解决方案**：
- 在真实终端中运行配置向导
- 提供配置文件模板 `config.toml.example`
- 支持手动编辑配置文件

## 开发笔记

### 编译优化

```bash
# 开发版本（快速编译）
cargo build

# 发布版本（优化性能）
cargo build --release

# 检查编译错误
cargo check

# 运行测试
cargo test
```

### 调试技巧

1. **启用详细日志**：
```bash
RUST_LOG=debug trans.exe run
```

2. **检查设备**：
```bash
trans.exe check
trans.exe list-devices
trans.exe device-info
```

3. **验证配置**：
```bash
trans.exe check_config
```

### 性能调优

**降低延迟**：
- 减小 `buffer_size` 值（256 或更低）
- 使用 F32 格式

**减少 CPU 占用**：
- 增大 `buffer_size` 值（1024 或更高）
- 简化处理器链

## 未来计划

### 短期目标

- [ ] 添加更多音频处理器（均衡器、压缩器等）
- [ ] 支持动态加载处理器插件
- [ ] 添加音频可视化功能
- [ ] 支持配置文件热重载

### 中期目标

- [ ] 支持多输入/多输出
- [ ] 添加音频录制功能
- [ ] 支持 VST 插件
- [ ] 添加实时音量显示

### 长期目标

- [ ] 跨平台支持（Linux、macOS）
- [ ] 图形化界面
- [ ] 网络音频传输
- [ ] AI 降噪功能

## 参考资源

- [cpal 文档](https://docs.rs/cpal/)
- [VB-Audio 官网](https://vb-audio.com/)
- [Rust 音频处理](https://github.com/RustAudio)
- [音频采样率](https://en.wikipedia.org/wiki/Sampling_rate)

## 版本历史

### v0.1.0 (2026-01-31)

**初始版本**：
- ✅ 全双工音频处理
- ✅ 交互式配置向导
- ✅ 设备自动检测
- ✅ 彩色日志输出
- ✅ 命令行工具
- ✅ 配置文件管理

## 贡献指南

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 联系方式

如有问题或建议，请通过 GitHub Issues 联系。