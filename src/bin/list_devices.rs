use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();

    println!("=== 可用输入设备 ===");
    match host.input_devices() {
        Ok(devices) => {
            for device in devices {
                if let Ok(name) = device.name() {
                    println!("  - {}", name);
                }
            }
        }
        Err(e) => {
            eprintln!("无法获取输入设备: {}", e);
        }
    }

    println!("\n=== 可用输出设备 ===");
    match host.output_devices() {
        Ok(devices) => {
            for device in devices {
                if let Ok(name) = device.name() {
                    println!("  - {}", name);
                }
            }
        }
        Err(e) => {
            eprintln!("无法获取输出设备: {}", e);
        }
    }
}