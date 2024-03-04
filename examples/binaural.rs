use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use glam::Vec3;
use rodio::{
    dynamic_mixer,
    source::{UniformSourceIterator, Zero},
    Decoder,
};

use steamaudio::{
    buffer::{Buffer, SpeakerLayout},
    context::Context,
    effect::{BinauralEffectParams, Effect, HrtfInterpolation},
    geometry::Orientation,
    simulation::{AirAbsorptionModel, DistanceAttenuationModel},
    transform::transform,
};

fn main() {
    let sampling_rate = 44100;
    let frame_size = 1024;
    let speaker_layout = SpeakerLayout::Stereo;

    // Create context
    let context = Context::new().unwrap();

    // Create stereo mixer for the final mix
    let (stereo_mixer_controller, stereo_mixer) =
        dynamic_mixer::mixer(speaker_layout.channels(), sampling_rate);
    stereo_mixer_controller.add(Zero::new(speaker_layout.channels(), sampling_rate));

    // Simulator is used to render sources
    let simulator = context.create_simulator(sampling_rate, frame_size).unwrap();

    // Create source and set it to active, and commit to the simulator
    let mut simulator_source = simulator.create_source().unwrap();
    simulator_source.set_active(true);
    simulator_source.set_distance_attenuation(DistanceAttenuationModel::Default);
    simulator_source.set_air_absorption(AirAbsorptionModel::Exponential([0.0, 1.0, 4.0]));
    simulator.commit();

    let direction = Arc::new(Mutex::new(Vec3::ZERO));
    {
        // Source to play
        let source = UniformSourceIterator::new(
            Decoder::new(File::open(r"example.mp3").unwrap()).unwrap(),
            1,
            sampling_rate,
        );

        // Create direct effect which applies the attenuation
        let direct_effect = context
            .create_direct_effect(sampling_rate, frame_size, 1)
            .unwrap();
        let simulator_source = simulator_source.clone();

        let binaural_effect = context
            .create_binaural_effect(
                &context.create_hrtf(sampling_rate, frame_size).unwrap(),
                sampling_rate,
                frame_size,
            )
            .unwrap();

        // Transform the source
        let direction = direction.clone();
        let mut direct_buffer = Buffer::new(1, frame_size);
        stereo_mixer_controller.add(transform(
            source,
            move |in_, out| {
                direct_effect.apply(&simulator_source, in_, &mut direct_buffer);
                binaural_effect.apply(
                    BinauralEffectParams {
                        direction: *direction.lock().unwrap(),
                        interpolation: HrtfInterpolation::Nearest,
                        spatial_blend: 1.0,
                    },
                    &direct_buffer,
                    out,
                );
            },
            speaker_layout.channels(),
            frame_size,
        ));
    }

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    stream_handle.play_raw(stereo_mixer).unwrap();

    // Rotate the source around the listener
    let mut i = 0f32;
    loop {
        {
            let mut direction = direction.lock().unwrap();
            direction.x = i.sin();
            direction.z = i.cos();

            simulator_source.set_source(Orientation {
                translation: Vec3::ZERO,
                rotation: Default::default(),
            });
            simulator.run_direct();
        }

        i += 0.01;
        sleep(Duration::from_millis(20))
    }
}
