#[cfg(feature = "midi")]
use crate::MidiBuffer;

/// The audio and MIDI buffers for this process cycle.
pub struct ProcessInfo<'a> {
    /// The audio input buffers.
    pub audio_inputs: &'a [Vec<f32>],

    /// The audio output buffers.
    pub audio_outputs: &'a mut [Vec<f32>],

    /// The number of audio frames in this process cycle.
    ///
    /// It is gauranteed that every buffer in `audio_inputs` and
    /// `audio_outputs` will have a length of at-least this size.
    pub frames: usize,

    /// For each audio input buffer in order, this will return true
    /// if every sample in that buffer is `0.0`, false otherwise.
    ///
    /// This is only relevant if this stream was run with
    /// `RunOptions::check_for_silent_inputs` set to true, which it
    /// is not on by default. If `RunOptions::check_for_silent_inputs`
    /// is false, then these values will always be false.
    pub silent_audio_inputs: &'a [bool],

    #[cfg(feature = "midi")]
    /// The MIDI input buffers.
    pub midi_inputs: &'a [MidiBuffer],

    #[cfg(feature = "midi")]
    /// The MIDI output buffers.
    pub midi_outputs: &'a mut [MidiBuffer],
}
