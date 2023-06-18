//! This example illustrates how to load and play an audio file, and control
//! where the sounds seems to come from.
use bevy::{audio::AudioSource, prelude::*};
use rodio::dynamic_mixer;

use bevy_steamaudio::{create, update, Listener};
use steamaudio::{
    context::Context,
    effect::{AmbisonicsBinauralEffectParams, Effect},
    transform::transform,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_positions)
        .add_systems(Update, create)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    std::mem::forget(stream);

    let context = Context::new().unwrap();
    let (ambisonics_mixer_controller, ambisonics_mixer) = dynamic_mixer::mixer(9, 44100);
    let ambisonics_binaural_effect = context
        .create_ambisonics_binaural_effect(44100, 64, &context.create_hrtf(44100, 64).unwrap(), 2)
        .unwrap();
    stream_handle
        .play_raw(transform(
            ambisonics_mixer,
            move |in_, out| {
                ambisonics_binaural_effect.apply(
                    AmbisonicsBinauralEffectParams { order: 2 },
                    in_,
                    out,
                );
            },
            2,
            64,
        ))
        .unwrap();

    let simulator = context.create_simulator(44100, 64).unwrap();
    commands.insert_resource(bevy_steamaudio::Audio {
        simulator,
        ambisonics_mixer_controller,
        ambisonics_encode_effect: context
            .create_ambisonics_encode_effect(44100, 64, 2)
            .unwrap(),
        context,
    });

    // Space between the two ears
    let gap = 4.0;

    // left ear
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(-gap / 2.0, 0.0, 0.0),
        ..default()
    });

    // right ear
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_xyz(gap / 2.0, 0.0, 0.0),
        ..default()
    });

    let music: Handle<AudioSource> = asset_server.load("example.mp3");

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.2,
                ..default()
            })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        music,
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Listener,
    ));
}

fn update_positions(
    time: Res<Time>,
    mut emitter: Query<&mut Transform, With<Handle<AudioSource>>>,
) {
    let mut emitter_transform = emitter.single_mut();
    emitter_transform.translation.x = time.elapsed_seconds().sin() * 3.0;
    emitter_transform.translation.z = time.elapsed_seconds().cos() * 3.0;
}
