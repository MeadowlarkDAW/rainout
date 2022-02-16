use crate::{AudioBackend, AudioBackendInfo, AudioDeviceInfo, Config, DeviceID};

#[cfg(feature = "midi")]
use crate::{MidiBackend, MidiBackendInfo, MidiDeviceInfo};

pub struct PlatformDeviceEnumerator {}

impl PlatformDeviceEnumerator {
    pub fn new() -> Result<Self, ()> {
        Ok(Self {})
    }

    /// Returns the available audio backends for this platform.
    pub fn available_audio_backends(&self) -> &'static [AudioBackend] {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Returns the available midi backends for this platform.
    pub fn available_midi_backends(&self) -> &'static [MidiBackend] {
        todo!()
    }

    /// Get information about a particular audio backend.
    ///
    /// This will update the list of available devices as well as the the
    /// status of whether or not this backend is running.
    ///
    /// This will return an error if the backend is not available on this system.
    pub fn enumerate_audio_backend(
        &mut self,
        backend: AudioBackend,
    ) -> Result<AudioBackendInfo, ()> {
        todo!()
    }

    /// Get information about a particular audio device.
    ///
    /// This will return an error if the given device was not found.
    pub fn enumerate_audio_device(
        &mut self,
        backend: AudioBackend,
        device_id: &DeviceID,
    ) -> Result<AudioDeviceInfo, ()> {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Get information about a particular midi backend.
    ///
    /// This will update the list of available devices as well as the the
    /// status of whether or not this backend is running.
    ///
    /// This will return an error if the backend is not available on this system.
    pub fn enumerate_midi_backend(&mut self, backend: MidiBackend) -> Result<MidiBackendInfo, ()> {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Get information about a particular midi device.
    ///
    /// This will return an error if the given device was not found.
    pub fn enumerate_midi_device(
        &mut self,
        backend: MidiBackend,
        device_id: &DeviceID,
    ) -> Result<MidiDeviceInfo, ()> {
        todo!()
    }

    /// Enumerate through each backend to find the preferred/best default audio
    /// backend for this system.
    ///
    /// If a higher priority backend does not have any available devices, then
    /// this will try to return the next best backend that does have an
    /// available device.
    ///
    /// This does not enumerate through the devices in each backend, just the
    /// names of each device.
    pub fn find_preferred_audio_backend(&mut self) -> AudioBackend {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Enumerate through each backend to find the preferred/best default midi
    /// backend for this system.
    ///
    /// If a higher priority backend does not have any available devices, then
    /// this will try to return the next best backend that does have an
    /// available device.
    ///
    /// This does not enumerate through the devices in each backend, just the
    /// names of each device.
    pub fn find_preferred_midi_backend(&mut self) -> MidiBackend {
        todo!()
    }

    /// Enumerate through each audio device to find the preferred/best default audio
    /// device for this backend.
    ///
    /// This process can be slow. Try to use `AudioBackendInfo::preferred_device`
    /// before calling this method.
    pub fn find_preferred_audio_device(
        &mut self,
        backend: AudioBackend,
    ) -> Option<AudioDeviceInfo> {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Enumerate through each midi device to find the preferred/best default midi
    /// device for this backend.
    ///
    /// This process can be slow. Try to use `MidiBackendInfo::preferred_in_device` and
    /// `MidiBackendInfo::preferred_out_device` before calling this method.
    pub fn find_preferred_midi_device(&mut self, backend: MidiBackend) -> Option<MidiDeviceInfo> {
        todo!()
    }

    /// Get the estimated total latency of a particular configuration before running it.
    ///
    /// `None` will be returned if the latency is not known at this time or if the
    /// given config is invalid.
    pub fn estimated_latency(&self, config: &Config) -> Option<u32> {
        todo!()
    }

    /// Get the sample rate of a particular configuration before running it.
    ///
    /// `None` will be returned if the sample rate is not known at this time or if the
    /// given config is invalid.
    pub fn sample_rate(&self, config: &Config) -> Option<u32> {
        todo!()
    }
}
