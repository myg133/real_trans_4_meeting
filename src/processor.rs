use anyhow::Result;

/// 音频处理器接口
pub trait AudioProcessor: Send + Sync {
    /// 处理音频数据，原地修改 buffer
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;
    
    /// 获取处理器名称
    fn name(&self) -> &str;
}

/// 直通处理器（不做任何处理，直接传递音频）
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

/// 处理器链：按顺序执行多个处理器
pub struct ProcessorChain {
    processors: Vec<Box<dyn AudioProcessor>>,
}

impl ProcessorChain {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn AudioProcessor>) {
        self.processors.push(processor);
    }

    pub fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        for processor in &mut self.processors {
            processor.process(buffer)?;
        }
        Ok(())
    }
}

/// 音量增益处理器
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
            *sample = (*sample * self.gain).clamp(-1.0, 1.0);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "音量增益处理器"
    }
}

/// 噪音门（静音低音量输入）
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