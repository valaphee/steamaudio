use std::{
    cmp, mem,
    sync::{Arc, Mutex},
    time::Duration,
};

use rodio::Source;

use crate::buffer::Buffer;

#[inline]
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
    let input_buffer = Buffer::from(vec![
        vec![0.0; frame_size as usize];
        input.channels() as usize
    ]);
    let output_buffer = Buffer::from(vec![
        vec![0.0; frame_size as usize];
        output_channels as usize
    ]);

    let total_duration = input.total_duration();

    let mut transform = Transform {
        function,
        input_buffer,
        output_buffer,
        current_frame: Arc::new(Frame::Data(FrameData {
            frame_size: 0,
            channels: 0,
            sampling_rate: 0,
            next: Mutex::new(Arc::new(Frame::Input(Mutex::new(Some(input))))),
        })),
        position_in_frame: 0,
        total_duration,
    };
    transform.next_frame();
    transform
}

pub struct Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    function: F,

    input_buffer: Buffer,
    output_buffer: Buffer,

    current_frame: Arc<Frame<I>>,
    position_in_frame: usize,

    total_duration: Option<Duration>,
}

enum Frame<I>
where
    I: Source<Item = f32>,
{
    Data(FrameData<I>),
    End,
    Input(Mutex<Option<I>>),
}

struct FrameData<I>
where
    I: Source<Item = f32>,
{
    frame_size: usize,
    channels: u16,
    sampling_rate: u32,

    next: Mutex<Arc<Frame<I>>>,
}

impl<I> Drop for FrameData<I>
where
    I: Source<Item = f32>,
{
    fn drop(&mut self) {
        // This is necessary to prevent stack overflows deallocating long chains of the
        // mutually recursive `Frame` and `FrameData` types. This iteratively
        // traverses as much of the chain as needs to be deallocated, and
        // repeatedly "pops" the head off the list. This solves the problem, as
        // when the time comes to actually deallocate the `FrameData`,
        // the `next` field will contain a `Frame::End`, or an `Arc` with additional
        // references, so the depth of recursive drops will be bounded.
        while let Ok(arc_next) = self.next.get_mut() {
            if let Some(next_ref) = Arc::get_mut(arc_next) {
                // This allows us to own the next Frame.
                let next = mem::replace(next_ref, Frame::End);
                if let Frame::Data(next_data) = next {
                    // Swap the current FrameData with the next one, allowing the current one
                    // to go out of scope.
                    *self = next_data;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

impl<I, F> Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    fn next_frame(&mut self) {
        let next_frame = {
            let mut next_frame_ptr = match &*self.current_frame {
                Frame::Data(FrameData { next, .. }) => next.lock().unwrap(),
                _ => unreachable!(),
            };

            let next_frame = match &**next_frame_ptr {
                Frame::Data(_) => next_frame_ptr.clone(),
                Frame::End => next_frame_ptr.clone(),
                Frame::Input(input) => {
                    let mut input = input.lock().unwrap().take().unwrap();

                    let frame_size = input.current_frame_len();

                    let next_frame = if frame_size == Some(0) {
                        Arc::new(Frame::End)
                    } else {
                        let channels = input.channels();
                        let sampling_rate = input.sample_rate();

                        let mut channel = 0;
                        let mut frame = 0;
                        for value in input.by_ref().take(cmp::min(
                            frame_size.unwrap_or(
                                self.input_buffer.samples() as usize
                                    * self.input_buffer.channels() as usize,
                            ),
                            self.input_buffer.samples() as usize
                                * self.input_buffer.channels() as usize,
                        )) {
                            self.input_buffer.data()[channel][frame] = value;

                            channel += 1;
                            if channel == channels as usize {
                                channel = 0;
                                frame += 1;
                            }
                        }

                        if frame == 0 {
                            Arc::new(Frame::End)
                        } else {
                            Arc::new(Frame::Data(FrameData {
                                frame_size: channels as usize * frame,
                                channels,
                                sampling_rate,
                                next: Mutex::new(Arc::new(Frame::Input(Mutex::new(Some(input))))),
                            }))
                        }
                    };
                    (self.function)(&self.input_buffer, &mut self.output_buffer);
                    next_frame
                }
            };

            *next_frame_ptr = next_frame.clone();
            next_frame
        };

        self.current_frame = next_frame;
        self.position_in_frame = 0;
    }
}

impl<I, F> Iterator for Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        let current_sample;

        match &*self.current_frame {
            Frame::Data(FrameData {
                frame_size,
                channels,
                ..
            }) => {
                current_sample = Some(
                    self.output_buffer.data[self.position_in_frame % *channels as usize]
                        [self.position_in_frame / *channels as usize],
                );
                self.position_in_frame += 1;
                if self.position_in_frame >= *frame_size {
                    self.next_frame();
                }
            }
            Frame::End => {
                current_sample = None;
            }
            Frame::Input(_) => unreachable!(),
        };

        current_sample
    }
}

impl<I, F> Source for Transform<I, F>
where
    I: Source<Item = f32>,
    F: FnMut(&Buffer, &mut Buffer),
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        match &*self.current_frame {
            Frame::Data(FrameData { frame_size, .. }) => Some(frame_size - self.position_in_frame),
            Frame::End => Some(0),
            Frame::Input(_) => unreachable!(),
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        match *self.current_frame {
            Frame::Data(FrameData { channels, .. }) => channels,
            Frame::End => 1,
            Frame::Input(_) => unreachable!(),
        }
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        match *self.current_frame {
            Frame::Data(FrameData { sampling_rate, .. }) => sampling_rate,
            Frame::End => 44100,
            Frame::Input(_) => unreachable!(),
        }
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}
