use std::fmt::format;

use eframe::{egui, epi};

use rusty_daw_io::{
    AudioBackend, AudioBackendInfo, AudioDeviceInfo, Config, DefaultChannelLayout, DeviceID,
    FixedBufferRangeMode, FixedBufferSizeRange, MidiBackend, MidiBackendInfo, MidiDeviceInfo,
    ProcessHandler, ProcessInfo, StreamHandle, StreamInfo,
};

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let app = DemoApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

pub struct DemoApp {
    stream_handle: Option<StreamHandle<DemoAppProcessHandler>>,

    available_audio_backends: Vec<(AudioBackend, Option<AudioBackendInfo>)>,
    available_midi_backends: Vec<(MidiBackend, Option<MidiBackendInfo>)>,

    configured_audio_in_ports: Vec<usize>,
    configured_audio_out_ports: Vec<usize>,

    selected_audio_backend_i: usize,
    selected_midi_backend_i: usize,
    selected_audio_device_i: usize,
    selected_sample_rate_i: usize,
    selected_buffer_size_i: usize,
    selected_buffer_size_value: u32,
}

impl DemoApp {
    pub fn new() -> Self {
        Self {
            stream_handle: None,

            available_audio_backends: Vec::new(),
            available_midi_backends: Vec::new(),

            configured_audio_in_ports: Vec::new(),
            configured_audio_out_ports: Vec::new(),

            selected_audio_backend_i: 0,
            selected_midi_backend_i: 0,
            selected_audio_device_i: 0,
            selected_sample_rate_i: 0,
            selected_buffer_size_i: 0,
            selected_buffer_size_value: 512,
        }
    }
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io settings GUI demo"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.available_audio_backends = vec![(
            AudioBackend::JackLinux,
            Some(AudioBackendInfo {
                backend: AudioBackend::JackLinux,
                version: Some(String::from("versionsss")),
                running: true,
                devices: vec![AudioDeviceInfo {
                    id: DeviceID { name: String::from("Jack System Device"), unique_id: None },
                    in_ports: vec![
                        String::from("mic_1"),
                        String::from("mic_2"),
                        String::from("port_3"),
                        String::from("port_4"),
                    ],
                    out_ports: vec![String::from("playback_1"), String::from("playback_2")],
                    sample_rates: vec![44100],
                    default_sample_rate: 44100,
                    fixed_buffer_size_range: Some(FixedBufferSizeRange {
                        mode: FixedBufferRangeMode::Range { min: 16, max: 2048 },
                        default: 512,
                    }),
                    default_input_layout: DefaultChannelLayout::Unspecified,
                    default_output_layout: DefaultChannelLayout::Unspecified,
                }],
                default_device: Some(0),
            }),
        )];

        self.configured_audio_in_ports = vec![0, 1];
        self.configured_audio_out_ports = vec![0, 1];

        dbg!(rusty_daw_io::available_audio_backends());
        dbg!(rusty_daw_io::available_midi_backends());
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ComboBox::from_label("Audio Backend")
                .selected_text(
                    self.available_audio_backends[self.selected_audio_backend_i].0.as_str(),
                )
                .show_ui(ui, |ui| {
                    for (i, backend) in self.available_audio_backends.iter().enumerate() {
                        ui.selectable_value(
                            &mut self.selected_audio_backend_i,
                            i,
                            backend.0.as_str(),
                        );
                    }
                });

            if let Some(audio_backend_info) =
                &self.available_audio_backends[self.selected_audio_backend_i].1
            {
                if audio_backend_info.backend.devices_are_relevant() {
                    ui.indent("audio_device", |ui| {
                        egui::ComboBox::from_label("Audio Device")
                            .selected_text(
                                &audio_backend_info.devices[self.selected_audio_device_i].id.name,
                            )
                            .show_ui(ui, |ui| {
                                for (i, device) in audio_backend_info.devices.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.selected_audio_device_i,
                                        i,
                                        &device.id.name,
                                    );
                                }
                            });
                    });
                }

                let audio_device_info = &audio_backend_info.devices[self.selected_audio_device_i];

                ui.indent("sample_rate", |ui| {
                    if audio_device_info.sample_rates.len() == 1 {
                        // Just show the user the configured sample rate.
                        ui.label(format!("sample rate: {}", audio_device_info.default_sample_rate));
                    } else {
                        egui::ComboBox::from_label("Sample Rate")
                            .selected_text(format!(
                                "{}",
                                audio_device_info.sample_rates[self.selected_sample_rate_i]
                            ))
                            .show_ui(ui, |ui| {
                                for (i, srate) in audio_device_info.sample_rates.iter().enumerate()
                                {
                                    ui.selectable_value(
                                        &mut self.selected_sample_rate_i,
                                        i,
                                        format!("{}", srate),
                                    );
                                }
                            });
                    }

                    if let Some(fixed_buffer_size_range) =
                        &audio_device_info.fixed_buffer_size_range
                    {
                        match &fixed_buffer_size_range.mode {
                            FixedBufferRangeMode::List(options) => {
                                if options.len() == 1 {
                                    // Just show the user the configured block size.
                                    ui.label(format!(
                                        "block size: {}",
                                        fixed_buffer_size_range.default
                                    ));
                                } else {
                                    egui::ComboBox::from_label("Block Size")
                                        .selected_text(format!(
                                            "{}",
                                            options[self.selected_buffer_size_i]
                                        ))
                                        .show_ui(ui, |ui| {
                                            for (i, size) in options.iter().enumerate() {
                                                ui.selectable_value(
                                                    &mut self.selected_buffer_size_i,
                                                    i,
                                                    format!("{}", size),
                                                );
                                            }
                                        });
                                }
                            }
                            FixedBufferRangeMode::Range { min, max } => {
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.selected_buffer_size_value,
                                        *min..=*max,
                                    )
                                    .text("Block Size"),
                                );
                            }
                        }
                    }
                });

                ui.separator();

                ui.label("Inputs");

                let mut remove_configured_port: Option<usize> = None;
                let mut try_change_port: Option<(usize, usize)> = None;
                let mut move_port_up: Option<usize> = None;
                let mut move_port_down: Option<usize> = None;
                let num_configured_ports = self.configured_audio_in_ports.len();
                for (i, selected_port) in self.configured_audio_in_ports.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let down_enabled = i < (num_configured_ports - 1);
                        ui.add_enabled_ui(down_enabled, |ui| {
                            if ui.button("v").clicked() {
                                move_port_down = Some(i);
                            }
                        });

                        let up_enabled = i > 0;
                        ui.add_enabled_ui(up_enabled, |ui| {
                            if ui.button("^").clicked() {
                                move_port_up = Some(i);
                            }
                        });

                        egui::ComboBox::from_id_source(format!("audio_in_port_{}", i))
                            .selected_text(format!(
                                "{}",
                                &audio_device_info.in_ports[*selected_port]
                            ))
                            .show_ui(ui, |ui| {
                                for (device_port_i, device_port_name) in
                                    audio_device_info.in_ports.iter().enumerate()
                                {
                                    let mut set_port = *selected_port;
                                    if ui
                                        .selectable_value(
                                            &mut set_port,
                                            device_port_i,
                                            format!("{}", device_port_name),
                                        )
                                        .changed()
                                    {
                                        try_change_port = Some((i, set_port));
                                    };
                                }
                            });

                        if ui.button("X").clicked() {
                            remove_configured_port = Some(i);
                        }
                    });
                }
                if let Some((i, new_port)) = try_change_port {
                    if can_select_port(new_port, &self.configured_audio_in_ports) {
                        self.configured_audio_in_ports[i] = new_port;
                    }
                }
                if let Some(i) = remove_configured_port {
                    self.configured_audio_in_ports.remove(i);
                }
                if let Some(i) = move_port_up {
                    self.configured_audio_in_ports.swap(i, i - 1);
                }
                if let Some(i) = move_port_down {
                    self.configured_audio_in_ports.swap(i, i + 1);
                }

                let add_enabled =
                    self.configured_audio_in_ports.len() < audio_device_info.in_ports.len();
                ui.add_enabled_ui(add_enabled, |ui| {
                    if ui.button("Add Input").clicked() {
                        if let Some(new_port) = find_next_available_port(
                            &self.configured_audio_in_ports,
                            audio_device_info.in_ports.len(),
                        ) {
                            self.configured_audio_in_ports.push(new_port);
                        }
                    }
                });

                ui.separator();

                ui.label("Outputs");

                let mut remove_configured_port: Option<usize> = None;
                let mut try_change_port: Option<(usize, usize)> = None;
                let mut move_port_up: Option<usize> = None;
                let mut move_port_down: Option<usize> = None;
                let num_configured_ports = self.configured_audio_out_ports.len();
                for (i, selected_port) in self.configured_audio_out_ports.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let down_enabled = i < (num_configured_ports - 1);
                        ui.add_enabled_ui(down_enabled, |ui| {
                            if ui.button("v").clicked() {
                                move_port_down = Some(i);
                            }
                        });

                        let up_enabled = i > 0;
                        ui.add_enabled_ui(up_enabled, |ui| {
                            if ui.button("^").clicked() {
                                move_port_up = Some(i);
                            }
                        });

                        egui::ComboBox::from_id_source(format!("audio_out_port_{}", i))
                            .selected_text(format!(
                                "{}",
                                &audio_device_info.out_ports[*selected_port]
                            ))
                            .show_ui(ui, |ui| {
                                for (device_port_i, device_port_name) in
                                    audio_device_info.out_ports.iter().enumerate()
                                {
                                    let mut set_port = *selected_port;
                                    if ui
                                        .selectable_value(
                                            &mut set_port,
                                            device_port_i,
                                            format!("{}", device_port_name),
                                        )
                                        .changed()
                                    {
                                        try_change_port = Some((i, set_port));
                                    };
                                }
                            });

                        if ui.button("X").clicked() {
                            remove_configured_port = Some(i);
                        }
                    });
                }
                if let Some((i, new_port)) = try_change_port {
                    if can_select_port(new_port, &self.configured_audio_out_ports) {
                        self.configured_audio_out_ports[i] = new_port;
                    }
                }
                if let Some(i) = remove_configured_port {
                    self.configured_audio_out_ports.remove(i);
                }
                if let Some(i) = move_port_up {
                    self.configured_audio_out_ports.swap(i, i - 1);
                }
                if let Some(i) = move_port_down {
                    self.configured_audio_out_ports.swap(i, i + 1);
                }

                let add_enabled =
                    self.configured_audio_out_ports.len() < audio_device_info.out_ports.len();
                ui.add_enabled_ui(add_enabled, |ui| {
                    if ui.button("Add Output").clicked() {
                        if let Some(new_port) = find_next_available_port(
                            &self.configured_audio_out_ports,
                            audio_device_info.out_ports.len(),
                        ) {
                            self.configured_audio_out_ports.push(new_port);
                        }
                    }
                });
            }
        });
    }
}

fn can_select_port(new_port: usize, configured_ports: &[usize]) -> bool {
    !configured_ports.contains(&new_port)
}

fn find_next_available_port(
    configured_ports: &[usize],
    num_available_ports: usize,
) -> Option<usize> {
    if configured_ports.len() >= num_available_ports {
        return None;
    }

    for i in 0..num_available_ports {
        if !configured_ports.contains(&i) {
            return Some(i);
        }
    }

    None
}

struct DemoAppProcessHandler {}

impl ProcessHandler for DemoAppProcessHandler {
    /// Initialize/allocate any buffers here. This will only be called once on
    /// creation.
    fn init(&mut self, stream_info: &StreamInfo) {}

    /// This gets called if the user made a change to the configuration that does not
    /// require restarting the audio thread.
    fn stream_changed(&mut self, stream_info: &StreamInfo) {}

    /// Process the current buffers. This will always be called on a realtime thread.
    fn process<'a>(&mut self, proc_info: ProcessInfo<'a>) {}
}
