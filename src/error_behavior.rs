/// Flags passed into the `run()` method that describe how to respond to
/// certain errors in the configuration,
#[derive(Default, Debug, Clone)]
pub struct ErrorBehavior {
    pub audio_backend_not_found: NotFoundBehavior,
    pub audio_device_not_found: NotFoundBehavior,
    pub audio_port_not_found: AudioPortNotFoundBehavior,
    pub buffer_size_config_error: BufferSizeConfigErrorBehavior,

    #[cfg(feature = "midi")]
    pub midi_backend_not_found: NotFoundBehavior,
    #[cfg(feature = "midi")]
    pub midi_device_not_found: MidiDeviceNotFoundBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotFoundBehavior {
    /// Try to use the next best backend/device before returning
    /// an error.
    ///
    /// This is the default behavior.
    TryNextBest,

    /// Stop trying to run the stream and return an error.
    ReturnWithError,
}

impl Default for NotFoundBehavior {
    fn default() -> Self {
        NotFoundBehavior::TryNextBest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPortNotFoundBehavior {
    /// Use empty (silent) buffers for each invalid port.
    ///
    /// This is the default behavior.
    UseEmptyBufferForInvalidPorts,

    /// Stop trying to run the stream and return an error.
    ReturnWithError,
}

impl Default for AudioPortNotFoundBehavior {
    fn default() -> Self {
        AudioPortNotFoundBehavior::UseEmptyBufferForInvalidPorts
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleRateConfigErrorBehavior {
    /// Try to use the next best sample rate before returning
    /// an error.
    ///
    /// This is the default behavior.
    TryNextBest,

    /// Try to use the next best sample rate before returning
    /// an error (but only if that sample rate is within the
    /// given range (inclusive)).
    TryNextBestWithMinMaxSR(u32, u32),

    /// Stop trying to run the stream and return an error.
    ReturnWithError,
}

impl Default for SampleRateConfigErrorBehavior {
    fn default() -> Self {
        SampleRateConfigErrorBehavior::TryNextBest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSizeConfigErrorBehavior {
    /// If the device supports fixed size buffers and yet the backend
    /// failed to set the buffer with the given size, try to use the
    /// next best fixed buffer size before falling back to using
    /// unfixed sized buffers.
    ///
    /// This has no effect if the device does not support fixed size
    /// buffers.
    ///
    /// This is the default behavior.
    TryNextBestThenFallbackToUnfixedSize,

    /// If the device supports fixed size buffers and yet the backend
    /// failed to set the buffer with the given size, try to use the
    /// next best fixed buffer size before returning an error.
    ///
    /// This has no effect if the device does not support fixed size
    /// buffers.
    TryNextBestThenReturnError,

    /// If the device supports fixed size buffers and yet the backend
    /// failed to set the buffer with the given size, use unfixed
    /// sized buffers instead.
    ///
    /// This has no effect if the device does not support fixed size
    /// buffers.
    FallbackToUnfixedSize,

    /// If the device supports fixed size buffers and yet the backend
    /// failed to set the buffer with the given size, stop trying
    /// to run the stream and return an error.
    ///
    /// This has no effect if the device does not support fixed size
    /// buffers.
    ReturnWithError,
}

impl Default for BufferSizeConfigErrorBehavior {
    fn default() -> Self {
        BufferSizeConfigErrorBehavior::TryNextBestThenFallbackToUnfixedSize
    }
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiDeviceNotFoundBehavior {
    /// Use empty (silent) buffers for each invalid device.
    ///
    /// This is the default behavior.
    UseEmptyBufferForInvalidDevices,

    /// Stop trying to run the stream and return an error.
    ReturnWithError,
}

#[cfg(feature = "midi")]
impl Default for MidiDeviceNotFoundBehavior {
    fn default() -> Self {
        MidiDeviceNotFoundBehavior::UseEmptyBufferForInvalidDevices
    }
}
