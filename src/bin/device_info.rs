use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();

    println!("=== 输入设备详细信息 ===");
    for device in host.input_devices().unwrap() {
        let name = device.name().unwrap();
        println!("\n设备: {}", name);
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                println!("  格式: {:?}, 采样率: {}-{} Hz, 通道数: {:?}, 缓冲区: {:?}", 
                    config.sample_format(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    config.channels(),
                    config.buffer_size());
            }
        }
    }

    println!("\n=== 输出设备详细信息 ===");
    for device in host.output_devices().unwrap() {
        let name = device.name().unwrap();
        println!("\n设备: {}", name);
        if let Ok(configs) = device.supported_output_configs() {
            for config in configs {
                println!("  格式: {:?}, 采样率: {}-{} Hz, 通道数: {:?}, 缓冲区: {:?}", 
                    config.sample_format(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    config.channels(),
                    config.buffer_size());
            }
        }
    }
}