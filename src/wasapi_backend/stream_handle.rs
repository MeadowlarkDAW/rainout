use crate::{PlatformStreamHandle, ProcessHandler, StreamInfo};

pub struct WasapiStreamHandle {
    pub stream_info: StreamInfo,
}

impl<P: ProcessHandler> PlatformStreamHandle<P> for WasapiStreamHandle {
    fn stream_info(&self) -> &crate::StreamInfo {
        &self.stream_info
    }
}
