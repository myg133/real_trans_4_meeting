use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, StreamConfig as CpalConfig, SupportedStreamConfig};
use crossbeam_channel::{Receiver, Sender};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};

use crate::processor::ProcessorChain;

pub struct StreamConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
}

pub struct AudioStream {
    input_device: Device,
    output_device: Device,
    sample_rate: u32,
    buffer_size: u32,
    input_stream: cpal::Stream,
    output_stream: cpal::Stream,
}

impl AudioStream {
    pub fn create_duplex_stream(
        input_name: &str,
        output_name: &str,
        sample_rate: u32,
        buffer_size: u32,
        processor: Arc<Mutex<ProcessorChain>>,
        is_input_direction: bool,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let input_device = find_device_by_name(&host, input_name, true)?;
        let output_device = find_device_by_name(&host, output_name, false)?;

        // 构建输入流配置
        let input_config = {
            let configs: Vec<_> = input_device.supported_input_configs()
                .context("获取输入设备支持配置失败")?
                .collect();
            if configs.is_empty() {
                return Err(anyhow::anyhow!("输入设备没有支持的配置"));
            }
            let config = configs.iter()
                .find(|c| c.max_sample_rate().0 >= sample_rate && c.min_sample_rate().0 <= sample_rate)
                .unwrap_or(&configs[0]);
            let rate = if config.max_sample_rate().0 >= sample_rate && config.min_sample_rate().0 <= sample_rate {
                sample_rate
            } else {
                config.max_sample_rate().0
            };
            config.with_sample_rate(cpal::SampleRate(rate)).into()
        };

        // 构建输出流配置
        let output_config = {
            let configs: Vec<_> = output_device.supported_output_configs()
                .context("获取输出设备支持配置失败")?
                .collect();
            if configs.is_empty() {
                return Err(anyhow::anyhow!("输出设备没有支持的配置"));
            }
            let config = configs.iter()
                .find(|c| c.max_sample_rate().0 >= sample_rate && c.min_sample_rate().0 <= sample_rate)
                .unwrap_or(&configs[0]);
            let rate = if config.max_sample_rate().0 >= sample_rate && config.min_sample_rate().0 <= sample_rate {
                sample_rate
            } else {
                config.max_sample_rate().0
            };
            config.with_sample_rate(cpal::SampleRate(rate)).into()
        };

        // 用于在输入和输出之间传递数据的通道
        let (data_sender, data_receiver) = crossbeam_channel::bounded::<Vec<f32>>(1024);

        // 创建输入流
        let processor_clone = processor.clone();
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = data.to_vec();
                
                if let Ok(mut proc) = processor_clone.lock() {
                    if let Err(e) = proc.process(&mut buffer) {
                        error!("处理音频数据失败: {}", e);
                    }
                }

                if let Err(e) = data_sender.send(buffer) {
                    error!("发送音频数据失败: {}", e);
                }
            },
            move |err| {
                error!("输入流错误: {}", err);
            },
            None,
        )?;

        // 创建输出流
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if let Ok(buffer) = data_receiver.try_recv() {
                    let copy_len = buffer.len().min(data.len());
                    data[..copy_len].copy_from_slice(&buffer[..copy_len]);
                }
            },
            move |err| {
                error!("输出流错误: {}", err);
            },
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;
        
        let direction = if is_input_direction { "输入" } else { "输出" };
        info!("{}流已启动: {} -> {} (处理)", direction, input_name, output_name);

        Ok(Self {
            input_device,
            output_device,
            sample_rate,
            buffer_size,
            input_stream,
            output_stream,
        })
    }

    

    fn get_supported_config(&self, device: &Device, input: bool) -> Result<SupportedStreamConfig> {
        let configs: Vec<_> = if input {
            device.supported_input_configs()
                .context("获取输入设备支持配置失败")?
                .collect()
        } else {
            device.supported_output_configs()
                .context("获取输出设备支持配置失败")?
                .collect()
        };

        if configs.is_empty() {
            return Err(anyhow::anyhow!("设备没有支持的音频配置"));
        }

        // 查找支持指定采样率的配置
        let config = configs.iter()
            .find(|config| config.max_sample_rate().0 >= self.sample_rate && config.min_sample_rate().0 <= self.sample_rate)
            .unwrap_or(&configs[0]);

        // 如果配置不支持目标采样率，使用配置的采样率
        let target_rate = if config.max_sample_rate().0 >= self.sample_rate && config.min_sample_rate().0 <= self.sample_rate {
            self.sample_rate
        } else {
            config.max_sample_rate().0
        };

        Ok(config.with_sample_rate(cpal::SampleRate(target_rate)))
    }
}

fn find_device_by_name(host: &Host, name: &str, input: bool) -> Result<Device> {
    let devices = if input {
        host.input_devices()?.collect::<Vec<_>>()
    } else {
        host.output_devices()?.collect::<Vec<_>>()
    };

    for device in devices {
        if let Ok(device_name) = device.name() {
            if device_name.contains(name) {
                return Ok(device);
            }
        }
    }

    // 列出可用设备帮助调试
    warn!("未找到包含 '{}' 的设备，可用设备列表:", name);
    for device in if input { host.input_devices()? } else { host.output_devices()? } {
        if let Ok(name) = device.name() {
            warn!("  - {}", name);
        }
    }

    Err(anyhow::anyhow!("未找到音频设备: {}", name))
}

pub fn list_devices() {
    let host = cpal::default_host();

    info!("可用输入设备:");
    for device in host.input_devices().unwrap() {
        if let Ok(name) = device.name() {
            info!("  - {}", name);
        }
    }

    info!("可用输出设备:");
    for device in host.output_devices().unwrap() {
        if let Ok(name) = device.name() {
            info!("  - {}", name);
        }
    }
}