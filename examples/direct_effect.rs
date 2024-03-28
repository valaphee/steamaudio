/// Demonstrates the direct_effect occlusion and transmission.
/// Note that in IPLDirectEffectParams the transmissionType needs to be set to
/// IPLTransmissionType_IPL_TRANSMISSIONTYPE_FREQDEPENDENT in order to properly
/// hear the transmission effect.
use std::{fs::File, thread::sleep, time::Duration};

use glam::Vec3;
use rodio::{
    dynamic_mixer,
    source::{UniformSourceIterator, Zero},
    Decoder,
};

use steamaudio::{
    buffer::SpeakerLayout, context::Context, effect::Effect, geometry::Orientation,
    transform::transform,
};

fn main() {
    let sampling_rate = 44100;
    let frame_size = 1024;
    let speaker_layout = SpeakerLayout::Stereo;

    // Create context
    let context = Context::new().unwrap();

    // Create scene
    let scene = context.create_scene().unwrap();

    // Create some audio geometry
    let vertices: [[f32; 3]; 4] = [
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    let triangles: [[u32; 3]; 2] = [[0, 1, 2], [0, 2, 3]];

    let material = steamaudio::scene::Material {
        absorption: [0.1, 1.0, 1.0],
        scattering: 0.05,
        transmission: [0.60, 0.0, 0.0],
    };

    let materials = [material];
    let material_indices: [u32; 2] = [0, 0];

    // Add mesh to the scene
    let mut static_mesh = scene
        .create_static_mesh(
            triangles.as_slice(),
            vertices.as_slice(),
            material_indices.as_slice(),
            materials.as_slice(),
        )
        .unwrap();
    static_mesh.set_visible(true);
    scene.commit();

    // Simulator is used to render sources
    // NOTE: Parameters like maxNumOcclusionSamples in SimulationSettings should
    // probably be set
    let mut simulator = context.create_simulator(sampling_rate, frame_size).unwrap();
    simulator.set_scene(&scene);
    simulator.set_reflections(4096, 16, 2.0, 1, 1.0);

    // Create source and set it to active
    let mut simulator_source = simulator.create_source(true).unwrap();
    simulator_source.set_occlusion();
    simulator_source.set_transmission(1);
    simulator_source.set_reflections();
    simulator_source.set_active(true);

    // Put the source behind the mesh
    simulator_source.set_source(Orientation {
        translation: Vec3::new(0.0, 0.0, -0.3),
        rotation: Default::default(),
    });

    // Put the listener in front of the mesh
    simulator.set_listener(Orientation {
        translation: Vec3::new(0.0, 0.0, 0.3),
        rotation: Default::default(),
    });

    // Commit and run simulation
    simulator.commit();
    simulator.run_direct();

    // Create stereo mixer for the final mix
    let (stereo_mixer_controller, stereo_mixer) =
        dynamic_mixer::mixer(speaker_layout.channels(), sampling_rate);
    stereo_mixer_controller.add(Zero::new(speaker_layout.channels(), sampling_rate));

    // Source to play
    let source = UniformSourceIterator::new(
        Decoder::new(File::open(r"example.mp3").unwrap()).unwrap(),
        2,
        sampling_rate,
    );

    // Create direct effect which applies the attenuation
    let direct_effect = context
        .create_direct_effect(sampling_rate, frame_size, 2)
        .unwrap();

    stereo_mixer_controller.add(transform(
        source,
        move |in_, out| {
            direct_effect.apply(&simulator_source, in_, out);
        },
        speaker_layout.channels(),
        frame_size,
    ));

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    stream_handle.play_raw(stereo_mixer).unwrap();

    loop {
        sleep(Duration::from_millis(20))
    }
}
