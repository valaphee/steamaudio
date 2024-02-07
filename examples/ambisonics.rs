use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use glam::{Quat, Vec3};
use rodio::{
    dynamic_mixer,
    source::{UniformSourceIterator, Zero},
    Decoder,
};

use steamaudio::{
    ambisonics_channels,
    buffer::{Buffer, SpeakerLayout},
    context::Context,
    effect::{AmbisonicsDecodeEffectParams, AmbisonicsEncodeEffectParams, Effect},
    geometry::Orientation,
    simulation::{AirAbsorptionModel, Directivity, DistanceAttenuationModel},
    transform::transform,
};

fn main() {
    let ambisonics_order = 2;
    let sampling_rate = 44100;
    let frame_size = 1024;
    let speaker_layout = SpeakerLayout::Stereo;
    let binaural = true;

    // Create context
    let context = Context::new().unwrap();

    // Create ambisonics mixer for the final mix
    let (ambisonics_mixer_controller, ambisonics_mixer) =
        dynamic_mixer::mixer(ambisonics_channels(ambisonics_order), sampling_rate);
    ambisonics_mixer_controller.add(Zero::new(
        ambisonics_channels(ambisonics_order),
        sampling_rate,
    ));

    // Simulator is used to render sources
    let simulator = context.create_simulator(sampling_rate, frame_size).unwrap();

    // Create source and set it to active, and commit to the simulator
    let mut simulator_source = simulator.create_source().unwrap();
    simulator_source.set_active(true);
    simulator_source.set_distance_attenuation(DistanceAttenuationModel::Default);
    simulator_source.set_air_absorption(AirAbsorptionModel::Exponential([0.0, 1.0, 4.0]));
    simulator_source.set_directivity(Directivity::Dipole {
        weight: 1.0,
        power: 2.0,
    });
    simulator.commit();

    let direction = Arc::new(Mutex::new(Vec3::ZERO));
    {
        // Source to play
        let source = UniformSourceIterator::new(
            Decoder::new(
                File::open(r"example.mp3").unwrap(),
            )
            .unwrap(),
            1,
            sampling_rate,
        );

        // Create direct effect which applies the attenuation
        let direct_effect = context
            .create_direct_effect(sampling_rate, frame_size, 1)
            .unwrap();
        let simulator_source = simulator_source.clone();
        let mut direct_buffer = Buffer::new(1, frame_size);

        // Create ambisonics effect which encodes the sound to the sound field, using
        // the given direction
        let ambisonics_encode_effect = context
            .create_ambisonics_encode_effect(sampling_rate, frame_size, ambisonics_order)
            .unwrap();
        let direction = direction.clone();

        // Transform the source
        ambisonics_mixer_controller.add(transform(
            source,
            move |in_, out| {
                direct_effect.apply(&simulator_source, in_, &mut direct_buffer);
                ambisonics_encode_effect.apply(
                    AmbisonicsEncodeEffectParams {
                        direction: *direction.lock().unwrap(),
                        order: ambisonics_order,
                    },
                    &direct_buffer,
                    out,
                );
            },
            ambisonics_channels(ambisonics_order),
            frame_size,
        ));
    }

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Decode the sound field from the mixer and play the result
    let ambisonics_decode_effect = context
        .create_ambisonics_decode_effect(
            sampling_rate,
            frame_size,
            speaker_layout.clone(),
            &context.create_hrtf(sampling_rate, frame_size).unwrap(),
            ambisonics_order,
        )
        .unwrap();
    stream_handle
        .play_raw(transform(
            ambisonics_mixer,
            move |in_, out| {
                ambisonics_decode_effect.apply(
                    AmbisonicsDecodeEffectParams {
                        orientation: Orientation {
                            translation: Default::default(),
                            rotation: Quat::default(),
                        },
                        order: ambisonics_order,
                        binaural,
                    },
                    in_,
                    out,
                )
            },
            speaker_layout.channels(),
            frame_size,
        ))
        .unwrap();

    // Rotate the source around the listener
    let mut i = 0f32;
    loop {
        {
            let mut direction = direction.lock().unwrap();
            direction.x = i.sin();
            direction.z = i.cos();

            simulator_source.set_source(Orientation {
                translation: *direction,
                rotation: Default::default(),
            });
            simulator.run_direct();
        }

        i += 0.01;
        sleep(Duration::from_millis(20))
    }
}
