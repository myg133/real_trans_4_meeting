mod audio_io;
mod processor;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use log::info;
use std::sync::Arc;

use audio_io::AudioStream;
use processor::ProcessorChain;
use cpal::traits::{DeviceTrait, HostTrait};

// 获取系统默认输入设备
fn get_default_input_device() -> Option<String> {
    let host = cpal::default_host();
    match host.default_input_device() {
        Some(device) => device.name().ok(),
        None => None,
    }
}

// 获取系统默认输出设备
fn get_default_output_device() -> Option<String> {
    let host = cpal::default_host();
    match host.default_output_device() {
        Some(device) => device.name().ok(),
        None => None,
    }
}

#[derive(Parser)]
#[command(name = "trans")]
#[command(about = "全双工音频处理程序 - 为视频会议/直播软件提供音频处理功能", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行音频处理程序
    Run,
    /// 交互式配置向导
    Config,
    /// 检查音频设备
    Check,
    /// 列出所有音频设备
    ListDevices,
    /// 显示设备详细信息（格式、采样率等）
    DeviceInfo,
}

fn list_devices() -> Result<(Vec<String>, Vec<String>)> {
    let host = cpal::default_host();
    
    let input_devices: Vec<String> = host.input_devices()?
        .filter_map(|d| d.name().ok())
        .collect();
    
    let output_devices: Vec<String> = host.output_devices()?
        .filter_map(|d| d.name().ok())
        .collect();
    
    Ok((input_devices, output_devices))
}

fn interactive_config() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  全双工音频处理程序 - 配置向导                                   ║");
    println!("║  适用于：OBS、Zoom、Teams、腾讯会议等视频会议/直播软件            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    let (input_devices, output_devices) = list_devices()?;

    // 检测虚拟设备
    let vbcable_inputs: Vec<&String> = output_devices.iter()
        .filter(|s| s.contains("CABLE") && s.contains("Input"))
        .collect();
    
    let vbcable_outputs: Vec<&String> = input_devices.iter()
        .filter(|s| s.contains("CABLE") && s.contains("Output"))
        .collect();

    println!("📻 检测到的虚拟音频设备:");
    println!("════════════════════════════════════════════════════════════════");
    println!("虚拟输入设备（虚拟扬声器）: {} 个", vbcable_inputs.len());
    for device in &vbcable_inputs {
        println!("  - {}", device);
    }
    println!("虚拟输出设备（虚拟麦克风）: {} 个", vbcable_outputs.len());
    for device in &vbcable_outputs {
        println!("  - {}", device);
    }
    println!();

    // 检查虚拟设备数量
    if vbcable_inputs.is_empty() || vbcable_outputs.is_empty() {
        println!("❌ 错误：未检测到足够的虚拟音频设备！");
        println!();
        println!("全双工音频处理需要至少 1 个虚拟音频设备。");
        println!();
        println!("请安装 VB-Cable:");
        println!("  下载地址: https://vb-audio.com/Cable/");
        println!("  建议安装: VB-Cable + VB-Cable A（共 2 个）");
        println!();
        println!("安装完成后，重新运行此程序。");
        std::process::exit(1);
    }

    if vbcable_inputs.len() == 1 && vbcable_outputs.len() == 1 {
        println!("✓ 检测到 1 个虚拟音频设备");
        println!("  这只支持单向音频处理");
        println!("  如需全双工处理，建议再安装一个 VB-Cable");
        println!();
    } else if vbcable_inputs.len() >= 2 && vbcable_outputs.len() >= 2 {
        println!("✓ 检测到 {} 个虚拟音频设备，支持全双工处理", vbcable_inputs.len());
        println!();
    }

    // 选择物理麦克风
    println!("🎤 选择物理麦克风（输入设备）:");
    let physical_input_devices: Vec<&String> = input_devices.iter()
        .filter(|s| !s.contains("CABLE"))
        .collect();
    
    if physical_input_devices.is_empty() {
        println!("❌ 错误：未检测到物理麦克风设备！");
        std::process::exit(1);
    }
    
    // 获取系统默认麦克风
    let default_mic = get_default_input_device();
    let default_mic_index = if let Some(ref name) = default_mic {
        physical_input_devices.iter().position(|s| s.contains(name)).unwrap_or(0)
    } else {
        0
    };
    
    let mic_items: Vec<&str> = physical_input_devices.iter().map(|s| s.as_str()).collect();
    let mic_index = Select::with_theme(&ColorfulTheme::default())
        .items(&mic_items)
        .default(default_mic_index)
        .with_prompt(if default_mic.is_some() {
            format!("当前系统默认: {}", default_mic.unwrap())
        } else {
            "选择麦克风".to_string()
        })
        .interact()?;
    let input_device = physical_input_devices[mic_index].clone();

    // 选择物理扬声器
    println!("\n🔊 选择物理扬声器（输出设备）:");
    let physical_output_devices: Vec<&String> = output_devices.iter()
        .filter(|s| !s.contains("CABLE"))
        .collect();
    
    if physical_output_devices.is_empty() {
        println!("❌ 错误：未检测到物理扬声器设备！");
        std::process::exit(1);
    }
    
    // 获取系统默认扬声器
    let default_speaker = get_default_output_device();
    let default_speaker_index = if let Some(ref name) = default_speaker {
        physical_output_devices.iter().position(|s| s.contains(name)).unwrap_or(0)
    } else {
        0
    };
    
    let speaker_items: Vec<&str> = physical_output_devices.iter().map(|s| s.as_str()).collect();
    let speaker_index = Select::with_theme(&ColorfulTheme::default())
        .items(&speaker_items)
        .default(default_speaker_index)
        .with_prompt(if default_speaker.is_some() {
            format!("当前系统默认: {}", default_speaker.unwrap())
        } else {
            "选择扬声器".to_string()
        })
        .interact()?;
    let output_device = physical_output_devices[speaker_index].clone();

    // 选择虚拟设备 A（用于输入流）
    println!("\n📻 选择虚拟设备 A（用于输入流 - 你说话 → 会议软件）:");
    println!("   这个设备将接收处理后的麦克风声音");
    let vbcable_a_items: Vec<&str> = vbcable_inputs.iter().map(|s| s.as_str()).collect();
    let vbcable_a_index = Select::with_theme(&ColorfulTheme::default())
        .items(&vbcable_a_items)
        .default(0)
        .interact()?;
    let vbcable_input = vbcable_inputs[vbcable_a_index].clone();
    
    // 找到对应的 Output 设备
    let vbcable_a_output = vbcable_outputs.iter()
        .find(|s| {
            let input_name = vbcable_input.replace(" Input", "");
            let output_name = s.replace(" Output", "");
            input_name == output_name
        })
        .unwrap_or(&vbcable_outputs[0]);

    // 选择虚拟设备 B（用于输出流）- 从可用设备中移除已选择的
    println!("\n📻 选择虚拟设备 B（用于输出流 - 会议软件 → 你听到）:");
    println!("   这个设备将接收会议软件的输出声音");
    
    let available_vbcable_outputs: Vec<&&String> = vbcable_outputs.iter()
        .filter(|s| *s != vbcable_a_output)
        .collect();
    
    let vbcable_output = if available_vbcable_outputs.is_empty() {
        // 如果只有一个虚拟设备，使用同一个
        println!("   ℹ️  只有一个虚拟设备，将同时用于输入和输出");
        vbcable_a_output.clone()
    } else {
        let items: Vec<&str> = available_vbcable_outputs.iter().map(|s| s.as_str()).collect();
        let index = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()?;
        available_vbcable_outputs[index].clone()
    };

    // 保存配置
    let config_str = format!(
        r#"# 音频设备配置 - 全双工音频处理程序
# 
# 工作原理：
# ┌─────────────────────────────────────────────────────────────────┐
# │ 输入流（你说话）:                                                │
# │   物理麦克风 → 程序处理 → CABLE-A Input → CABLE-A Output → 会   │
# │   议软件                                                         │
# │                                                                 │
# │ 输出流（对方说话）:                                              │
# │   会议软件 → CABLE Output → 程序处理 → 物理扬声器 → 你听到       │
# └─────────────────────────────────────────────────────────────────┘
#
# 适用于：OBS、Zoom、Teams、腾讯会议等任何视频会议/直播软件
#
# 会议软件设置：
#   输入设备（麦克风）: CABLE-A Input (VB-Audio Cable A)
#   输出设备（扬声器）: CABLE Output (VB-Audio Virtual Cable)

# ========================================
# 输入流配置（处理你的麦克风声音）
# ========================================

# 物理输入设备 - 你的真实麦克风
input_device_name = "{}"

# 虚拟设备 A - 程序输出处理后的麦克风声音
vbcable_input_name = "{}"

# ========================================
# 输出流配置（处理对方的声音）
# ========================================

# 虚拟设备 Output - 会议软件输出声音到这里
vbcable_output_name = "{}"

# 物理输出设备 - 你最终听到的设备（耳机或扬声器）
output_device_name = "{}"

# ========================================
# 音频参数
# ========================================
sample_rate = 48000  # 采样率 (Hz)
buffer_size = 512    # 缓冲区大小 (帧)
"#,
        input_device, vbcable_input, vbcable_output, output_device
    );

    std::fs::write("config.toml", config_str)?;

    println!("\n✅ 配置已保存到 {}", "config.toml".green().bold());
    println!("\n📋 {} 会议软件设置:", "⚙️".yellow());
    println!("  {} 输入设备（麦克风）: {}", "🎤".cyan(), 
        vbcable_a_output.cyan().bold());
    println!("  {} 输出设备（扬声器）: {}", "🔊".cyan(), vbcable_output.cyan().bold());
    println!("\n现在运行 {} 或 {} 启动程序", "trans.exe run".green(), "trans.exe".green());

    Ok(())
}

fn check_devices() -> Result<()> {
    let (input_devices, output_devices) = list_devices()?;

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  音频设备列表                                                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    println!("\n📻 输入设备:");
    for device in &input_devices {
        if device.contains("CABLE") {
            println!("  [虚拟] {}", device);
        } else {
            println!("  [物理] {}", device);
        }
    }
    
    println!("\n🔊 输出设备:");
    for device in &output_devices {
        if device.contains("CABLE") {
            println!("  [虚拟] {}", device);
        } else {
            println!("  [物理] {}", device);
        }
    }

    Ok(())
}

fn show_device_info() -> Result<()> {
    let host = cpal::default_host();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  设备详细信息                                                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    println!("\n📻 输入设备详细信息:");
    for device in host.input_devices().unwrap() {
        let name = device.name().unwrap();
        println!("\n  设备: {}", name);
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                println!("    格式: {:?}, 采样率: {}-{} Hz, 通道数: {:?}", 
                    config.sample_format(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    config.channels());
            }
        }
    }

    println!("\n🔊 输出设备详细信息:");
    for device in host.output_devices().unwrap() {
        let name = device.name().unwrap();
        println!("\n  设备: {}", name);
        if let Ok(configs) = device.supported_output_configs() {
            for config in configs {
                println!("    格式: {:?}, 采样率: {}-{} Hz, 通道数: {:?}", 
                    config.sample_format(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    config.channels());
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config) => {
            return interactive_config();
        }
        Some(Commands::Check) => {
            return check_devices();
        }
        Some(Commands::ListDevices) => {
            let (input_devices, output_devices) = list_devices()?;
            println!("=== 输入设备 ===");
            for device in &input_devices {
                println!("  - {}", device);
            }
            println!("\n=== 输出设备 ===");
            for device in &output_devices {
                println!("  - {}", device);
            }
            return Ok(());
        }
        Some(Commands::DeviceInfo) => {
            return show_device_info();
        }
        Some(Commands::Run) | None => {
            // 检查配置文件是否存在，如果不存在则自动运行配置向导
            if !std::path::Path::new("config.toml").exists() {
                println!("⚠️  {} 未找到配置文件", "config.toml".yellow());
                println!("{} 正在启动配置向导...", "🚀".green());
                println!();
                interactive_config()?;
                println!();
                println!("{} 配置完成！正在启动程序...", "✅".green());
                println!();
            }
        }
    }

    info!("启动全双工音频处理程序...");

    // 创建处理器链
    let mut input_processor = ProcessorChain::new();
    input_processor.add_processor(Box::new(processor::PassThroughProcessor));

    let mut output_processor = ProcessorChain::new();
    output_processor.add_processor(Box::new(processor::PassThroughProcessor));

    // 获取音频设备配置
    let config = config::AudioConfig::load_or_default()?;

    info!("配置:");
    info!("╔════════════════════════════════════════════════════════════════╗");
    info!("║ 输入流（你说话）                                                ║");
    info!("║   物理麦克风: {} → 处理 → {}", config.input_device_name, config.vbcable_input_name);
    info!("║   内部管道: {} → {}", config.vbcable_input_name.replace(" Input", " Output"), config.vbcable_input_name);
    info!("║   {} 会议软件输入设备选择: {}", "⚡".yellow(), config.vbcable_input_name.cyan().bold());
    info!("╠════════════════════════════════════════════════════════════════╣");
    info!("║ 输出流（对方说话）                                              ║");
    info!("║   {} 会议软件输出设备选择: {}", "⚡".yellow(), config.vbcable_output_name.cyan().bold());
    info!("║   {} → 处理 → 物理扬声器: {}", config.vbcable_output_name, config.output_device_name);
    info!("╠════════════════════════════════════════════════════════════════╣");
    info!("║ 音频参数                                                       ║");
    info!("║   采样率: {} Hz", config.sample_rate);
    info!("║   缓冲区大小: {} 帧", config.buffer_size);
    info!("╚════════════════════════════════════════════════════════════════╝");

    // 启动输入流: 物理麦克风 -> 处理器 -> CABLE-A Input
    // 音频通过内部管道传到 CABLE-A Output，视频会议软件从 CABLE-A Output 读取
    let _input_stream = AudioStream::create_duplex_stream(
        &config.input_device_name,
        &config.vbcable_input_name,
        config.sample_rate,
        config.buffer_size,
        Arc::new(std::sync::Mutex::new(input_processor)),
        true,
    )?;

    // 启动输出流: CABLE Output -> 处理器 -> 物理扬声器
    // 视频会议软件输出到 CABLE Output，程序处理后传到物理扬声器
    let _output_stream = AudioStream::create_duplex_stream(
        &config.vbcable_output_name,
        &config.output_device_name,
        config.sample_rate,
        config.buffer_size,
        Arc::new(std::sync::Mutex::new(output_processor)),
        false,
    )?;

    // 运行音频流（保持程序运行）
    info!("音频流已启动，按 Ctrl+C 退出...");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}