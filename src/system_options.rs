use crate::{AudioBusConfig, Config, DevicesInfo, MidiControllerConfig};

#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    pub audio_server_options: Vec<String>,
    pub current_audio_server_index: usize,
    pub current_audio_server_name: String,

    pub midi_server_options: Vec<String>,
    pub current_midi_server_index: usize,
    pub current_midi_server_name: String,

    pub audio_device_options: Vec<String>,
    pub current_audio_device_index: usize,
    pub current_audio_device_name: String,

    pub sample_rate_options: Vec<u32>,
    pub current_sample_rate_index: usize,
    pub current_sample_rate_str: String,

    pub buffer_size_range: BufferSizeRange,
    pub current_buffer_size: u32,
    pub current_buffer_size_str: String,

    pub audio_in_system_port_options: Vec<String>,
    pub audio_out_system_port_options: Vec<String>,

    pub midi_in_system_port_options: Vec<String>,
    pub midi_out_system_port_options: Vec<String>,

    pub audio_in_busses: Vec<AudioBusDisplayState>,
    pub audio_out_busses: Vec<AudioBusDisplayState>,

    pub midi_in_controllers: Vec<MidiControllerDisplayState>,
    pub midi_out_controllers: Vec<MidiControllerDisplayState>,

    pub current_audio_server_available: bool,
    pub current_midi_server_available: bool,
    pub playback_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemPortDisplayState {
    pub current_system_port_index: usize,
    pub current_system_port_name: String,
    pub can_remove: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioBusDisplayState {
    /// The ID to use for this bus. This ID is for the "internal" bus that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" bus is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusDisplayState` and `MidiControllerDisplayState`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id: String,

    /// The ports (of the system device) that this bus will be connected to.
    pub ports: Vec<SystemPortDisplayState>,

    pub can_remove: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiControllerDisplayState {
    /// The ID to use for this controller. This ID is for the "internal" controller that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" controller is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusDisplayState` and `MidiControllerDisplayState`.
    pub id: String,

    /// The name of the system port this controller is connected to.
    pub system_port: SystemPortDisplayState,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct BufferSizeRange {
    pub min: u32,
    pub max: u32,
}

pub struct SystemOptions {
    devices_info: DevicesInfo,

    display_state: DisplayState,

    // For loading the default config
    default_audio_server: usize,
    default_midi_server: usize,

    config_status: ConfigStatus,
    do_build_config: bool,
}

impl SystemOptions {
    pub fn new() -> Self {
        let devices_info = DevicesInfo::new();

        let mut default_audio_server = 0;
        for (i, server) in devices_info.audio_servers_info().iter().enumerate() {
            if &server.name == devices_info.default_audio_server() {
                default_audio_server = i;
                break;
            }
        }

        let mut default_midi_server = 0;
        for (i, server) in devices_info.midi_servers_info().iter().enumerate() {
            if &server.name == devices_info.default_midi_server() {
                default_midi_server = i;
                break;
            }
        }

        let mut new_self = Self {
            devices_info,

            display_state: DisplayState::default(),

            default_audio_server,
            default_midi_server,

            config_status: ConfigStatus::default(),
            do_build_config: false,
        };

        new_self.display_state.audio_server_options = new_self
            .devices_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        new_self.display_state.midi_server_options = new_self
            .devices_info
            .midi_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        new_self.set_audio_defaults();
        new_self.set_midi_defaults();

        new_self.build_config();
        new_self.do_build_config = true;

        new_self
    }

    pub fn devices_info(&self) -> &DevicesInfo {
        &self.devices_info
    }

    pub fn select_audio_server(&mut self, index: usize) {
        let index = index.min(self.display_state.audio_server_options.len() - 1);
        if self.display_state.current_audio_server_index != index {
            self.display_state.current_audio_server_index = index;
            self.display_state.current_audio_server_name = self.display_state.audio_server_options
                [self.display_state.current_audio_server_index]
                .clone();

            self.display_state.audio_device_options = self.devices_info.audio_servers_info()
                [self.display_state.current_audio_server_index]
                .devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            self.display_state.current_audio_server_available = self
                .devices_info
                .audio_servers_info()[self.display_state.current_audio_server_index]
                .available;

            self.set_defaults_for_current_audio_server();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn select_audio_device(&mut self, index: usize) {
        if self.display_state.audio_device_options.len() > 0 {
            let index = index.min(self.display_state.audio_device_options.len() - 1);
            if self.display_state.current_audio_device_index != index {
                self.display_state.current_audio_device_index = index;

                self.display_state.current_audio_device_name = self
                    .display_state
                    .audio_device_options
                    .get(self.display_state.current_audio_device_index)
                    .unwrap_or(&String::from("Unavailable"))
                    .clone();

                self.set_defaults_for_current_audio_device();

                if self.do_build_config {
                    self.build_config();
                }
            }
        } else {
            self.display_state.playback_only = false;
        }
    }

    pub fn select_sample_rate(&mut self, index: usize) {
        if self.display_state.sample_rate_options.len() > 0 {
            let index = index.min(self.display_state.sample_rate_options.len() - 1);
            if self.display_state.current_sample_rate_index != index {
                self.display_state.current_sample_rate_index = index;

                self.display_state.current_sample_rate_str = format!(
                    "{}",
                    self.display_state.sample_rate_options
                        [self.display_state.current_sample_rate_index]
                );

                if self.do_build_config {
                    self.build_config();
                }
            }
        } else {
            self.display_state.current_sample_rate_str = String::from("Unavailable");
        }
    }

    pub fn select_buffer_size(&mut self, size: u32) {
        if self
            .display_state
            .audio_device_options
            .get(self.display_state.current_audio_device_index)
            .is_some()
        {
            let size = size
                .min(self.display_state.buffer_size_range.min)
                .max(self.display_state.buffer_size_range.max);

            if self.display_state.current_buffer_size != size {
                self.display_state.current_buffer_size = size;

                self.display_state.current_sample_rate_str = format!(
                    "{}",
                    self.display_state.sample_rate_options
                        [self.display_state.current_sample_rate_index]
                );

                if self.do_build_config {
                    self.build_config();
                }
            }
        } else {
            self.display_state.current_buffer_size_str = String::from("Unavailable");
        }
    }

    pub fn select_auto_buffer_size(&mut self) {
        if self
            .display_state
            .audio_device_options
            .get(self.display_state.current_audio_device_index)
            .is_some()
        {
            let size = self.devices_info.audio_servers_info()
                [self.display_state.current_audio_server_index]
                .devices[self.display_state.current_audio_device_index]
                .default_buffer_size
                .min(self.display_state.buffer_size_range.min)
                .max(self.display_state.buffer_size_range.max);

            if self.display_state.current_buffer_size != size {
                self.display_state.current_buffer_size = size;

                self.display_state.current_sample_rate_str = format!(
                    "{}",
                    self.display_state.sample_rate_options
                        [self.display_state.current_sample_rate_index]
                );

                if self.do_build_config {
                    self.build_config();
                }
            }
        } else {
            self.display_state.current_buffer_size_str = String::from("Unavailable");
        }
    }

    pub fn remove_audio_in_bus(&mut self, index: usize) {
        let mut do_remove = false;
        if let Some(bus) = self.display_state.audio_in_busses.get(index) {
            do_remove = bus.can_remove;
        }

        if do_remove {
            self.display_state.audio_in_busses.remove(index);

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn remove_audio_out_bus(&mut self, index: usize) {
        let mut do_remove = false;
        if let Some(bus) = self.display_state.audio_out_busses.get(index) {
            do_remove = bus.can_remove;
        }

        if do_remove {
            self.display_state.audio_out_busses.remove(index);

            // If only one audio out bus is left, mark that it cannot be removed.
            if self.display_state.audio_out_busses.len() == 1 {
                self.display_state.audio_out_busses[0].can_remove = false;
            }

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn add_audio_in_bus(&mut self) {
        if self.display_state.audio_in_system_port_options.len() > 0 {
            // Find the next available ID.
            let mut id = String::from("Mic In");
            let mut i = 1;
            loop {
                let mut is_unique = true;
                for bus in self.display_state.audio_in_busses.iter() {
                    if &id == &bus.id {
                        is_unique = false;
                        break;
                    }
                }
                if is_unique {
                    break;
                } else {
                    i += 1;
                    id = format!("Mic In #{}", i);
                }
            }

            // Find the next available port.
            let mut next_port = self
                .display_state
                .audio_in_busses
                .last()
                .map(|b| {
                    b.ports
                        .last()
                        .map(|p| p.current_system_port_index + 1)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            if next_port >= self.display_state.audio_in_system_port_options.len() {
                next_port = 0;
            }

            self.display_state
                .audio_in_busses
                .push(AudioBusDisplayState {
                    id,
                    ports: vec![SystemPortDisplayState {
                        current_system_port_index: next_port,
                        current_system_port_name: self.display_state.audio_in_system_port_options
                            [next_port]
                            .clone(),
                        can_remove: false,
                    }],
                    can_remove: true,
                });

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn add_audio_out_bus(&mut self) {
        if self.display_state.audio_out_system_port_options.len() > 0 {
            // Find the next available ID.
            let mut id = String::from("Speakers Out");
            let mut i = 1;
            loop {
                let mut is_unique = true;
                for bus in self.display_state.audio_out_busses.iter() {
                    if &id == &bus.id {
                        is_unique = false;
                        break;
                    }
                }
                if is_unique {
                    break;
                } else {
                    i += 1;
                    id = format!("Speakers Out #{}", i);
                }
            }

            // Find the next available ports.
            let mut next_port_left = self
                .display_state
                .audio_out_busses
                .last()
                .map(|b| {
                    b.ports
                        .last()
                        .map(|p| p.current_system_port_index + 1)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let mut next_port_right = next_port_left + 1;
            if next_port_left >= self.display_state.audio_out_system_port_options.len() {
                next_port_left = 0;
            }
            if next_port_right >= self.display_state.audio_out_system_port_options.len() {
                next_port_right = 0;
            }

            self.display_state
                .audio_out_busses
                .push(AudioBusDisplayState {
                    id,
                    ports: vec![
                        SystemPortDisplayState {
                            current_system_port_index: next_port_left,
                            current_system_port_name: self
                                .display_state
                                .audio_out_system_port_options[next_port_left]
                                .clone(),
                            can_remove: true,
                        },
                        SystemPortDisplayState {
                            current_system_port_index: next_port_right,
                            current_system_port_name: self
                                .display_state
                                .audio_out_system_port_options[next_port_right]
                                .clone(),
                            can_remove: true,
                        },
                    ],
                    can_remove: false,
                });

            // If there is more than one output bus, mark all of them as removeable.
            if self.display_state.audio_out_busses.len() > 1 {
                for bus in self.display_state.audio_out_busses.iter_mut() {
                    bus.can_remove = true;
                }
            }

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn rename_audio_in_bus<S: Into<String>>(&mut self, bus_index: usize, name: S) {
        if let Some(bus) = self.display_state.audio_in_busses.get_mut(bus_index) {
            bus.id = name.into();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn rename_audio_out_bus<S: Into<String>>(&mut self, bus_index: usize, name: S) {
        if let Some(bus) = self.display_state.audio_out_busses.get_mut(bus_index) {
            bus.id = name.into();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn remove_audio_in_bus_port(&mut self, bus_index: usize, port_index: usize) {
        if let Some(bus) = self.display_state.audio_in_busses.get_mut(bus_index) {
            let mut do_remove = false;
            if let Some(port) = bus.ports.get(port_index) {
                do_remove = port.can_remove;
            }

            if do_remove {
                bus.ports.remove(port_index);

                // If there is only one port left, mark that it cannot be removed.
                if bus.ports.len() == 1 {
                    bus.ports[0].can_remove = false;
                }

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn remove_audio_out_bus_port(&mut self, bus_index: usize, port_index: usize) {
        if let Some(bus) = self.display_state.audio_out_busses.get_mut(bus_index) {
            let mut do_remove = false;
            if let Some(port) = bus.ports.get(port_index) {
                do_remove = port.can_remove;
            }

            if do_remove {
                bus.ports.remove(port_index);

                // If there is only one port left, mark that it cannot be removed.
                if bus.ports.len() == 1 {
                    bus.ports[0].can_remove = false;
                }

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn add_audio_in_bus_port(&mut self, bus_index: usize) {
        if self.display_state.audio_in_system_port_options.len() > 0 {
            if let Some(bus) = self.display_state.audio_in_busses.get_mut(bus_index) {
                // Find the next available port.
                let mut next_port = bus
                    .ports
                    .last()
                    .map(|p| p.current_system_port_index + 1)
                    .unwrap_or(0);
                if next_port >= self.display_state.audio_in_system_port_options.len() {
                    next_port = 0;
                }

                bus.ports.push(SystemPortDisplayState {
                    current_system_port_index: next_port,
                    current_system_port_name: self.display_state.audio_in_system_port_options
                        [next_port]
                        .clone(),
                    can_remove: false,
                });

                // If there is more than one port, mark all of them as removeable.
                if bus.ports.len() > 1 {
                    for port in bus.ports.iter_mut() {
                        port.can_remove = true;
                    }
                }

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn add_audio_out_bus_port(&mut self, bus_index: usize) {
        if self.display_state.audio_out_system_port_options.len() > 0 {
            if let Some(bus) = self.display_state.audio_out_busses.get_mut(bus_index) {
                // Find the next available port.
                let mut next_port = bus
                    .ports
                    .last()
                    .map(|p| p.current_system_port_index + 1)
                    .unwrap_or(0);
                if next_port >= self.display_state.audio_out_system_port_options.len() {
                    next_port = 0;
                }

                bus.ports.push(SystemPortDisplayState {
                    current_system_port_index: next_port,
                    current_system_port_name: self.display_state.audio_out_system_port_options
                        [next_port]
                        .clone(),
                    can_remove: false,
                });

                // If there is more than one port, mark all of them as removeable.
                if bus.ports.len() > 1 {
                    for port in bus.ports.iter_mut() {
                        port.can_remove = true;
                    }
                }

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn select_audio_in_bus_system_port(
        &mut self,
        bus_index: usize,
        port_index: usize,
        system_port_index: usize,
    ) {
        if let Some(bus) = self.display_state.audio_in_busses.get_mut(bus_index) {
            if let Some(port) = bus.ports.get_mut(port_index) {
                if let Some(system_port) = self
                    .display_state
                    .audio_in_system_port_options
                    .get(system_port_index)
                {
                    port.current_system_port_index = system_port_index;
                    port.current_system_port_name = system_port.clone();

                    if self.do_build_config {
                        self.build_config();
                    }
                }
            }
        }
    }

    pub fn select_audio_out_bus_system_port(
        &mut self,
        bus_index: usize,
        port_index: usize,
        system_port_index: usize,
    ) {
        if let Some(bus) = self.display_state.audio_out_busses.get_mut(bus_index) {
            if let Some(port) = bus.ports.get_mut(port_index) {
                if let Some(system_port) = self
                    .display_state
                    .audio_out_system_port_options
                    .get(system_port_index)
                {
                    port.current_system_port_index = system_port_index;
                    port.current_system_port_name = system_port.clone();

                    if self.do_build_config {
                        self.build_config();
                    }
                }
            }
        }
    }

    pub fn select_midi_server(&mut self, index: usize) {
        let index = index.min(self.display_state.midi_server_options.len() - 1);
        if self.display_state.current_midi_server_index != index {
            self.display_state.current_midi_server_index = index;
            self.display_state.current_midi_server_name = self.display_state.midi_server_options
                [self.display_state.current_midi_server_index]
                .clone();

            self.display_state.midi_in_system_port_options = self.devices_info.midi_servers_info()
                [self.display_state.current_midi_server_index]
                .in_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();
            self.display_state.midi_out_system_port_options = self.devices_info.midi_servers_info()
                [self.display_state.current_midi_server_index]
                .out_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            self.display_state.current_midi_server_available =
                self.devices_info.midi_servers_info()[self.display_state.current_midi_server_index]
                    .available;

            self.set_defaults_for_current_midi_server();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn remove_midi_in_controller(&mut self, index: usize) {
        if self.display_state.midi_in_controllers.get(index).is_some() {
            self.display_state.midi_in_controllers.remove(index);

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn remove_midi_out_controller(&mut self, index: usize) {
        if self.display_state.midi_out_controllers.get(index).is_some() {
            self.display_state.midi_out_controllers.remove(index);

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn add_midi_in_controller(&mut self) {
        if self.display_state.midi_in_system_port_options.len() > 0 {
            // Find the next available ID.
            let mut id = String::from("Midi In");
            let mut i = 1;
            loop {
                let mut is_unique = true;
                for controller in self.display_state.midi_in_controllers.iter() {
                    if &id == &controller.id {
                        is_unique = false;
                        break;
                    }
                }
                if is_unique {
                    break;
                } else {
                    i += 1;
                    id = format!("Midi In #{}", i);
                }
            }

            // Find the next available port.
            let mut next_port = self
                .display_state
                .midi_in_controllers
                .last()
                .map(|c| c.system_port.current_system_port_index + 1)
                .unwrap_or(0);
            if next_port >= self.display_state.midi_in_system_port_options.len() {
                next_port = 0;
            }

            self.display_state
                .midi_in_controllers
                .push(MidiControllerDisplayState {
                    id,
                    system_port: SystemPortDisplayState {
                        current_system_port_index: next_port,
                        current_system_port_name: self.display_state.midi_in_system_port_options
                            [next_port]
                            .clone(),
                        can_remove: false,
                    },
                });

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn add_midi_out_controller(&mut self) {
        if self.display_state.midi_out_system_port_options.len() > 0 {
            // Find the next available ID.
            let mut id = String::from("Midi Out");
            let mut i = 1;
            loop {
                let mut is_unique = true;
                for controller in self.display_state.midi_out_controllers.iter() {
                    if &id == &controller.id {
                        is_unique = false;
                        break;
                    }
                }
                if is_unique {
                    break;
                } else {
                    i += 1;
                    id = format!("Midi Out #{}", i);
                }
            }

            // Find the next available port.
            let mut next_port = self
                .display_state
                .midi_out_controllers
                .last()
                .map(|c| c.system_port.current_system_port_index + 1)
                .unwrap_or(0);
            if next_port >= self.display_state.midi_out_system_port_options.len() {
                next_port = 0;
            }

            self.display_state
                .midi_out_controllers
                .push(MidiControllerDisplayState {
                    id,
                    system_port: SystemPortDisplayState {
                        current_system_port_index: next_port,
                        current_system_port_name: self.display_state.midi_out_system_port_options
                            [next_port]
                            .clone(),
                        can_remove: false,
                    },
                });

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn rename_midi_in_controller<S: Into<String>>(&mut self, controller_index: usize, name: S) {
        if let Some(controller) = self
            .display_state
            .midi_in_controllers
            .get_mut(controller_index)
        {
            controller.id = name.into();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn rename_midi_out_controller<S: Into<String>>(
        &mut self,
        controller_index: usize,
        name: S,
    ) {
        if let Some(controller) = self
            .display_state
            .midi_out_controllers
            .get_mut(controller_index)
        {
            controller.id = name.into();

            if self.do_build_config {
                self.build_config();
            }
        }
    }

    pub fn select_midi_in_controller_system_port(
        &mut self,
        controller_index: usize,
        system_port_index: usize,
    ) {
        if let Some(controller) = self
            .display_state
            .midi_in_controllers
            .get_mut(controller_index)
        {
            if let Some(system_port) = self
                .display_state
                .midi_in_system_port_options
                .get(system_port_index)
            {
                controller.system_port.current_system_port_index = system_port_index;
                controller.system_port.current_system_port_name = system_port.clone();

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn select_midi_out_controller_system_port(
        &mut self,
        controller_index: usize,
        system_port_index: usize,
    ) {
        if let Some(controller) = self
            .display_state
            .midi_out_controllers
            .get_mut(controller_index)
        {
            if let Some(system_port) = self
                .display_state
                .midi_out_system_port_options
                .get(system_port_index)
            {
                controller.system_port.current_system_port_index = system_port_index;
                controller.system_port.current_system_port_name = system_port.clone();

                if self.do_build_config {
                    self.build_config();
                }
            }
        }
    }

    pub fn refresh_servers(&mut self) {
        let prev_state = self.display_state.clone();
        let prev_sample_rate = *self
            .display_state
            .sample_rate_options
            .get(self.display_state.current_sample_rate_index)
            .unwrap_or(&0);

        self.devices_info.refresh_audio_servers();
        self.devices_info.refresh_midi_servers();

        // Don't rebuild the config multiple times.
        self.do_build_config = false;

        // Revert to blank slate.

        self.display_state = DisplayState::default();

        let mut default_audio_server = 0;
        for (i, server) in self.devices_info.audio_servers_info().iter().enumerate() {
            if &server.name == self.devices_info.default_audio_server() {
                default_audio_server = i;
                break;
            }
        }
        let mut default_midi_server = 0;
        for (i, server) in self.devices_info.midi_servers_info().iter().enumerate() {
            if &server.name == self.devices_info.default_midi_server() {
                default_midi_server = i;
                break;
            }
        }
        self.default_audio_server = default_audio_server;
        self.default_midi_server = default_midi_server;

        // Server options are static.
        self.display_state.audio_server_options = prev_state.audio_server_options.clone();
        self.display_state.midi_server_options = prev_state.midi_server_options.clone();

        self.set_audio_defaults();
        self.set_midi_defaults();

        // Use servers from previous state.
        self.select_audio_server(prev_state.current_audio_server_index);
        self.select_midi_server(prev_state.current_midi_server_index);

        // If previous audio device still exists, use it.
        for (device_i, device) in self.display_state.audio_device_options.iter().enumerate() {
            if device == &prev_state.current_audio_device_name {
                self.select_audio_device(device_i);

                // If device is valid, attempt to restore its previous settings.
                if !self.display_state.audio_out_system_port_options.is_empty() {
                    // If previous sample rate still exists, use it.
                    for (sample_rate_i, sample_rate) in
                        self.display_state.sample_rate_options.iter().enumerate()
                    {
                        if *sample_rate == prev_sample_rate {
                            self.select_sample_rate(sample_rate_i);
                            break;
                        }
                    }

                    // If previous buffer size is still valid, use it.
                    if prev_state.current_buffer_size >= self.display_state.buffer_size_range.min
                        && prev_state.current_buffer_size
                            <= self.display_state.buffer_size_range.max
                    {
                        self.select_buffer_size(prev_state.current_buffer_size);
                    }

                    // If an input bus was created by default, remove it.
                    if self.display_state.audio_in_busses.len() == 1 {
                        self.remove_audio_in_bus(0);
                    }

                    let num_default_out_busses = self.display_state.audio_out_busses.len();

                    // Attempt to restore previous input busses.
                    for prev_bus in prev_state.audio_in_busses.iter() {
                        let mut new_ports = Vec::<SystemPortDisplayState>::new();
                        for port in prev_bus.ports.iter() {
                            for (system_port_i, system_port) in self
                                .display_state
                                .audio_in_system_port_options
                                .iter()
                                .enumerate()
                            {
                                if &port.current_system_port_name == system_port {
                                    new_ports.push(SystemPortDisplayState {
                                        current_system_port_index: system_port_i,
                                        current_system_port_name: system_port.clone(),
                                        can_remove: false,
                                    });
                                    break;
                                }
                            }
                        }

                        // If the number of new ports is 0, discard the bus.
                        if new_ports.len() > 0 {
                            // If the number of new ports is greater than 0, mark all of them as removeable.
                            if new_ports.len() > 1 {
                                for port in new_ports.iter_mut() {
                                    port.can_remove = true;
                                }
                            }

                            self.display_state
                                .audio_in_busses
                                .push(AudioBusDisplayState {
                                    id: prev_bus.id.clone(),
                                    ports: new_ports,
                                    can_remove: true,
                                });
                        }
                    }

                    // Attempt to restore previous output busses.
                    for prev_bus in prev_state.audio_out_busses.iter() {
                        let mut new_ports = Vec::<SystemPortDisplayState>::new();
                        for port in prev_bus.ports.iter() {
                            for (system_port_i, system_port) in self
                                .display_state
                                .audio_out_system_port_options
                                .iter()
                                .enumerate()
                            {
                                if &port.current_system_port_name == system_port {
                                    new_ports.push(SystemPortDisplayState {
                                        current_system_port_index: system_port_i,
                                        current_system_port_name: system_port.clone(),
                                        can_remove: false,
                                    });
                                    break;
                                }
                            }
                        }

                        // If the number of new ports is 0, discard the bus.
                        if new_ports.len() > 0 {
                            // If the number of new ports is greater than 0, mark all of them as removeable.
                            if new_ports.len() > 1 {
                                for port in new_ports.iter_mut() {
                                    port.can_remove = true;
                                }
                            }

                            self.display_state
                                .audio_out_busses
                                .push(AudioBusDisplayState {
                                    id: prev_bus.id.clone(),
                                    ports: new_ports,
                                    can_remove: false,
                                });
                        }
                    }

                    // If any new output busses were created, remove the one that was created by default.
                    if self.display_state.audio_out_busses.len() > num_default_out_busses
                        && num_default_out_busses == 1
                    {
                        self.display_state.audio_out_busses.remove(0);
                    }

                    // If there is more than one output bus, mark all of them as removeable.
                    if self.display_state.audio_out_busses.len() > 1 {
                        for bus in self.display_state.audio_out_busses.iter_mut() {
                            bus.can_remove = true;
                        }
                    }
                }

                break;
            }
        }

        // If an input bus was created by default, remove it.
        if self.display_state.midi_in_controllers.len() == 1 {
            self.remove_midi_in_controller(0);
        }

        // Attempt to restore previous midi input controllers.
        for controller in prev_state.midi_in_controllers.iter() {
            let mut new_port = None;
            for (system_port_i, system_port) in self
                .display_state
                .midi_in_system_port_options
                .iter()
                .enumerate()
            {
                if &controller.system_port.current_system_port_name == system_port {
                    new_port = Some(SystemPortDisplayState {
                        current_system_port_index: system_port_i,
                        current_system_port_name: system_port.clone(),
                        can_remove: false,
                    });
                    break;
                }
            }

            // If the port no longer exists, discard the controller.
            if let Some(new_port) = new_port {
                self.display_state
                    .midi_in_controllers
                    .push(MidiControllerDisplayState {
                        id: controller.id.clone(),
                        system_port: new_port,
                    });
            }
        }

        // Attempt to restore previous midi output controllers.
        for controller in prev_state.midi_out_controllers.iter() {
            let mut new_port = None;
            for (system_port_i, system_port) in self
                .display_state
                .midi_out_system_port_options
                .iter()
                .enumerate()
            {
                if &controller.system_port.current_system_port_name == system_port {
                    new_port = Some(SystemPortDisplayState {
                        current_system_port_index: system_port_i,
                        current_system_port_name: system_port.clone(),
                        can_remove: false,
                    });
                    break;
                }
            }

            // If the port no longer exists, discard the controller.
            if let Some(new_port) = new_port {
                self.display_state
                    .midi_out_controllers
                    .push(MidiControllerDisplayState {
                        id: controller.id.clone(),
                        system_port: new_port,
                    });
            }
        }

        self.build_config();
        self.do_build_config = true;
    }

    pub fn set_audio_defaults(&mut self) {
        self.display_state.current_audio_server_index = self.default_audio_server;
        self.display_state.current_audio_server_name = self.display_state.audio_server_options
            [self.display_state.current_audio_server_index]
            .clone();

        self.display_state.audio_device_options = self.devices_info.audio_servers_info()
            [self.display_state.current_audio_server_index]
            .devices
            .iter()
            .map(|d| d.name.clone())
            .collect();

        self.display_state.current_audio_server_available = self.devices_info.audio_servers_info()
            [self.display_state.current_audio_server_index]
            .available;

        self.set_defaults_for_current_audio_server();

        if self.do_build_config {
            self.build_config();
        }
    }

    pub fn set_midi_defaults(&mut self) {
        self.display_state.current_midi_server_index = self.default_midi_server;
        self.display_state.current_midi_server_name = self.display_state.midi_server_options
            [self.display_state.current_midi_server_index]
            .clone();

        self.display_state.midi_in_system_port_options = self.devices_info.midi_servers_info()
            [self.display_state.current_midi_server_index]
            .in_devices
            .iter()
            .map(|d| d.name.clone())
            .collect();
        self.display_state.midi_out_system_port_options = self.devices_info.midi_servers_info()
            [self.display_state.current_midi_server_index]
            .out_devices
            .iter()
            .map(|d| d.name.clone())
            .collect();

        self.display_state.current_midi_server_available = self.devices_info.midi_servers_info()
            [self.display_state.current_midi_server_index]
            .available;

        self.set_defaults_for_current_midi_server();

        if self.do_build_config {
            self.build_config();
        }
    }

    pub fn set_defaults_for_current_audio_server(&mut self) {
        self.display_state.current_audio_device_index = self.devices_info.audio_servers_info()
            [self.display_state.current_audio_server_index]
            .default_device;

        self.display_state.current_audio_device_name = self
            .display_state
            .audio_device_options
            .get(self.display_state.current_audio_device_index)
            .unwrap_or(&String::from("Unavailable"))
            .clone();

        self.set_defaults_for_current_audio_device();

        if self.do_build_config {
            self.build_config();
        }
    }

    pub fn set_defaults_for_current_audio_device(&mut self) {
        self.display_state.audio_in_busses.clear();
        self.display_state.audio_out_busses.clear();

        if let Some(device) = self.devices_info.audio_servers_info()
            [self.display_state.current_audio_server_index]
            .devices
            .get(self.display_state.current_audio_device_index)
        {
            self.display_state.audio_in_system_port_options = device.in_ports.clone();
            self.display_state.audio_out_system_port_options = device.out_ports.clone();

            self.display_state.sample_rate_options = device.sample_rates.clone();
            self.display_state.buffer_size_range = device.buffer_size_range;

            self.display_state.current_sample_rate_index = device
                .default_sample_rate_index
                .min(self.display_state.sample_rate_options.len() - 1);

            self.display_state.current_sample_rate_str = format!(
                "{}",
                self.display_state.sample_rate_options
                    [self.display_state.current_sample_rate_index]
            );

            self.display_state.current_buffer_size = device
                .default_buffer_size
                .min(device.buffer_size_range.min)
                .max(device.buffer_size_range.max);

            self.display_state.playback_only = device.in_ports.is_empty();

            self.display_state.current_buffer_size_str =
                format!("{}", self.display_state.current_buffer_size);

            if let Some(port) = device.in_ports.get(device.default_in_port) {
                self.display_state
                    .audio_in_busses
                    .push(AudioBusDisplayState {
                        id: String::from("Mic In"),
                        ports: vec![SystemPortDisplayState {
                            current_system_port_index: device.default_in_port,
                            current_system_port_name: port.clone(),
                            can_remove: false,
                        }],
                        can_remove: true,
                    });
            }

            if let Some(left_port) = device.out_ports.get(device.default_out_port_left) {
                if let Some(right_port) = device.out_ports.get(device.default_out_port_right) {
                    self.display_state
                        .audio_out_busses
                        .push(AudioBusDisplayState {
                            id: String::from("Speakers Out"),
                            ports: vec![
                                SystemPortDisplayState {
                                    current_system_port_index: device.default_out_port_left,
                                    current_system_port_name: left_port.clone(),
                                    can_remove: true,
                                },
                                SystemPortDisplayState {
                                    current_system_port_index: device.default_out_port_right,
                                    current_system_port_name: right_port.clone(),
                                    can_remove: true,
                                },
                            ],
                            can_remove: false,
                        });
                }
            }
        } else {
            self.display_state.audio_in_system_port_options.clear();
            self.display_state.audio_out_system_port_options.clear();

            self.display_state.sample_rate_options.clear();
            self.display_state.current_sample_rate_str = String::from("Unavailable");

            self.display_state.buffer_size_range = BufferSizeRange::default();
            self.display_state.current_buffer_size_str = String::from("Unavailable");

            self.display_state.playback_only = false;
        }

        if self.do_build_config {
            self.build_config();
        }
    }

    pub fn set_defaults_for_current_midi_server(&mut self) {
        self.display_state.midi_in_controllers.clear();
        self.display_state.midi_out_controllers.clear();

        if let Some(midi_in_port) = self.display_state.midi_in_system_port_options.get(
            self.devices_info.midi_servers_info()[self.display_state.current_midi_server_index]
                .default_in_port,
        ) {
            self.display_state
                .midi_in_controllers
                .push(MidiControllerDisplayState {
                    id: String::from("Midi In"),
                    system_port: SystemPortDisplayState {
                        current_system_port_index: self.devices_info.midi_servers_info()
                            [self.display_state.current_midi_server_index]
                            .default_in_port,
                        current_system_port_name: midi_in_port.clone(),
                        can_remove: false,
                    },
                });
        }

        if self.do_build_config {
            self.build_config();
        }
    }

    pub fn display_state(&self) -> &DisplayState {
        &self.display_state
    }

    pub fn config_status(&self) -> &ConfigStatus {
        &self.config_status
    }

    fn build_config(&mut self) {
        // Invalid if there are no audio out ports.
        if self.display_state.audio_out_system_port_options.is_empty() {
            self.config_status = if self.display_state.current_audio_server_available {
                ConfigStatus::NoAudioDeviceAvailable
            } else {
                ConfigStatus::AudioServerUnavailable(
                    self.display_state.current_audio_server_name.clone(),
                )
            };
            return;
        }

        // Sanity checks

        // Invalid if there are no audio out busses. This shouldn't happen.
        if self.display_state.audio_out_busses.is_empty() {
            log::error!("Config has no audio out busses. This shouldn't happen.");
            self.config_status = ConfigStatus::UnknownError;
            return;
        }
        // Invalid if there are no audio in ports and the config wants an audio in bus. This shouldn't happen.
        if self.display_state.audio_in_system_port_options.is_empty()
            && !self.display_state.audio_in_busses.is_empty()
        {
            log::error!("Config wants to create audio in bus, but device is playback only. This shouldn't happen.");
            self.config_status = ConfigStatus::UnknownError;
            return;
        }
        // Invalid if there are no midi in ports and the config wants a midi in controller. This shouldn't happen.
        if self.display_state.midi_in_system_port_options.is_empty()
            && !self.display_state.midi_in_controllers.is_empty()
        {
            log::error!("Config wants to create midi in controller, but no system midi in ports were found. This shouldn't happen.");
            self.config_status = ConfigStatus::UnknownError;
            return;
        }
        // Invalid if there are no midi out ports and the config wants a midi out controller. This shouldn't happen.
        if self.display_state.midi_out_system_port_options.is_empty()
            && !self.display_state.midi_out_controllers.is_empty()
        {
            log::error!("Config wants to create midi out controller, but no system midi out ports were found. This shouldn't happen.");
            self.config_status = ConfigStatus::UnknownError;
            return;
        }

        let audio_in_busses = self
            .display_state
            .audio_in_busses
            .iter()
            .map(|b| AudioBusConfig {
                id: b.id.clone(),
                system_ports: b
                    .ports
                    .iter()
                    .map(|p| p.current_system_port_name.clone())
                    .collect(),
            })
            .collect();

        let audio_out_busses = self
            .display_state
            .audio_out_busses
            .iter()
            .map(|b| AudioBusConfig {
                id: b.id.clone(),
                system_ports: b
                    .ports
                    .iter()
                    .map(|p| p.current_system_port_name.clone())
                    .collect(),
            })
            .collect();

        let (midi_server, midi_in_controllers, midi_out_controllers) =
            if !self.display_state.midi_in_controllers.is_empty()
                || !self.display_state.midi_out_controllers.is_empty()
            {
                (
                    Some(self.display_state.current_midi_server_name.clone()),
                    self.display_state
                        .midi_in_controllers
                        .iter()
                        .map(|c| MidiControllerConfig {
                            id: c.id.clone(),
                            system_port: c.system_port.current_system_port_name.clone(),
                        })
                        .collect(),
                    self.display_state
                        .midi_out_controllers
                        .iter()
                        .map(|c| MidiControllerConfig {
                            id: c.id.clone(),
                            system_port: c.system_port.current_system_port_name.clone(),
                        })
                        .collect(),
                )
            } else {
                (None, Vec::new(), Vec::new())
            };

        let config = Config {
            audio_server: self.display_state.current_audio_server_name.clone(),
            system_audio_device: self.display_state.current_audio_device_name.clone(),

            audio_in_busses,
            audio_out_busses,

            sample_rate: self
                .display_state
                .sample_rate_options
                .get(self.display_state.current_sample_rate_index)
                .map(|s| *s),
            buffer_size: Some(self.display_state.current_buffer_size),

            midi_server,
            midi_in_controllers,
            midi_out_controllers,
        };
        let sample_rate = self.devices_info.sample_rate(&config).unwrap_or(1);
        let latency_frames = self.devices_info.estimated_latency(&config).unwrap_or(0);
        let latency_ms = 1_000.0 * f64::from(latency_frames) / f64::from(sample_rate);

        self.config_status = ConfigStatus::Ok {
            config,
            sample_rate,
            latency_frames,
            latency_ms,
        };
    }
}

#[derive(Debug, Clone)]
pub enum ConfigStatus {
    Ok {
        config: Config,
        sample_rate: u32,
        latency_frames: u32,
        latency_ms: f64,
    },
    AudioServerUnavailable(String),
    NoAudioDeviceAvailable,
    UnknownError,
}

impl Default for ConfigStatus {
    fn default() -> Self {
        ConfigStatus::UnknownError
    }
}

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub sample_rates: Vec<u32>,
    pub buffer_size_range: BufferSizeRange,

    pub default_in_port: usize,
    pub default_out_port_left: usize,
    pub default_out_port_right: usize,
    pub default_sample_rate_index: usize,
    pub default_buffer_size: u32,
}

#[derive(Debug, Clone)]
pub struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub devices: Vec<AudioDeviceInfo>,
    pub available: bool,

    pub default_device: usize,
}

impl AudioServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            devices: Vec::new(),
            available: false,
            default_device: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiDeviceInfo {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub in_devices: Vec<MidiDeviceInfo>,
    pub out_devices: Vec<MidiDeviceInfo>,
    pub available: bool,

    pub default_in_port: usize,
}

impl MidiServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            in_devices: Vec::new(),
            out_devices: Vec::new(),
            available: false,
            default_in_port: 0,
        }
    }
}
