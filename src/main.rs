mod audio_io;
mod processor;
mod config;

use anyhow::Result;
use log::info;
use std::sync::Arc;

use audio_io::AudioStream;
use processor::ProcessorChain;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

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
    info!("║   内部管道: {} → {}", config.vbcable_input_name.replace(" Input", " Output"), config.vbcable_input_name.replace(" Input", " Output"));
    info!("║   OBS 输入设备选择: {}", config.vbcable_input_name.replace(" Input", " Output"));
    info!("╠════════════════════════════════════════════════════════════════╣");
    info!("║ 输出流（对方说话）                                              ║");
    info!("║   OBS 输出设备选择: {}", config.vbcable_output_name);
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