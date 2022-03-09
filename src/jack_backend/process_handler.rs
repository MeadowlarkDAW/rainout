use crate::{ProcessHandler, ProcessInfo, StreamInfo};

#[cfg(feature = "midi")]
use crate::{error::MidiBufferPushError, MidiBuffer};

pub struct JackProcessHandler<P: ProcessHandler> {
    process_handler: P,

    audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
    audio_out_ports: Vec<jack::Port<jack::AudioOut>>,

    audio_in_buffers: Vec<Vec<f32>>,
    audio_out_buffers: Vec<Vec<f32>>,

    #[cfg(feature = "midi")]
    midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
    #[cfg(feature = "midi")]
    midi_out_ports: Vec<jack::Port<jack::MidiOut>>,

    #[cfg(feature = "midi")]
    midi_in_buffers: Vec<MidiBuffer>,
    #[cfg(feature = "midi")]
    midi_out_buffers: Vec<MidiBuffer>,

    audio_buffer_size: usize,
    check_for_silence: bool,
    silent_audio_in_flags: Vec<bool>,
}

impl<P: ProcessHandler> JackProcessHandler<P> {
    pub fn new(
        process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        #[cfg(feature = "midi")] midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
        #[cfg(feature = "midi")] midi_out_ports: Vec<jack::Port<jack::MidiOut>>,
        stream_info: &StreamInfo,
    ) -> Self {
        let audio_buffer_size = stream_info.buffer_size.max_buffer_size() as usize;

        let audio_in_buffers =
            (0..audio_in_ports.len()).map(|_| Vec::with_capacity(audio_buffer_size)).collect();
        let audio_out_buffers =
            (0..audio_out_ports.len()).map(|_| Vec::with_capacity(audio_buffer_size)).collect();

        let silent_audio_in_flags = vec![false; audio_in_ports.len()];

        #[cfg(feature = "midi")]
        let (midi_in_buffers, midi_out_buffers) = {
            if let Some(midi_info) = &stream_info.midi_info {
                let midi_buffer_size = midi_info.midi_buffer_size;

                (
                    (0..midi_in_ports.len()).map(|_| MidiBuffer::new(midi_buffer_size)).collect(),
                    (0..midi_out_ports.len()).map(|_| MidiBuffer::new(midi_buffer_size)).collect(),
                )
            } else {
                (Vec::new(), Vec::new())
            }
        };

        Self {
            process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            #[cfg(feature = "midi")]
            midi_in_ports,
            #[cfg(feature = "midi")]
            midi_out_ports,
            #[cfg(feature = "midi")]
            midi_in_buffers,
            #[cfg(feature = "midi")]
            midi_out_buffers,
            audio_buffer_size: audio_buffer_size as usize,
            check_for_silence: stream_info.checking_for_silent_inputs,
            silent_audio_in_flags,
        }
    }
}

impl<P: ProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut frames: usize = 0;

        // Copy audio inputs
        for (buffer, port) in self.audio_in_buffers.iter_mut().zip(self.audio_in_ports.iter()) {
            let port_buffer = port.as_slice(ps);

            // Sanity checks.
            if port_buffer.len() > self.audio_buffer_size {
                log::warn!(
                    "Jack sent a buffer size of {} when the max buffer size is {}",
                    port_buffer.len(),
                    self.audio_buffer_size
                );
            }
            if frames != 0 && port_buffer.len() != frames {
                log::error!(
                    "Jack sent buffers of unmatched length: {}, {}",
                    frames,
                    port_buffer.len()
                );
                frames = port_buffer.len().min(frames);
            } else {
                frames = port_buffer.len()
            }

            buffer.resize(port_buffer.len(), 0.0);
            buffer.copy_from_slice(&port_buffer);
        }

        if self.audio_in_buffers.len() == 0 {
            // Check outputs for number of frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                frames = out_port.as_mut_slice(ps).len();
            }
        }

        // Clear audio outputs.
        for buffer in self.audio_out_buffers.iter_mut() {
            buffer.clear();
            buffer.resize(frames, 0.0);
        }

        #[cfg(feature = "midi")]
        {
            // Collect MIDI inputs
            for (midi_buffer, port) in
                self.midi_in_buffers.iter_mut().zip(self.midi_in_ports.iter())
            {
                midi_buffer.clear();

                for event in port.iter(ps) {
                    if let Err(e) = midi_buffer.push_raw(event.time, event.bytes) {
                        match e {
                            MidiBufferPushError::BufferFull => {
                                log::error!("Midi event dropped because buffer is full!");
                            }
                            MidiBufferPushError::EventTooLong(_) => {
                                log::debug!(
                                    "Midi event {:?} was dropped because it is too long",
                                    event.bytes
                                );
                            }
                        }
                    }
                }
            }

            // Clear MIDI outputs
            for midi_buffer in self.midi_out_buffers.iter_mut() {
                midi_buffer.clear();
            }
        }

        if self.check_for_silence {
            // TODO: This could probably be optimized.
            for (buffer, flag) in
                self.audio_in_buffers.iter().zip(self.silent_audio_in_flags.iter_mut())
            {
                *flag = true;
                for smp in buffer.iter() {
                    if *smp != 0.0 {
                        *flag = false;
                        break;
                    }
                }
            }
        }

        self.process_handler.process(ProcessInfo {
            audio_inputs: &self.audio_in_buffers,
            audio_outputs: &mut self.audio_out_buffers,
            frames,
            silent_audio_inputs: &self.silent_audio_in_flags,
            #[cfg(feature = "midi")]
            midi_inputs: &self.midi_in_buffers,
            #[cfg(feature = "midi")]
            midi_outputs: &mut self.midi_out_buffers,
        });

        // Copy processed data to audio outputs
        for (buffer, port) in self.audio_out_buffers.iter().zip(self.audio_out_ports.iter_mut()) {
            let port_buffer = port.as_mut_slice(ps);

            // Sanity check
            let mut len = port_buffer.len();
            if port_buffer.len() != buffer.len() {
                log::error!(
                    "Jack sent buffers of unmatched length: {}, {}",
                    port_buffer.len(),
                    buffer.len()
                );
                len = port_buffer.len().min(buffer.len());
            }

            port_buffer[0..len].copy_from_slice(&buffer[0..len]);
        }

        #[cfg(feature = "midi")]
        {
            // Copy processed data to MIDI outputs
            for (midi_buffer, port) in
                self.midi_out_buffers.iter().zip(self.midi_out_ports.iter_mut())
            {
                let mut port_writer = port.writer(ps);

                for event in midi_buffer.events() {
                    if let Err(e) = port_writer
                        .write(&jack::RawMidi { time: event.delta_frames, bytes: &event.data() })
                    {
                        log::error!("Warning: Could not copy midi data to Jack output: {}", e);
                    }
                }
            }
        }

        jack::Control::Continue
    }
}
