#[derive(Debug)]
pub enum SpawnRtThreadError {
    AudioServerUnavailable(String),
    SystemDuplexDeviceNotFound(String),
    SystemAudioInDeviceNotFound(String),
    SystemAudioOutDeviceNotFound(String),
    SystemInChannelNotFound(String, u16),
    SystemOutChannelNotFound(String, u16),
    NoSystemChannelsGiven(String),
    DeviceIdNotUnique(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for SpawnRtThreadError {}

impl std::fmt::Display for SpawnRtThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnRtThreadError::AudioServerUnavailable(server) => {
                write!(
                    f,
                    "Error spawning rt thread: The audio sever is unavailable: {}",
                    server
                )
            }
            SpawnRtThreadError::SystemDuplexDeviceNotFound(device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system duplex audio device {} could not be found",
                    device
                )
            }
            SpawnRtThreadError::SystemAudioInDeviceNotFound(device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system input audio device {} could not be found",
                    device
                )
            }
            SpawnRtThreadError::SystemAudioOutDeviceNotFound(device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system output audio device {} could not be found",
                    device
                )
            }
            SpawnRtThreadError::SystemInChannelNotFound(device, channel) => {
                write!(
                    f,
                    "Error spawning rt thread: The system audio device {} does not have the input channel {}",
                    device,
                    channel,
                )
            }
            SpawnRtThreadError::SystemOutChannelNotFound(device, channel) => {
                write!(
                    f,
                    "Error spawning rt thread: The system audio device {} does not have the output channel {}",
                    device,
                    channel,
                )
            }
            SpawnRtThreadError::NoSystemChannelsGiven(id) => {
                write!(
                    f,
                    "Error spawning rt thread: No system channels were set for the device with id {}",
                    id,
                )
            }
            SpawnRtThreadError::DeviceIdNotUnique(id) => {
                write!(
                    f,
                    "Error spawning rt thread: Two or more devices have the same id {}",
                    id,
                )
            }
            SpawnRtThreadError::PlatformSpecific(e) => {
                write!(f, "Error spawning rt thread: Platform error: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum StreamError {
    AudioServerDisconnected(String),
    AudioDeviceDisconnected(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for StreamError {}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::AudioServerDisconnected(server) => {
                write!(
                    f,
                    "Stream error: The audio sever was disconnected: {}",
                    server
                )
            }
            StreamError::AudioDeviceDisconnected(device) => {
                write!(
                    f,
                    "Stream error: The audio device was disconnected: {}",
                    device
                )
            }
            StreamError::PlatformSpecific(e) => {
                write!(f, "Stream error: Platform error: {}", e)
            }
        }
    }
}