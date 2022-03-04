use ringbuf::{Consumer, RingBuffer};

use crate::{error::StreamError, DeviceID};

#[non_exhaustive]
#[derive(Debug, Clone)]
/// A message sent from the audio thread.
pub enum StreamMsg {
    /// An audio device was unplugged while the stream was running. Any connected
    /// ports will input/output silence.
    AudioDeviceDisconnected(DeviceID),

    /// An audio device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    ///
    /// This will only be sent after an `AudioDeviceDisconnected` event.
    AudioDeviceReconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// The MIDI output device was not found. This port will produce no MIDI events.
    MidiDeviceDisconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// A MIDI device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    ///
    /// This will only be sent after an `MidiDeviceDisconnected` event.
    MidiDeviceReconnected(DeviceID),

    /// An error that caused the stream to close. Please discard this Stream Handle
    /// channel and prepare to start a new stream.
    Error(StreamError),

    /// The audio stream was closed gracefully. Please discard this Stream Handle.
    Closed,
}

/// The message channel that recieves notifications from the audio thread including
/// any errors that have occurred.
pub struct StreamMsgChannel {
    from_audio_thread_rx: Consumer<StreamMsg>,
}

impl StreamMsgChannel {
    pub(crate) fn new(msg_buffer_size: usize) -> (Self, ringbuf::Producer<StreamMsg>) {
        let (to_channel_tx, from_audio_thread_rx) =
            RingBuffer::<StreamMsg>::new(msg_buffer_size).split();

        (Self { from_audio_thread_rx }, to_channel_tx)
    }

    /// Returns capacity of the message buffer.
    ///
    /// The capacity of the buffer is constant.
    pub fn capacity(&self) -> usize {
        self.from_audio_thread_rx.capacity()
    }

    /// Checks if the message buffer is empty.
    ///
    /// *The result may become irrelevant at any time because of concurring activity of the producer.*
    pub fn is_empty(&self) -> bool {
        self.from_audio_thread_rx.is_empty()
    }

    /// Removes latest element from the message buffer and returns it.
    /// Returns `None` if the message buffer is empty.
    pub fn pop(&mut self) -> Option<StreamMsg> {
        self.from_audio_thread_rx.pop()
    }

    /// Repeatedly calls the closure `f` passing elements removed from the message buffer to it.
    ///
    /// The closure is called until it returns `false` or the message buffer is empty.
    ///
    /// The method returns number of elements been removed from the buffer.
    pub fn pop_each<F: FnMut(StreamMsg) -> bool>(&mut self, f: F, count: Option<usize>) -> usize {
        self.from_audio_thread_rx.pop_each(f, count)
    }
}
