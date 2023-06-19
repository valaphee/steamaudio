//! This example illustrates how to load and play an audio file, and control
//! where the sounds seems to come from.
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{audio::AudioSource, input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use bevy_steamaudio::{DopplerEffect, Listener, SteamAudioPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SteamAudioPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_positions)
        .add_systems(Update, (grab_mouse, process_input))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // sound emitter
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
        Emitter,
        asset_server.load::<AudioSource, _>("example.mp3"),
        DopplerEffect::default(),
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
        Freecam,
    ));
}

#[derive(Component)]
struct Emitter;

fn update_positions(
    time: Res<Time>,

    mut for_emitter: Query<&mut Transform, With<Emitter>>
) {
    let mut emitter_transform = for_emitter.single_mut();
    emitter_transform.translation.x = time.elapsed_seconds().sin() * 3.0;
    emitter_transform.translation.z = time.elapsed_seconds().cos() * 30.0;
}

#[derive(Component)]
pub struct Freecam;

pub fn grab_mouse(
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,

    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();
    if mouse_input.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

pub fn process_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,

    windows: Query<&Window>,
    mut transforms: Query<&mut Transform, With<Freecam>>,
) {
    let mut mouse_delta: Vec2 = Vec2::ZERO;
    let window = windows.single();
    if !window.cursor.visible {
        for event in mouse_motion_event_reader.iter() {
            mouse_delta += event.delta;
        }
    }

    let time_delta = time.raw_delta_seconds();

    for mut transform in transforms.iter_mut() {
        let mut move_x = 0.0;
        let mut move_y = 0.0;
        let mut move_z = 0.0;
        if keyboard_input.pressed(KeyCode::W) {
            move_x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            move_y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            move_y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_z -= 1.0;
        }
        if move_x != 0.0 || move_y != 0.0 || move_z != 0.0 {
            let move_vec =
                transform.rotation * Vec3::new(-move_z, 0., -move_x) + Vec3::new(0., move_y, 0.);
            transform.translation += move_vec * time_delta * 20.0;
        }

        if mouse_delta.x.abs() > 1e-5 || mouse_delta.y.abs() > 1e-5 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                (yaw + (mouse_delta.x * -0.0003)) % (PI * 2.0),
                (pitch + (mouse_delta.y * -0.0003)).clamp(-FRAC_PI_2, FRAC_PI_2),
                0.0,
            );
        }
    }
}
