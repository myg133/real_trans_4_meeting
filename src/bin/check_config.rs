use anyhow::Result;
use colored::Colorize;
use cpal::traits::{DeviceTrait, HostTrait};

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  音频设备配置检查工具                                           ║");
    println!("║  适用于：OBS、Zoom、Teams、腾讯会议等视频会议/直播软件            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    let host = cpal::default_host();

    // 收集所有设备
    let input_devices: Vec<_> = host.input_devices()?.collect();
    let output_devices: Vec<_> = host.output_devices()?.collect();

    // 查找 VB-Cable 设备
    println!("📻 虚拟音频设备 (VB-Cable):");
    println!("════════════════════════════════════════════════════════════════");

    let mut cable_a_found = false;
    let mut cable_found = false;

    println!("\n【CABLE-A (用于输入流)】");
    for device in &output_devices {
        if let Ok(name) = device.name() {
            if name.contains("CABLE-A Input") {
                println!("  ✓ CABLE-A Input: {}", name);
                cable_a_found = true;
            }
        }
    }
    for device in &input_devices {
        if let Ok(name) = device.name() {
            if name.contains("CABLE-A Output") {
                println!("  ✓ CABLE-A Output: {}", name);
                println!("    → OBS 的输入设备应该选择这个");
                cable_a_found = true;
            }
        }
    }
    if !cable_a_found {
        println!("  ✗ 未找到 CABLE-A 设备");
    }

    println!("\n【CABLE (用于输出流)】");
    for device in &output_devices {
        if let Ok(name) = device.name() {
            if name.contains("CABLE Input") && !name.contains("CABLE-A") {
                println!("  ✓ CABLE Input: {}", name);
                cable_found = true;
            }
        }
    }
    for device in &input_devices {
        if let Ok(name) = device.name() {
            if name.contains("CABLE Output") && !name.contains("CABLE-A") {
                println!("  ✓ CABLE Output: {}", name);
                println!("    → OBS 的输出设备应该选择这个");
                cable_found = true;
            }
        }
    }
    if !cable_found {
        println!("  ✗ 未找到 CABLE 设备");
    }

    println!("\n🎤 物理输入设备 (麦克风):");
    println!("════════════════════════════════════════════════════════════════");
    for device in &input_devices {
        if let Ok(name) = device.name() {
            if !name.contains("CABLE") {
                println!("  - {}", name);
            }
        }
    }

    println!("\n🔊 物理输出设备 (扬声器):");
    println!("════════════════════════════════════════════════════════════════");
    for device in &output_devices {
        if let Ok(name) = device.name() {
            if !name.contains("CABLE") {
                println!("  - {}", name);
            }
        }
    }

    println!("\n📋 {} 配置建议:", "⚙️".yellow());
    println!("════════════════════════════════════════════════════════════════");
    println!("在 {} 中:", "config.toml".green().bold());
    println!("  vbcable_input_name  = {}", "\"CABLE-A Input\"".cyan());
    println!("  vbcable_output_name = {}", "\"CABLE Output\"".cyan());
    println!();
    println!("在 {} 中:", "会议软件（OBS、Zoom、Teams 等）".yellow());
    println!(
        "  {} 输入设备（麦克风）: {}",
        "🎤".cyan(),
        "CABLE-A Output (VB-Audio Cable A)".cyan().bold()
    );
    println!(
        "  {} 输出设备（扬声器）: {}",
        "🔊".cyan(),
        "CABLE Output (VB-Audio Virtual Cable)".cyan().bold()
    );

    Ok(())
}
