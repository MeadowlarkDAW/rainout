use crate::error::{ChangeAudioBufferSizeError, ChangeAudioPortConfigError, RunConfigError};
use crate::{
    AudioBackendInfo, AudioBufferSizeConfig, Config, ErrorBehavior, ErrorHandler, MidiBackendInfo,
    ProcessHandler, RunOptions, StreamHandle, StreamInfo,
};

pub fn audio_backends() -> Vec<AudioBackendInfo> {
    todo!()
}

pub fn midi_backends() -> Vec<MidiBackendInfo> {
    todo!()
}

pub fn estimated_latency(config: &Config) -> Option<u32> {
    todo!()
}

pub fn sample_rate(config: &Config) -> Option<u32> {
    todo!()
}

pub fn run<P: ProcessHandler, E: ErrorHandler>(
    config: &Config,
    options: &RunOptions,
    error_behavior: &ErrorBehavior,
    process_handler: P,
    error_handler: E,
) -> Result<StreamHandle<P, E>, RunConfigError> {
    todo!()
}

pub struct PlatformStreamHandle<P: ProcessHandler, E: ErrorHandler> {
    process_handler: P,
    error_handler: E,
}

impl<P: ProcessHandler, E: ErrorHandler> PlatformStreamHandle<P, E> {
    pub fn stream_info(&self) -> &StreamInfo {
        todo!()
    }

    pub fn change_audio_port_config(
        &mut self,
        audio_in_ports: Option<Vec<String>>,
        audio_out_ports: Option<Vec<String>>,
    ) -> Result<(), ChangeAudioPortConfigError> {
        todo!()
    }

    pub fn change_audio_buffer_size_config(
        &mut self,
        config: AudioBufferSizeConfig,
    ) -> Result<(), ChangeAudioBufferSizeError> {
        todo!()
    }

    pub fn can_change_audio_port_config(&self) -> bool {
        todo!()
    }

    pub fn can_change_audio_buffer_size_config(&self) -> bool {
        todo!()
    }
}
