#[derive(Debug)]
pub enum SpawnRtThreadError {
    AudioServerUnavailable(String),
    SystemDeviceNotFound(String),
    SystemHalfDuplexDeviceNotFound(String),
    SystemPortNotFound(String, String),
    NoSystemPortsGiven(String),
    IdNotUnique(String),
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
            SpawnRtThreadError::SystemDeviceNotFound(device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system audio device {} could not be found",
                    device
                )
            }
            SpawnRtThreadError::SystemHalfDuplexDeviceNotFound(device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system half duplex audio device {} could not be found",
                    device
                )
            }
            SpawnRtThreadError::SystemPortNotFound(port, device) => {
                write!(
                    f,
                    "Error spawning rt thread: The system port {} could not be found. This port was requested for bus/controller with id {}",
                    port,
                    device,
                )
            }
            SpawnRtThreadError::NoSystemPortsGiven(id) => {
                write!(
                    f,
                    "Error spawning rt thread: No system ports were set for the bus with id {}",
                    id,
                )
            }
            SpawnRtThreadError::IdNotUnique(id) => {
                write!(
                    f,
                    "Error spawning rt thread: Two or more busses/controllers have the same id {}",
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
