use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use rodio::{
    dynamic_mixer, dynamic_mixer::DynamicMixerController, source::Zero, Source as RodioSource,
};

use steamaudio::{
    buffer::Buffer,
    context::Context,
    effect::{BinauralEffect, BinauralEffectParams, Effect, HrtfInterpolation},
    geometry::Orientation,
    simulation::{AirAbsorptionModel, DistanceAttenuationModel, Simulator},
    transform::transform,
};

#[derive(Default)]
pub struct SteamAudioPlugin;

impl Plugin for SteamAudioPlugin {
    fn build(&self, app: &mut App) {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        std::mem::forget(stream);
        let (mixer_controller, mixer) = dynamic_mixer::mixer(2, 48000);
        mixer_controller.add(Zero::new(2, 48000));
        stream_handle.play_raw(mixer).unwrap();

        let context = Context::new().unwrap();
        let simulator = context.create_simulator(48000, 64).unwrap();
        let binaural_effect = context
            .create_binaural_effect(&context.create_hrtf(48000, 64).unwrap(), 48000, 64)
            .unwrap();

        app.insert_resource(Audio {
            mixer_controller,
            context,
            simulator,
            binaural_effect,
        });

        app.add_systems(PreUpdate, create_source)
            .add_systems(PostUpdate, update_listener_and_source);
    }
}

#[derive(Resource)]
pub struct Audio {
    pub mixer_controller: Arc<DynamicMixerController<f32>>,

    pub context: Context,
    pub simulator: Simulator,

    pub binaural_effect: BinauralEffect,
}

#[derive(Component)]
pub struct Listener;

#[derive(Component)]
pub struct Source {
    pub source: steamaudio::simulation::Source,
    direction: Arc<Mutex<Vec3>>,
}

pub fn create_source(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_sources: Res<Assets<AudioSource>>,
    for_sources: Query<(Entity, &Handle<AudioSource>), Without<Source>>,
) {
    for (entity, audio_source) in for_sources.iter() {
        if let Some(audio_source) = audio_sources.get(audio_source) {
            let mut source = audio.simulator.create_source().unwrap();
            source.set_active(true);
            source.set_distance_attenuation(DistanceAttenuationModel::default());
            source.set_air_absorption(AirAbsorptionModel::default());
            audio.simulator.commit();

            let direction: Arc<Mutex<Vec3>> = Default::default();
            commands.entity(entity).insert(Source {
                source: source.clone(),
                direction: direction.clone(),
            });

            let audio_source = audio_source.decoder();
            let direct_effect = audio
                .context
                .create_direct_effect(audio_source.sample_rate(), 64, audio_source.channels())
                .unwrap();
            let binaural_effect = audio.binaural_effect.clone();
            let mut tmp = Buffer::from(vec![
                vec![0.0; 64 as usize];
                audio_source.channels() as usize
            ]);
            audio.mixer_controller.add(transform(
                audio_source.convert_samples(),
                move |in_, mut out| {
                    direct_effect.apply(&source, in_, &mut tmp);
                    binaural_effect.apply(
                        BinauralEffectParams {
                            direction: *direction.lock().unwrap(),
                            interpolation: HrtfInterpolation::Nearest,
                            spatial_blend: 1.0,
                        },
                        &tmp,
                        &mut out,
                    );
                },
                2,
                64,
            ));
        }
    }
}

pub fn update_listener_and_source(
    mut audio: ResMut<Audio>,
    for_listener: Query<Ref<Transform>, With<Listener>>,
    mut for_sources: Query<(Ref<Transform>, &mut Source)>,
) {
    let mut update = false;

    let listener_transform = for_listener.single();
    let listener_orientation = Orientation {
        translation: listener_transform.translation,
        rotation: listener_transform.rotation,
    };
    if listener_transform.is_changed() {
        audio.simulator.set_listener(listener_orientation);
        update = true;
    }

    for (transform, mut source) in for_sources.iter_mut() {
        if transform.is_changed() {
            source.source.set_source(Orientation {
                translation: transform.translation,
                rotation: transform.rotation,
            });
            let direction = audio
                .context
                .calculate_relative_direction(transform.translation, listener_orientation);
            *source.direction.lock().unwrap() = direction;
            update = true;
        } else if listener_transform.is_changed() {
            *source.direction.lock().unwrap() = audio
                .context
                .calculate_relative_direction(transform.translation, listener_orientation);
        }
    }

    if update {
        audio.simulator.run_direct();
    }
}
