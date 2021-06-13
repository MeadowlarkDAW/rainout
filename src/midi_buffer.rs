// TODO: Increase message size to allow more complex midi messages?
pub const MAX_MIDI_MSG_SIZE: usize = 8;

pub const MIDI_BUFFER_SIZE: usize = 1024;

#[derive(Clone, Copy)]
pub struct RawMidi {
    /// The amount of time passed, in frames, relative to the start of the process cycle.
    pub delta_frames: u32,

    data: [u8; MAX_MIDI_MSG_SIZE],

    len: u8,
}

impl RawMidi {
    /// Create a new midi message from raw bytes.
    ///
    /// * `delta_frames` - The amount of time passed, in frames, relative to the start of the process cycle.
    /// * `data` - The raw bytes of the midi message.
    ///
    /// This returns an error if the length of `data` is greater than `MAX_MIDI_MSG_SIZE` (7).
    pub fn new(delta_frames: u32, data: &[u8]) -> Result<Self, usize> {
        if data.len() <= MAX_MIDI_MSG_SIZE {
            let mut cp_data = [0; MAX_MIDI_MSG_SIZE];
            &mut cp_data[0..data.len()].copy_from_slice(data);

            Ok(Self {
                delta_frames,
                data: cp_data,
                len: data.len() as u8,
            })
        } else {
            Err(data.len())
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[0..usize::from(self.len)]
    }

    pub fn len(&self) -> usize {
        usize::from(self.len)
    }
}

impl std::fmt::Debug for RawMidi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Raw Midi: {{ delta frames: {}, len: {}, data: {:?} }}",
            self.delta_frames,
            self.len,
            &self.data[0..usize::from(self.len)]
        )
    }
}

impl Default for RawMidi {
    fn default() -> Self {
        RawMidi {
            delta_frames: 0,
            data: [0; MAX_MIDI_MSG_SIZE],
            len: 0,
        }
    }
}

pub struct MidiControllerBuffer {
    events: [RawMidi; MIDI_BUFFER_SIZE],
    len: usize,
}

impl MidiControllerBuffer {
    pub(crate) fn new() -> Self {
        Self {
            events: [RawMidi::default(); MIDI_BUFFER_SIZE],
            len: 0,
        }
    }

    pub fn events(&self) -> &[RawMidi] {
        &self.events[0..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0
    }

    pub fn push(&mut self, event: RawMidi) -> Result<(), MidiBufferPushError> {
        if self.len >= MIDI_BUFFER_SIZE {
            return Err(MidiBufferPushError::BufferFull);
        }

        self.events[self.len] = event;

        self.len += 1;

        Ok(())
    }

    pub fn extend_from_slice(&mut self, events: &[RawMidi]) -> Result<(), MidiBufferPushError> {
        if self.len >= MIDI_BUFFER_SIZE {
            return Err(MidiBufferPushError::BufferFull);
        }

        let total_len = self.len + events.len();
        let len = total_len.min(MIDI_BUFFER_SIZE);

        &mut self.events[self.len..len].copy_from_slice(&events[0..len - self.len]);

        self.len = len;

        if total_len > len {
            Err(MidiBufferPushError::BufferFull)
        } else {
            Ok(())
        }
    }

    pub fn push_raw(&mut self, delta_frames: u32, data: &[u8]) -> Result<(), MidiBufferPushError> {
        if self.len >= MIDI_BUFFER_SIZE {
            return Err(MidiBufferPushError::BufferFull);
        }

        match RawMidi::new(delta_frames, data) {
            Ok(event) => {
                self.events[self.len] = event;

                self.len += 1;

                Ok(())
            }
            Err(len) => Err(MidiBufferPushError::EventTooLong(len)),
        }
    }

    /// Replaces the contents of this buffer with the contents of the given buffer.
    pub fn clear_and_copy_from(&mut self, buffer: &MidiControllerBuffer) {
        self.len = buffer.len;
        &mut self.events[0..buffer.len].copy_from_slice(&buffer.events[0..buffer.len]);
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl std::fmt::Debug for MidiControllerBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Midi Device Buffer: {{ len: {}, events: {:?} }}",
            self.len,
            &self.events[0..self.len]
        )
    }
}

#[derive(Debug)]
pub enum MidiBufferPushError {
    /// The buffer is full.
    BufferFull,

    /// The given midi event is too long.
    EventTooLong(usize),
}

impl std::error::Error for MidiBufferPushError {}

impl std::fmt::Display for MidiBufferPushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiBufferPushError::BufferFull => {
                write!(f, "Buffer is full",)
            }
            MidiBufferPushError::EventTooLong(len) => {
                write!(
                    f,
                    "Event with length {} is longer than the maximum length {}",
                    len, MAX_MIDI_MSG_SIZE,
                )
            }
        }
    }
}
