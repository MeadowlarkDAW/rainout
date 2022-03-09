#[cfg(all(target_os = "linux", feature = "jack-linux"))]
pub(crate) mod jack_backend;
#[cfg(all(target_os = "macos", feature = "jack-macos"))]
pub(crate) mod jack_backend;
#[cfg(all(target_os = "windows", feature = "jack-windows"))]
pub(crate) mod jack_backend;

#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
/// The list of backends supported by rainout
pub enum Backend {
    Jack,
    Pipewire,
    Alsa,
    CoreAudio,
    Wasapi,
    Asio,
}

#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, Copy, PartialEq)]
/// The list of backends supported by rainout
pub enum Backend {
    Jack,
    Pipewire,
    Alsa,
    CoreAudio,
    Wasapi,
    Asio,
}

impl Backend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Backend::Jack => "Jack",
            Backend::Pipewire => "Pipewire",
            Backend::Alsa => "Alsa",
            Backend::CoreAudio => "CoreAudio",
            Backend::Wasapi => "WASAPI",
            Backend::Asio => "ASIO",
        }
    }
}

mod configuration;
mod enumeration;
mod process_info;
mod run;
mod stream_info;
mod stream_message;

#[cfg(feature = "midi")]
mod midi_buffer;

pub mod error;

pub use configuration::*;
pub use enumeration::*;
pub use process_info::*;
pub use run::*;
pub use stream_info::*;
pub use stream_message::*;

#[cfg(feature = "midi")]
pub use midi_buffer::*;
