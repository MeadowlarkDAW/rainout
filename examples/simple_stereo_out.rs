use rainout::{ProcessHandler, ProcessInfo, RainoutConfig, RunOptions, StreamInfo};

pub fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let my_processor = MyAudioProcessor { phase: 0.0, pitch: 440.0, gain: 0.5, step: 0.0 };

    // Default config and options asks for a stereo output.
    let config = RainoutConfig::default();
    let options = RunOptions::default();

    let stream_handle = rainout::run(&config, &options, my_processor).unwrap();

    // Wait some time before closing.
    std::thread::sleep(std::time::Duration::from_secs(10));

    // The stream is automatically closed when `stream_handle` is dropped.
    let _ = stream_handle;
}

pub struct MyAudioProcessor {
    phase: f32,
    pitch: f32,
    gain: f32,
    step: f32,
}

impl ProcessHandler for MyAudioProcessor {
    /// Initialize/allocate any buffers here. This will only be called once on
    /// creation.
    fn init(&mut self, stream_info: &StreamInfo) {
        dbg!(stream_info);

        self.step = std::f32::consts::PI * 2.0 * self.pitch / stream_info.sample_rate as f32;
    }

    /// This gets called if the user made a change to the configuration that does not
    /// require restarting the audio thread.
    fn stream_changed(&mut self, stream_info: &StreamInfo) {
        println!("stream info changed");
        dbg!(stream_info);
    }

    /// Process the current buffers. This will always be called on a realtime thread.
    fn process<'a>(&mut self, proc_info: ProcessInfo<'a>) {
        // Only processing on stereo outputs.
        if proc_info.audio_outputs.len() < 2 {
            return;
        }

        // Hints to the compiler to elid bounds checking. You may also use unsafe
        // here since it is gauranteed that the size of each buffer is the size
        // `proc_info.frames`.
        let frames = proc_info
            .frames
            .min(proc_info.audio_outputs[0].len())
            .min(proc_info.audio_outputs[1].len());

        for i in 0..frames {
            // generate rudamentary sine wave
            let smp = self.phase.sin() * self.gain;
            self.phase += self.step;
            if self.phase >= std::f32::consts::PI * 2.0 {
                self.phase -= std::f32::consts::PI * 2.0
            }

            proc_info.audio_outputs[0][i] = smp;
            proc_info.audio_outputs[1][i] = smp;
        }
    }
}
