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

mod config;
mod enumeration;
mod run;

pub mod error;
pub mod error_behavior;

pub use config::*;
pub use enumeration::*;
pub use error_behavior::ErrorBehavior;
pub use run::*;
