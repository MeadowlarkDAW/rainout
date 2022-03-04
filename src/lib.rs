#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

mod configuration;
mod enumeration;
mod process_info;
mod run;
mod stream_info;
mod stream_message;

#[cfg(feature = "midi")]
mod midi_buffer;

pub mod error;
pub mod error_behavior;

pub use configuration::*;
pub use enumeration::*;
pub use process_info::*;
pub use run::*;
pub use stream_info::*;
pub use stream_message::*;

#[cfg(feature = "midi")]
pub use midi_buffer::*;
