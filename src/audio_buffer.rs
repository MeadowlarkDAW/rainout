#[derive(Debug)]
pub struct AudioDeviceBuffer {
    pub(crate) channel_buffers: Vec<Vec<f32>>,
    pub(crate) frames: usize,
}

impl AudioDeviceBuffer {
    pub(crate) fn new(channels: u16, max_buffer_size: u32) -> AudioDeviceBuffer {
        let mut channel_buffers = Vec::<Vec<f32>>::new();
        for _ in 0..channels {
            channel_buffers.push(Vec::<f32>::with_capacity(max_buffer_size as usize));
        }

        AudioDeviceBuffer {
            channel_buffers,
            frames: 0,
        }
    }

    pub(crate) fn clear_and_resize(&mut self, frames: usize) {
        for channel in self.channel_buffers.iter_mut() {
            channel.clear();

            // This should never allocate because each buffer was given a capacity of
            // the maximum buffer size that the audio server will send.
            channel.resize(frames, 0.0);
        }

        self.frames = frames;
    }

    pub fn get(&self, channel: usize) -> Option<&[f32]> {
        self.channel_buffers.get(channel).map(|c| c.as_slice())
    }

    pub fn get_mut(&mut self, channel: usize) -> Option<&mut [f32]> {
        self.channel_buffers
            .get_mut(channel)
            .map(|c| c.as_mut_slice())
    }

    pub fn channels(&self) -> &[Vec<f32>] {
        self.channel_buffers.as_slice()
    }

    pub fn channels_mut(&mut self) -> &mut [Vec<f32>] {
        self.channel_buffers.as_mut_slice()
    }

    pub fn num_channels(&self) -> usize {
        self.channel_buffers.len()
    }

    pub fn frames(&self) -> usize {
        self.frames
    }
}

impl std::ops::Index<usize> for AudioDeviceBuffer {
    type Output = [f32];

    fn index(&self, index: usize) -> &Self::Output {
        self.channel_buffers[index].as_slice()
    }
}
impl std::ops::IndexMut<usize> for AudioDeviceBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_buffers[index].as_mut_slice()
    }
}
