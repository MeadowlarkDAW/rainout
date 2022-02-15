use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum StreamError {
    // TODO
}
impl Error for StreamError {}
impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum FatalStreamError {
    // TODO
}
impl Error for FatalStreamError {}
impl fmt::Display for FatalStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum RunConfigError {
    // TODO
}
impl Error for RunConfigError {}
impl fmt::Display for RunConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeAudioPortConfigError {
    // TODO
}
impl Error for ChangeAudioPortConfigError {}
impl fmt::Display for ChangeAudioPortConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeAudioBufferSizeError {
    // TODO
}
impl Error for ChangeAudioBufferSizeError {}
impl fmt::Display for ChangeAudioBufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
