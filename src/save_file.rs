use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::{Cursor, Write};

use std::fs::File;
use std::path::PathBuf;

pub const VERSION: &'static str = "0.1";

static XML_INDENT_SPACES: usize = 2;

use crate::{AudioBusConfig, AudioConfig, MidiConfig, MidiControllerConfig};

pub fn load_audio_config_from_file<P: Into<PathBuf>>(
    path: P,
) -> Result<AudioConfig, ConfigFileError> {
    let mut xml_reader = Reader::from_file(path.into())?;
    xml_reader.trim_text(true);

    let mut buf = Vec::new();

    let mut config = AudioConfig {
        server: String::new(),
        system_device: String::new(),

        in_busses: Vec::new(),
        out_busses: Vec::new(),

        sample_rate: None,
        buffer_size: None,
    };

    enum ReadState {
        Invalid,
        Server,
        SystemDevice,
        Port,
        SampleRate,
        BufferSize,
    }

    enum BusState {
        In,
        Out,
        Invalid,
    }

    let mut read_state = ReadState::Invalid;
    let mut bus_state = BusState::Invalid;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref event)) => match event.name() {
                b"server" => read_state = ReadState::Server,
                b"system_device" => read_state = ReadState::SystemDevice,
                b"in_busses" => bus_state = BusState::In,
                b"out_busses" => bus_state = BusState::Out,
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

                    match bus_state {
                        BusState::In => {
                            config.in_busses.push(AudioBusConfig {
                                id,
                                system_ports: Vec::new(),
                            });
                        }
                        BusState::Out => {
                            config.out_busses.push(AudioBusConfig {
                                id,
                                system_ports: Vec::new(),
                            });
                        }
                        BusState::Invalid => {
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
                let text = event.unescape_and_decode(&xml_reader)?;

                match &read_state {
                    ReadState::Server => config.server = text,
                    ReadState::SystemDevice => config.system_device = text,
                    ReadState::Port => {
                        let bus = match &bus_state {
                            BusState::In => config.in_busses.last_mut().ok_or_else(|| {
                                ConfigFileError::InvalidConfigFile(xml_reader.buffer_position())
                            })?,
                            BusState::Out => config.out_busses.last_mut().ok_or_else(|| {
                                ConfigFileError::InvalidConfigFile(xml_reader.buffer_position())
                            })?,
                            BusState::Invalid => {
                                return Err(ConfigFileError::InvalidConfigFile(
                                    xml_reader.buffer_position(),
                                ));
                            }
                        };

                        bus.system_ports.push(text);
                    }
                    ReadState::SampleRate => {
                        config.sample_rate = if &text == "auto" {
                            None
                        } else if let Ok(s) = text.parse::<u32>() {
                            Some(s)
                        } else {
                            None
                        };
                    }
                    ReadState::BufferSize => {
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

pub fn load_midi_config_from_file<P: Into<PathBuf>>(
    path: P,
) -> Result<MidiConfig, ConfigFileError> {
    let mut xml_reader = Reader::from_file(path.into())?;
    xml_reader.trim_text(true);

    let mut buf = Vec::new();

    let mut config = MidiConfig {
        server: String::new(),

        in_controllers: Vec::new(),
        out_controllers: Vec::new(),
    };

    enum ReadState {
        Invalid,
        Server,
        SystemPort,
    }

    enum ControllerState {
        In,
        Out,
        Invalid,
    }

    let mut read_state = ReadState::Invalid;
    let mut controller_state = ControllerState::Invalid;

    loop {
        match xml_reader.read_event(&mut buf) {
            Ok(Event::Start(ref event)) => match event.name() {
                b"server" => read_state = ReadState::Server,
                b"in_controllers" => controller_state = ControllerState::In,
                b"out_controllers" => controller_state = ControllerState::Out,
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

                    match controller_state {
                        ControllerState::In => {
                            config.in_controllers.push(MidiControllerConfig {
                                id,
                                system_port: String::new(),
                            });
                        }
                        ControllerState::Out => {
                            config.out_controllers.push(MidiControllerConfig {
                                id,
                                system_port: String::new(),
                            });
                        }
                        ControllerState::Invalid => {
                            return Err(ConfigFileError::InvalidConfigFile(
                                xml_reader.buffer_position(),
                            ));
                        }
                    }
                }
                b"system_port" => read_state = ReadState::SystemPort,
                _ => read_state = ReadState::Invalid,
            },
            Ok(Event::Text(ref event)) => {
                let text = event.unescape_and_decode(&xml_reader)?;

                match &read_state {
                    ReadState::Server => config.server = text,
                    ReadState::SystemPort => {
                        let controller = match &controller_state {
                            ControllerState::In => {
                                config.in_controllers.last_mut().ok_or_else(|| {
                                    ConfigFileError::InvalidConfigFile(xml_reader.buffer_position())
                                })?
                            }
                            ControllerState::Out => {
                                config.out_controllers.last_mut().ok_or_else(|| {
                                    ConfigFileError::InvalidConfigFile(xml_reader.buffer_position())
                                })?
                            }
                            ControllerState::Invalid => {
                                return Err(ConfigFileError::InvalidConfigFile(
                                    xml_reader.buffer_position(),
                                ));
                            }
                        };

                        controller.system_port = text;
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

pub fn write_audio_config_to_file<P: Into<PathBuf>>(
    path: P,
    config: &AudioConfig,
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

    let mut xml_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', XML_INDENT_SPACES);

    // Start

    let mut audio_config_elem = BytesStart::owned(b"audio_config".to_vec(), "audio_config".len());
    audio_config_elem.push_attribute(("version", VERSION));
    xml_writer.write_event(Event::Start(audio_config_elem))?;

    // Server

    let server_elem = BytesStart::owned(b"server".to_vec(), "server".len());
    xml_writer.write_event(Event::Start(server_elem))?;
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&config.server)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"server")))?;

    // System Device

    let server_device_elem = BytesStart::owned(b"system_device".to_vec(), "system_device".len());
    xml_writer.write_event(Event::Start(server_device_elem))?;
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(
        &config.system_device,
    )))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"system_device")))?;

    // Audio In Busses

    let in_busses_elem = BytesStart::owned(b"in_busses".to_vec(), "in_busses".len());
    xml_writer.write_event(Event::Start(in_busses_elem))?;
    for bus in config.in_busses.iter() {
        write_audio_bus_config(&mut xml_writer, bus)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"in_busses")))?;

    // Audio Out Busses

    let out_busses_elem = BytesStart::owned(b"out_busses".to_vec(), "out_busses".len());
    xml_writer.write_event(Event::Start(out_busses_elem))?;
    for bus in config.out_busses.iter() {
        write_audio_bus_config(&mut xml_writer, bus)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"out_busses")))?;

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

    // End

    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"audio_config")))?;

    let xml_result = xml_writer.into_inner().into_inner();

    let mut file = File::create(path.into())?;
    file.write_all(&xml_result)?;

    Ok(())
}

pub fn write_midi_config_to_file<P: Into<PathBuf>>(
    path: P,
    config: &MidiConfig,
) -> Result<(), ConfigFileError> {
    let write_midi_controller_config = |xml_writer: &mut Writer<Cursor<Vec<u8>>>,
                                        controller: &MidiControllerConfig|
     -> Result<(), ConfigFileError> {
        let mut controller_elem = BytesStart::owned(b"controller".to_vec(), "controller".len());
        controller_elem.push_attribute(("id", controller.id.as_str()));
        xml_writer.write_event(Event::Start(controller_elem))?;

        // System Port
        let system_port_elem = BytesStart::owned(b"system_port".to_vec(), "system_port".len());
        xml_writer.write_event(Event::Start(system_port_elem))?;
        xml_writer.write_event(Event::Text(BytesText::from_plain_str(
            &controller.system_port,
        )))?;
        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"system_port")))?;

        xml_writer.write_event(Event::End(BytesEnd::borrowed(b"controller")))?;

        Ok(())
    };

    let mut xml_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', XML_INDENT_SPACES);

    // Start

    let mut midi_config_elem = BytesStart::owned(b"midi_config".to_vec(), "midi_config".len());
    midi_config_elem.push_attribute(("version", VERSION));
    xml_writer.write_event(Event::Start(midi_config_elem))?;

    // Server

    let server_elem = BytesStart::owned(b"server".to_vec(), "server".len());
    xml_writer.write_event(Event::Start(server_elem))?;
    xml_writer.write_event(Event::Text(BytesText::from_plain_str(&config.server)))?;
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"server")))?;

    // In Controllers

    let in_controllers_elem = BytesStart::owned(b"in_controllers".to_vec(), "in_controllers".len());
    xml_writer.write_event(Event::Start(in_controllers_elem))?;
    for controller in config.in_controllers.iter() {
        write_midi_controller_config(&mut xml_writer, controller)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"in_controllers")))?;

    // Out Controllers

    let out_controllers_elem =
        BytesStart::owned(b"out_controllers".to_vec(), "out_controllers".len());
    xml_writer.write_event(Event::Start(out_controllers_elem))?;
    for controller in config.out_controllers.iter() {
        write_midi_controller_config(&mut xml_writer, controller)?;
    }
    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"out_controllers")))?;

    // End

    xml_writer.write_event(Event::End(BytesEnd::borrowed(b"midi_config")))?;

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
    fn write_and_load_audio_config() {
        let audio_config = AudioConfig {
            server: String::from("Jack"),
            system_device: String::from("Jack Server"),

            in_busses: vec![
                AudioBusConfig {
                    id: String::from("Mic #1"),
                    system_ports: vec![String::from("system:capture_1")],
                },
                AudioBusConfig {
                    id: String::from("Mic #2"),
                    system_ports: vec![String::from("system:capture_2")],
                },
            ],

            out_busses: vec![
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

            sample_rate: Some(44100),
            buffer_size: None,
        };

        write_audio_config_to_file("test_audio_config.xml", &audio_config).unwrap();

        let read_config = load_audio_config_from_file("test_audio_config.xml").unwrap();

        assert_eq!(audio_config, read_config);
    }

    #[test]
    fn write_and_load_midi_config() {
        let midi_config = MidiConfig {
            server: String::from("Jack"),

            in_controllers: vec![
                MidiControllerConfig {
                    id: String::from("Midi In #1"),
                    system_port: String::from("system:midi_capture_1"),
                },
                MidiControllerConfig {
                    id: String::from("Midi In #2"),
                    system_port: String::from("system:midi_capture_2"),
                },
            ],

            out_controllers: vec![
                MidiControllerConfig {
                    id: String::from("Midi Out #1"),
                    system_port: String::from("system:midi_playback_1"),
                },
                MidiControllerConfig {
                    id: String::from("Midi Out #2"),
                    system_port: String::from("system:midi_playback_2"),
                },
            ],
        };

        write_midi_config_to_file("test_midi_config.xml", &midi_config).unwrap();

        let read_config = load_midi_config_from_file("test_midi_config.xml").unwrap();

        assert_eq!(midi_config, read_config);
    }
}
