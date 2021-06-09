use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::{Cursor, Write};

use std::fs::File;
use std::path::PathBuf;

pub const VERSION: &'static str = "0.1";

static XML_INDENT_SPACES: usize = 3;

use crate::{AudioBusConfig, Config, MidiControllerConfig};

pub fn load_config_from_file<P: Into<PathBuf>>(path: P) -> Result<Config, ConfigFileError> {
    let mut xml_reader = Reader::from_file(path.into())?;
    xml_reader.trim_text(true);

    let mut buf = Vec::new();

    let mut config = Config {
        audio_server: String::new(),
        system_audio_device: String::new(),

        audio_in_busses: Vec::new(),
        audio_out_busses: Vec::new(),

        sample_rate: None,
        buffer_size: None,

        midi_server: None,

        midi_in_controllers: Vec::new(),
        midi_out_controllers: Vec::new(),
    };

    enum ReadState {
        Invalid,
        AudioServer,
        MidiServer,
        SystemAudioDevice,
        Port,
        SampleRate,
        BufferSize,
    }

    enum BusControllerState {
        AudioIn,
        AudioOut,
        MidiIn,
        MidiOut,
        Invalid,
    }

    let mut read_state = ReadState::Invalid;
    let mut bus_controller_state = BusControllerState::Invalid;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref event)) => match event.name() {
                b"audio_server" => read_state = ReadState::AudioServer,
                b"midi_server" => read_state = ReadState::MidiServer,
                b"system_audio_device" => read_state = ReadState::SystemAudioDevice,
                b"audio_in_busses" => bus_controller_state = BusControllerState::AudioIn,
                b"audio_out_busses" => bus_controller_state = BusControllerState::AudioOut,
                b"bus" => {
                    let mut id = String::new();
                    for a in event.attributes() {
                        let a = a?;
                        if a.key == b"id" {
                            id = String::from_utf8(a.value.to_vec()).map_err(|_| {
                                ConfigFileError::FailedToParseUTF8(a.value.to_vec())
                            })?;
                            break;
                        }
                    }

                    match bus_controller_state {
                        BusControllerState::AudioIn => {
                            config.audio_in_busses.push(AudioBusConfig {
                                id,
                                system_ports: Vec::new(),
                            });
                        }
                        BusControllerState::AudioOut => {
                            config.audio_out_busses.push(AudioBusConfig {
                                id,
                                system_ports: Vec::new(),
                            });
                        }
                        _ => {
                            return Err(ConfigFileError::InvalidConfigFile(
                                xml_reader.buffer_position(),
                            ));
                        }
                    }
                }
                b"midi_in_controllers" => bus_controller_state = BusControllerState::MidiIn,
                b"midi_out_controllers" => bus_controller_state = BusControllerState::MidiOut,
                b"controller" => {
                    let mut id = String::new();
                    for a in event.attributes() {
                        let a = a?;
                        if a.key == b"id" {
                            id = String::from_utf8(a.value.to_vec()).map_err(|_| {
                                ConfigFileError::FailedToParseUTF8(a.value.to_vec())
                            })?;
                            break;
                        }
                    }

                    match bus_controller_state {
                        BusControllerState::MidiIn => {
                            config.midi_in_controllers.push(MidiControllerConfig {
                                id,
                                system_port: String::new(),
                            });
                        }
                        BusControllerState::MidiOut => {
                            config.midi_out_controllers.push(MidiControllerConfig {
                                id,
                                system_port: String::new(),
                            });
                        }
                        _ => {
                            return Err(ConfigFileError::InvalidConfigFile(
                                xml_reader.buffer_position(),
                            ));
                        }
                    }
                }
                b"port" => read_state = ReadState::Port,
                b"sample_rate" => read_state = ReadState::SampleRate,
                b"buffer_size" => read_state = ReadState::BufferSize,
                _ => read_state = ReadState::Invalid,
            },
            Ok(Event::Text(ref event)) => {
                let mut text = event.unescape_and_decode(&xml_reader)?;

                match &read_state {
                    ReadState::AudioServer => config.audio_server = text,
                    ReadState::MidiServer => {
                        let mut temp_text = text.clone();
                        temp_text.make_ascii_lowercase();

                        if temp_text == "none" {
                            config.midi_server = None;
                        } else {
                            config.midi_server = Some(text);
                        }
                    }
                    ReadState::SystemAudioDevice => config.system_audio_device = text,
                    ReadState::Port => {
                        match &bus_controller_state {
                            BusControllerState::AudioIn => {
                                config
                                    .audio_in_busses
                                    .last_mut()
                                    .ok_or_else(|| {
                                        ConfigFileError::InvalidConfigFile(
                                            xml_reader.buffer_position(),
                                        )
                                    })?
                                    .system_ports
                                    .push(text);
                            }
                            BusControllerState::AudioOut => {
                                config
                                    .audio_out_busses
                                    .last_mut()
                                    .ok_or_else(|| {
                                        ConfigFileError::InvalidConfigFile(
                                            xml_reader.buffer_position(),
                                        )
                                    })?
                                    .system_ports
                                    .push(text);
                            }
                            BusControllerState::MidiIn => {
                                config
                                    .midi_in_controllers
                                    .last_mut()
                                    .ok_or_else(|| {
                                        ConfigFileError::InvalidConfigFile(
                                            xml_reader.buffer_position(),
                                        )
                                    })?
                                    .system_port = text;
                            }
                            BusControllerState::MidiOut => {
                                config
                                    .midi_out_controllers
                                    .last_mut()
                                    .ok_or_else(|| {
                                        ConfigFileError::InvalidConfigFile(
                                            xml_reader.buffer_position(),
                                        )
                                    })?
                                    .system_port = text;
                            }
                            BusControllerState::Invalid => {
                                return Err(ConfigFileError::InvalidConfigFile(
                                    xml_reader.buffer_position(),
                                ));
                            }
                        };
                    }
                    ReadState::SampleRate => {
                        text.make_ascii_lowercase();

                        config.sample_rate = if &text == "auto" {
                            None
                        } else if let Ok(s) = text.parse::<u32>() {
                            Some(s)
                        } else {
                            None
                        };
                    }
                    ReadState::BufferSize => {
                        text.make_ascii_lowercase();

                        config.buffer_size = if &text == "auto" {
                            None
                        } else if let Ok(s) = text.parse::<u32>() {
                            Some(s)
                        } else {
                            None
                        };
                    }
                    ReadState::Invalid => (),
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConfigFileError::Xml(e)),
            _ => (),
        }

        buf.clear();
    }

    Ok(config)
}

pub fn write_config_to_file<P: Into<PathBuf>>(
    path: P,
    config: &Config,
) -> Result<(), ConfigFileError> {
    let write_audio_bus_config = |xml_writer: &mut Writer<Cursor<Vec<u8>>>,
                                  bus: &AudioBusConfig|
     -> Result<(), ConfigFileError> {
        let mut bus_elem = BytesStart::owned(b"bus".to_vec(), "bus".len());
        bus_elem.push_attribute(("id", bus.id.as_str()));
        xml_writer.write_event(Event::Start(bus_elem))?;

        // System Ports
        let system_ports_elem = BytesStart::owned(b"system_ports".to_vec(), "system_ports".len());
        xml_writer.write_event(Event::Start(system_ports_elem))?;
        for port in bus.system_ports.iter() {
            let port_elem = BytesStart::owned(b"port".to_vec(), "port".len());
            xml_writer.write_event(Event::Start(port_elem))?;
            xml_writer.write_event(Event::Text(BytesText::from_plain_str(&port)))?;
            xml_writer.write_event(Event::End(BytesEnd::borrowed(b"port")))?;
        }
        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"system_ports")))?;

        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"bus")))?;

        Ok(())
    };

    let write_midi_controller_config = |xml_writer: &mut Writer<Cursor<Vec<u8>>>,
                                        controller: &MidiControllerConfig|
     -> Result<(), ConfigFileError> {
        let mut controller_elem = BytesStart::owned(b"controller".to_vec(), "controller".len());
        controller_elem.push_attribute(("id", controller.id.as_str()));
        xml_writer.write_event(Event::Start(controller_elem))?;

        // System Port
        let port_elem = BytesStart::owned(b"port".to_vec(), "port".len());
        xml_writer.write_event(Event::Start(port_elem))?;
        xml_writer.write_event(Event::Text(BytesText::from_plain_str(
            &controller.system_port,
        )))?;
        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"port")))?;

        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"controller")))?;

        Ok(())
    };

    let mut xml_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', XML_INDENT_SPACES);

    // Start

    let mut config_elem = BytesStart::owned(b"config".to_vec(), "config".len());
    config_elem.push_attribute(("version", VERSION));
    xml_writer.write_event(Event::Start(config_elem))?;

    // Audio Server

    let server_elem = BytesStart::owned(b"audio_server".to_vec(), "audio_server".len());
    xml_writer.write_event(Event::Start(server_elem))?;
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&config.audio_server)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"audio_server")))?;

    // System Audio Device

    let audio_device_elem =
        BytesStart::owned(b"system_audio_device".to_vec(), "system_audio_device".len());
    xml_writer.write_event(Event::Start(audio_device_elem))?;
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(
        &config.system_audio_device,
    )))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"system_audio_device")))?;

    // Audio Out Busses

    let audio_out_busses_elem =
        BytesStart::owned(b"audio_out_busses".to_vec(), "audio_out_busses".len());
    xml_writer.write_event(Event::Start(audio_out_busses_elem))?;
    for bus in config.audio_out_busses.iter() {
        write_audio_bus_config(&mut xml_writer, bus)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"audio_out_busses")))?;

    // Audio In Busses

    let audio_in_busses_elem =
        BytesStart::owned(b"audio_in_busses".to_vec(), "audio_in_busses".len());
    xml_writer.write_event(Event::Start(audio_in_busses_elem))?;
    for bus in config.audio_in_busses.iter() {
        write_audio_bus_config(&mut xml_writer, bus)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"audio_in_busses")))?;

    // Sample Rate

    let sample_rate_elem = BytesStart::owned(b"sample_rate".to_vec(), "sample_rate".len());
    xml_writer.write_event(Event::Start(sample_rate_elem))?;
    let t = if let Some(sample_rate) = config.sample_rate {
        format!("{}", sample_rate)
    } else {
        String::from("auto")
    };
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&t)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"sample_rate")))?;

    // Buffer Size

    let buffer_size_elem = BytesStart::owned(b"buffer_size".to_vec(), "buffer_size".len());
    xml_writer.write_event(Event::Start(buffer_size_elem))?;
    let t = if let Some(buffer_size) = config.buffer_size {
        format!("{}", buffer_size)
    } else {
        String::from("auto")
    };
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&t)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"buffer_size")))?;

    // Midi Server

    let midi_server_elem = BytesStart::owned(b"midi_server".to_vec(), "midi_server".len());
    xml_writer.write_event(Event::Start(midi_server_elem))?;
    let t = if let Some(midi_server) = &config.midi_server {
        midi_server.clone()
    } else {
        String::from("none")
    };
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&t)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"midi_server")))?;

    // In Controllers

    let midi_in_controllers_elem =
        BytesStart::owned(b"midi_in_controllers".to_vec(), "midi_in_controllers".len());
    xml_writer.write_event(Event::Start(midi_in_controllers_elem))?;
    for controller in config.midi_in_controllers.iter() {
        write_midi_controller_config(&mut xml_writer, controller)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"midi_in_controllers")))?;

    // Out Controllers

    let midi_out_controllers_elem = BytesStart::owned(
        b"midi_out_controllers".to_vec(),
        "midi_out_controllers".len(),
    );
    xml_writer.write_event(Event::Start(midi_out_controllers_elem))?;
    for controller in config.midi_out_controllers.iter() {
        write_midi_controller_config(&mut xml_writer, controller)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"midi_out_controllers")))?;

    // End

    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"config")))?;

    let xml_result = xml_writer.into_inner().into_inner();

    let mut file = File::create(path.into())?;
    file.write_all(&xml_result)?;

    Ok(())
}

#[derive(Debug)]
pub enum ConfigFileError {
    Xml(quick_xml::Error),
    File(std::io::Error),
    InvalidConfigFile(usize),
    FailedToParseUTF8(Vec<u8>),
}

impl std::error::Error for ConfigFileError {}

impl std::fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigFileError::Xml(e) => {
                write!(f, "XML error: {}", e)
            }
            ConfigFileError::File(e) => {
                write!(f, "File error: {:?}", e)
            }
            ConfigFileError::InvalidConfigFile(pos) => {
                write!(f, "Invalid config file. Error detected at position {}", pos)
            }
            ConfigFileError::FailedToParseUTF8(bytes) => {
                write!(f, "Failed to parse UTF-8 string. Bytes: {:?}", bytes)
            }
        }
    }
}

impl From<quick_xml::Error> for ConfigFileError {
    fn from(e: quick_xml::Error) -> Self {
        ConfigFileError::Xml(e)
    }
}

impl From<std::io::Error> for ConfigFileError {
    fn from(e: std::io::Error) -> Self {
        ConfigFileError::File(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_load_config() {
        let config = Config {
            audio_server: String::from("Jack"),
            system_audio_device: String::from("Jack Server"),

            audio_in_busses: vec![
                AudioBusConfig {
                    id: String::from("Mic #1"),
                    system_ports: vec![String::from("system:capture_1")],
                },
                AudioBusConfig {
                    id: String::from("Mic #2"),
                    system_ports: vec![String::from("system:capture_2")],
                },
            ],

            audio_out_busses: vec![
                AudioBusConfig {
                    id: String::from("Speaker #1"),
                    system_ports: vec![
                        String::from("system:playback_1"),
                        String::from("system:playback_2"),
                    ],
                },
                AudioBusConfig {
                    id: String::from("Speaker #2"),
                    system_ports: vec![
                        String::from("system:playback_3"),
                        String::from("system:playback_4"),
                    ],
                },
            ],

            midi_server: Some(String::from("Jack")),

            midi_in_controllers: vec![
                MidiControllerConfig {
                    id: String::from("Midi In #1"),
                    system_port: String::from("system:midi_capture_1"),
                },
                MidiControllerConfig {
                    id: String::from("Midi In #2"),
                    system_port: String::from("system:midi_capture_2"),
                },
            ],

            midi_out_controllers: vec![
                MidiControllerConfig {
                    id: String::from("Midi Out #1"),
                    system_port: String::from("system:midi_playback_1"),
                },
                MidiControllerConfig {
                    id: String::from("Midi Out #2"),
                    system_port: String::from("system:midi_playback_2"),
                },
            ],

            sample_rate: Some(44100),
            buffer_size: None,
        };

        write_config_to_file("test_config.xml", &config).unwrap();

        let read_config = load_config_from_file("test_config.xml").unwrap();

        assert_eq!(config, read_config);
    }
}
