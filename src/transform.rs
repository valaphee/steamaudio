use std::time::Duration;

use rodio::Source;

use crate::buffer::Buffer;

pub fn transform<I, F>(
    input: I,
    function: F,
    output_channels: u16,
    frame_size: u32,
) -> Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    let mut input_frame = Buffer::from(vec![
        vec![0.0; frame_size as usize];
        input.channels() as usize
    ]);
    let mut output_frame = Buffer::from(vec![
        vec![0.0; frame_size as usize];
        output_channels as usize
    ]);

    Transform {
        function,

        frame_size,
        sample: 0,

        input_frame,
        input_frame_channel: 0,

        output_frame,
        output_frame_channel: 0,

        input,
    }
}

pub struct Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    input: I,
    function: F,

    frame_size: u32,
    sample: u32,

    input_frame: Buffer,
    input_frame_channel: u16,

    output_frame: Buffer,
    output_frame_channel: u16,
}

impl<I, F> Iterator for Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input_frame_channel != self.input_frame.channel_count() {
            // Read the remaining channels for the current sample
            if self.output_frame_channel == self.output_frame.channel_count() {
                for input_frame_channel in
                    self.input_frame_channel..self.input_frame.channel_count()
                {
                    self.input_frame.data()[input_frame_channel as usize][self.sample as usize] =
                        self.input.next().unwrap_or(0.0);
                }

                // Next sample
                self.sample += 1;
                self.input_frame_channel = 0;
                self.output_frame_channel = 0;
            } else {
                self.input_frame.data()[self.input_frame_channel as usize][self.sample as usize] =
                    self.input.next().unwrap_or(0.0);
                self.input_frame_channel += 1;
            }
        } else if self.output_frame_channel == self.output_frame.channel_count() {
            // Next sample
            self.sample += 1;
            self.input_frame_channel = 0;
            self.output_frame_channel = 0;
        }

        // Process all samples in buffer
        if self.sample == self.frame_size {
            self.sample = 0;

            (self.function)(&self.input_frame, &mut self.output_frame);
        }

        let value =
            self.output_frame.data()[self.output_frame_channel as usize][self.sample as usize];
        self.output_frame_channel += 1;

        Some(value)
    }
}

impl<I, F> Source for Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.output_frame.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
