mod enumeration;
mod notification_handler;
mod process_handler;
mod run;

use notification_handler::JackNotificationHandler;
use process_handler::JackProcessHandler;

pub use enumeration::*;
pub use run::*;

const DUMMY_CLIENT_NAME: &'static str = "rainout_dummy_client";
