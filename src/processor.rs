use anyhow::Result;

/// 音频处理器接口
pub trait AudioProcessor: Send + Sync {
    /// 处理音频数据，原地修改 buffer
    fn process(&mut self, buffer: &mut [f32]) -> Result<()>;

    /// 获取处理器名称
    #[allow(dead_code)]
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
#[allow(dead_code)]
pub struct GainProcessor {
    gain: f32,
}

#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct NoiseGateProcessor {
    threshold: f32,
}

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    mod passthrough_processor {
        use super::*;

        #[test]
        fn test_does_not_modify_buffer() {
            let mut processor = PassThroughProcessor;
            let mut buffer = vec![0.5, -0.3, 0.8, 0.0, -0.9];
            let expected = buffer.clone();

            let result = processor.process(&mut buffer);

            assert!(result.is_ok());
            assert_eq!(buffer, expected);
        }

        #[test]
        fn test_returns_ok() {
            let mut processor = PassThroughProcessor;
            let mut buffer = vec![0.5];

            let result = processor.process(&mut buffer);

            assert!(result.is_ok());
        }

        #[test]
        fn test_handles_empty_buffer() {
            let mut processor = PassThroughProcessor;
            let mut buffer: Vec<f32> = vec![];

            let result = processor.process(&mut buffer);

            assert!(result.is_ok());
        }

        #[test]
        fn test_name() {
            let processor = PassThroughProcessor;
            assert_eq!(processor.name(), "直通处理器");
        }
    }

    mod gain_processor {
        use super::*;

        #[test]
        fn test_amplifies_audio() {
            let mut processor = GainProcessor::new(2.0);
            let mut buffer = vec![0.5, -0.5, 0.3];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 1.0).abs() < 0.001);
            assert!((buffer[1] - (-1.0)).abs() < 0.001);
            assert!((buffer[2] - 0.6).abs() < 0.001);
        }

        #[test]
        fn test_attenuates_audio() {
            let mut processor = GainProcessor::new(0.5);
            let mut buffer = vec![1.0, -0.8, 0.4];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 0.5).abs() < 0.001);
            assert!((buffer[1] - (-0.4)).abs() < 0.001);
            assert!((buffer[2] - 0.2).abs() < 0.001);
        }

        #[test]
        fn test_clips_positive_values() {
            let mut processor = GainProcessor::new(3.0);
            let mut buffer = vec![0.5];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_clips_negative_values() {
            let mut processor = GainProcessor::new(3.0);
            let mut buffer = vec![-0.5];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - (-1.0)).abs() < 0.001);
        }

        #[test]
        fn test_handles_empty_buffer() {
            let mut processor = GainProcessor::new(2.0);
            let mut buffer: Vec<f32> = vec![];

            let result = processor.process(&mut buffer);

            assert!(result.is_ok());
        }

        #[test]
        fn test_handles_zero_gain() {
            let mut processor = GainProcessor::new(0.0);
            let mut buffer = vec![0.5, -0.5, 1.0];

            processor.process(&mut buffer).unwrap();

            assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
        }

        #[test]
        fn test_name() {
            let processor = GainProcessor::new(1.0);
            assert_eq!(processor.name(), "音量增益处理器");
        }
    }

    mod noise_gate_processor {
        use super::*;

        #[test]
        fn test_mutes_below_threshold() {
            let mut processor = NoiseGateProcessor::new(0.5);
            let mut buffer = vec![0.3, -0.3, 0.1];

            processor.process(&mut buffer).unwrap();

            assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
        }

        #[test]
        fn test_preserves_above_threshold() {
            let mut processor = NoiseGateProcessor::new(0.3);
            let mut buffer = vec![0.5, -0.5, 0.8];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 0.5).abs() < 0.001);
            assert!((buffer[1] - (-0.5)).abs() < 0.001);
            assert!((buffer[2] - 0.8).abs() < 0.001);
        }

        #[test]
        fn test_mixed_values() {
            let mut processor = NoiseGateProcessor::new(0.3);
            let mut buffer = vec![0.5, -0.2, 0.1, -0.8];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 0.5).abs() < 0.001);
            assert_eq!(buffer[1], 0.0);
            assert_eq!(buffer[2], 0.0);
            assert!((buffer[3] - (-0.8)).abs() < 0.001);
        }

        #[test]
        fn test_at_exact_threshold() {
            let mut processor = NoiseGateProcessor::new(0.5);
            let mut buffer = vec![0.5, -0.5];

            processor.process(&mut buffer).unwrap();

            assert!((buffer[0] - 0.5).abs() < 0.001);
            assert!((buffer[1] - (-0.5)).abs() < 0.001);
        }

        #[test]
        fn test_handles_empty_buffer() {
            let mut processor = NoiseGateProcessor::new(0.1);
            let mut buffer: Vec<f32> = vec![];

            let result = processor.process(&mut buffer);

            assert!(result.is_ok());
        }

        #[test]
        fn test_name() {
            let processor = NoiseGateProcessor::new(0.1);
            assert_eq!(processor.name(), "噪音门处理器");
        }
    }

    mod processor_chain {
        use super::*;

        #[test]
        fn test_empty_chain() {
            let mut chain = ProcessorChain::new();
            let mut buffer = vec![0.5, -0.3];

            let result = chain.process(&mut buffer);

            assert!(result.is_ok());
            assert_eq!(buffer, vec![0.5, -0.3]);
        }

        #[test]
        fn test_single_processor() {
            let mut chain = ProcessorChain::new();
            chain.add_processor(Box::new(GainProcessor::new(2.0)));
            let mut buffer = vec![0.5];

            chain.process(&mut buffer).unwrap();

            assert!((buffer[0] - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_multiple_processors() {
            let mut chain = ProcessorChain::new();
            chain.add_processor(Box::new(NoiseGateProcessor::new(0.1)));
            chain.add_processor(Box::new(GainProcessor::new(2.0)));
            let mut buffer = vec![0.5, 0.05];

            chain.process(&mut buffer).unwrap();

            assert!((buffer[0] - 1.0).abs() < 0.001);
            assert_eq!(buffer[1], 0.0);
        }

        #[test]
        fn test_execution_order() {
            let mut chain = ProcessorChain::new();
            chain.add_processor(Box::new(GainProcessor::new(2.0)));
            chain.add_processor(Box::new(GainProcessor::new(0.5)));
            let mut buffer = vec![1.0];

            chain.process(&mut buffer).unwrap();

            assert!((buffer[0] - 0.5).abs() < 0.001);
        }

        #[test]
        fn test_handles_empty_buffer() {
            let mut chain = ProcessorChain::new();
            chain.add_processor(Box::new(PassThroughProcessor));
            let mut buffer: Vec<f32> = vec![];

            let result = chain.process(&mut buffer);

            assert!(result.is_ok());
        }
    }
}
