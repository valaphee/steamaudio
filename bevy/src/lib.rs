use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use rodio::{dynamic_mixer::DynamicMixerController, Source as RodioSource};

use steamaudio::{
    buffer::Buffer,
    context::Context,
    effect::{AmbisonicsEncodeEffect, AmbisonicsEncodeEffectParams, Effect},
    geometry::Orientation,
    simulation::Simulator,
    transform::transform,
};

#[derive(Resource)]
pub struct Audio {
    pub context: Context,
    pub simulator: Simulator,

    pub ambisonics_mixer_controller: Arc<DynamicMixerController<f32>>,
    pub ambisonics_encode_effect: AmbisonicsEncodeEffect,
}

#[derive(Component)]
pub struct Listener;

#[derive(Component)]
pub struct Source {
    pub source: steamaudio::simulation::Source,
    direction: Arc<Mutex<Vec3>>,
}

pub fn create(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_sources: Res<Assets<AudioSource>>,
    sources: Query<(Entity, &Handle<AudioSource>), Without<Source>>,
) {
    for (entity, audio_source) in sources.iter() {
        if let Some(audio_source) = audio_sources.get(audio_source) {
            let mut source = audio.simulator.create_source().unwrap();
            source.set_active(true);
            source.set_distance_attenuation();
            source.set_air_absorption();
            source.set_directivity();
            audio.simulator.commit();

            let direction: Arc<Mutex<Vec3>> = Default::default();
            commands.entity(entity).insert(Source {
                source: source.clone(),
                direction: direction.clone(),
            });

            let audio_source = audio_source.decoder();
            let direct_effect = audio
                .context
                .create_direct_effect(
                    audio_source.sample_rate(),
                    64,
                    audio_source.channels() as u8,
                )
                .unwrap();
            let ambisonics_encode_effect = audio.ambisonics_encode_effect.clone();
            let mut tmp = Buffer::from(vec![
                vec![0.0; 64 as usize];
                audio_source.channels() as usize
            ]);
            audio.ambisonics_mixer_controller.add(transform(
                audio_source.convert_samples(),
                move |in_, mut out| {
                    direct_effect.apply(&source, in_, &mut tmp);
                    ambisonics_encode_effect.apply(
                        AmbisonicsEncodeEffectParams {
                            direction: *direction.lock().unwrap(),
                            order: 2,
                        },
                        &tmp,
                        &mut out,
                    );
                },
                9,
                64,
            ));
        }
    }
}

pub fn update(
    mut audio: ResMut<Audio>,
    listener: Query<Ref<Transform>, With<Listener>>,
    mut sources: Query<(Ref<Transform>, &mut Source)>,
) {
    let mut update = false;

    let listener_transform = listener.single();
    let listener_orientation = Orientation {
        translation: listener_transform.translation,
        rotation: listener_transform.rotation,
    };
    if listener_transform.is_changed() {
        update = true;
        audio.simulator.set_listener(listener_orientation);
    }

    for (transform, mut source) in sources.iter_mut() {
        if transform.is_changed() {
            update = true;
            source.source.set_source(Orientation {
                translation: transform.translation,
                rotation: transform.rotation,
            });
            *source.direction.lock().unwrap() = audio
                .context
                .calculate_relative_direction(transform.translation, listener_orientation);
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
