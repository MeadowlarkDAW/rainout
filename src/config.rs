#[derive(Debug, Clone, PartialEq)]
pub struct AudioBusConfig {
    /// The ID to use for this bus. This ID is for the "internal" bus that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" bus is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusConfig` and `MidiControllerConfig`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id: String,

    /// The ports (of the system device) that this bus will be connected to.
    pub system_ports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiControllerConfig {
    /// The ID to use for this controller. This ID is for the "internal" controller that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" controller is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusConfig` and `MidiControllerConfig`.
    pub id: String,

    /// The name of the system port this controller is connected to.
    pub system_port: String,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Config {
    /// The name of the audio server to use.
    pub audio_server: String,

    /// The name of the system audio device to use.
    pub system_audio_device: String,

    /// The audio input busses to create/use. These are the "internal" busses that appear to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware device that these "internal" busses are connected to. This will return an error
    /// if the system device is playback only.
    ///
    /// Examples of device IDs can include:
    ///
    /// * Realtek Device In
    /// * Built-In Mic
    /// * Drums Mic
    pub audio_in_busses: Vec<AudioBusConfig>,

    /// The audio output busses to create/use. These are the "internal" busses that appear to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware device that these "internal" busses are connected to.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device Stereo Out
    /// * Headphones Out
    /// * Speakers Out
    pub audio_out_busses: Vec<AudioBusConfig>,

    /// The sample rate to use.
    ///
    /// Set this to `None` to use the default sample rate of the system device.
    pub sample_rate: Option<u32>,

    /// The maximum number of frames per channel.
    ///
    /// Set this to `None` to use the default settings of the system device.
    pub buffer_size: Option<u32>,

    /// The name of the midi server to use.
    ///
    /// Set this to `None` for no midi.
    pub midi_server: Option<String>,

    /// The midi input controllers to create/use. These are the "internal" controllers that appear to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" controllers are connected to.
    pub midi_in_controllers: Vec<MidiControllerConfig>,

    /// The midi output controllers to create/use. These are the "internal" controllers that appear to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" controllers are connected to.
    pub midi_out_controllers: Vec<MidiControllerConfig>,
}
