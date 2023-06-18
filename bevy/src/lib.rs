use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{mesh::MeshVertexAttribute, render_resource::VertexFormat},
};
use rodio::{
    dynamic_mixer,
    dynamic_mixer::DynamicMixerController,
    source::{UniformSourceIterator, Zero},
    Source as RodioSource,
};

use steamaudio::{
    buffer::{Buffer, SpeakerLayout},
    context::Context,
    effect::{AmbisonicsDecodeEffectParams, AmbisonicsEncodeEffectParams, Effect},
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

        let context = Context::new().unwrap();

        let sampling_rate = 44100;
        let frame_size = 1024;
        let (mixer_controller, mixer) = dynamic_mixer::mixer(9, sampling_rate);
        mixer_controller.add(Zero::new(2, sampling_rate));
        let ambisonics_decode_effect = context
            .create_ambisonics_decode_effect(
                sampling_rate,
                frame_size,
                SpeakerLayout::Stereo,
                &context.create_hrtf(sampling_rate, frame_size).unwrap(),
                2,
            )
            .unwrap();
        let listener_rotation: Arc<Mutex<Quat>> = Default::default();
        let listener_rotation_0 = listener_rotation.clone();
        stream_handle
            .play_raw(transform(
                mixer,
                move |in_, out| {
                    ambisonics_decode_effect.apply(
                        AmbisonicsDecodeEffectParams {
                            orientation: Orientation {
                                translation: Default::default(),
                                rotation: *listener_rotation_0.lock().unwrap(),
                            },
                            order: 2,
                            binaural: true,
                        },
                        in_,
                        out,
                    )
                },
                2,
                frame_size,
            ))
            .unwrap();

        let simulator = context.create_simulator(sampling_rate, frame_size).unwrap();

        app.insert_resource(Audio {
            mixer_controller,
            listener_rotation,
            context,
            simulator,
            frame_size,
        });

        app.add_systems(PreUpdate, create_source)
            .add_systems(PostUpdate, update_listener_and_source);
    }
}

#[derive(Resource)]
pub struct Audio {
    mixer_controller: Arc<DynamicMixerController<f32>>,
    listener_rotation: Arc<Mutex<Quat>>,

    context: Context,
    simulator: Simulator,
    frame_size: u32,
}

#[derive(Component)]
pub struct Listener;

#[derive(Component)]
pub struct Source {
    pub source: steamaudio::simulation::Source,
    direction: Arc<Mutex<Vec3>>,
}

#[derive(Component)]
pub struct SoundMaterials {
    pub materials: Vec<steamaudio::scene::Material>,
}

pub const MESH_ATTRIBUTE_SOUND_MATERIAL: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Sound_Material", 7, VertexFormat::Uint32);

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

            let audio_source = UniformSourceIterator::new(audio_source.decoder(), 1, 44100);
            let direct_effect = audio
                .context
                .create_direct_effect(
                    audio_source.sample_rate(),
                    audio.frame_size,
                    audio_source.channels(),
                )
                .unwrap();
            let mut direct_buffer = Buffer::from(vec![
                vec![0.0; audio.frame_size as usize];
                audio_source.channels() as usize
            ]);
            let ambisonics_encode_effect = audio
                .context
                .create_ambisonics_encode_effect(audio_source.sample_rate(), audio.frame_size, 2)
                .unwrap();
            audio.mixer_controller.add(transform(
                audio_source,
                move |in_, out| {
                    direct_effect.apply(&source, in_, &mut direct_buffer);
                    ambisonics_encode_effect.apply(
                        AmbisonicsEncodeEffectParams {
                            direction: *direction.lock().unwrap(),
                            order: 2,
                        },
                        &direct_buffer,
                        out,
                    );
                },
                9,
                audio.frame_size,
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
    if listener_transform.is_changed() {
        audio.simulator.set_listener(Orientation {
            translation: listener_transform.translation,
            rotation: listener_transform.rotation,
        });
        *audio.listener_rotation.lock().unwrap() = listener_transform.rotation;
        update = true;
    }

    for (transform, mut source) in for_sources.iter_mut() {
        if transform.is_changed() {
            source.source.set_source(Orientation {
                translation: transform.translation,
                rotation: transform.rotation,
            });
            *source.direction.lock().unwrap() =
                (transform.translation - listener_transform.translation).normalize();
            update = true;
        } else if listener_transform.is_changed() {
            *source.direction.lock().unwrap() =
                (transform.translation - listener_transform.translation).normalize();
        }
    }

    if update {
        audio.simulator.run_direct();
    }
}
