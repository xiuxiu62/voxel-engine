mod chunk;
mod mesh;
mod voxel;

use bevy::{
    app::{AppExit, Update},
    asset::{AssetServer, Assets, Handle},
    color::Color,
    core_pipeline::{
        bloom::BloomSettings,
        core_3d::{Camera3d, Camera3dBundle},
        tonemapping::Tonemapping,
    },
    ecs::{
        event::EventWriter,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{vec3, Vec3},
    pbr::{
        light_consts, DirectionalLight, DirectionalLightBundle, PbrBundle, StandardMaterial,
        VolumetricFogSettings,
    },
    prelude::{default, App, PluginGroup, Startup},
    render::{
        camera::ClearColor,
        mesh::Mesh,
        settings::{Backends, RenderCreation, WgpuSettings},
        texture::Image,
        view::GpuCulling,
        RenderPlugin,
    },
    time::Time,
    transform::components::Transform,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use chunk::Chunk;
use voxel::Voxel;

const TITLE: &str = "Voxel";

#[derive(Debug, Resource)]
struct ExampleAsset {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn main() {
    let wgpu_settings = WgpuSettings {
        backends: Some(Backends::VULKAN | Backends::METAL),
        ..Default::default()
    };
    let render_plugin = RenderPlugin {
        render_creation: RenderCreation::Automatic(wgpu_settings),
        ..Default::default()
    };
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: TITLE.to_owned(),
            ..default()
        }),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(render_plugin).set(window_plugin))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup, render_chunks.after(setup)))
        .add_systems(Update, handle_input)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    (0..3)
        .flat_map(|x| (0..3).map(move |z| Chunk::new(vec3(x as f32, 0.0, z as f32))))
        .for_each(|mut chunk| {
            for x in 0..Chunk::SIZE {
                for y in 0..Chunk::SIZE {
                    for z in 0..Chunk::SIZE {
                        chunk.set(x, y, z, Voxel { id: 1 });
                    }
                }
            }

            commands.spawn(chunk);
        });

    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_translation(vec3(0.0, 0.0, -10.0))
                    .looking_at(vec3(10.0, 0.0, 10.0), Vec3::Y),
                ..Default::default()
            },
            GpuCulling,
        ))
        .insert(Tonemapping::TonyMcMapface)
        .insert(BloomSettings::default())
        .insert(VolumetricFogSettings {
            ambient_intensity: 0.0,
            ..Default::default()
        });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let texture: Handle<Image> = asset_server.load("array_texture.png");
    let mesh = meshes.add(mesh::generate_cube());
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        ..Default::default()
    });

    commands.insert_resource(ExampleAsset { mesh, material });
}

fn render_chunks(
    mut commands: Commands,
    example_asset: Res<ExampleAsset>,
    chunk_query: Query<&Chunk>,
) {
    chunk_query.iter().for_each(|chunk| {
        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    if chunk.get(x, y, z).is_none() {
                        return;
                    }

                    let transform = Transform::from_xyz(
                        (Voxel::SIZE * x as f32)
                            + (Voxel::SIZE * Chunk::SIZE as f32 * chunk.position.x),
                        Voxel::SIZE * y as f32,
                        (Voxel::SIZE * z as f32)
                            + (Voxel::SIZE * Chunk::SIZE as f32 * chunk.position.z),
                    );

                    commands.spawn(PbrBundle {
                        mesh: example_asset.mesh.clone_weak(),
                        material: example_asset.material.clone_weak(),
                        transform,
                        ..Default::default()
                    });
                }
            }
        }
    });
}

fn handle_input(
    timer: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_writer.send(AppExit::Success);
    }

    const SPEED: f32 = 10.0;
    let mut translate_camera = |translation: Vec3| {
        camera.single_mut().translation += translation * SPEED * timer.delta_seconds()
    };

    keys.get_pressed().for_each(|key| match key {
        KeyCode::KeyW => translate_camera(Vec3::Z),
        KeyCode::KeyS => translate_camera(-Vec3::Z),
        KeyCode::KeyA => translate_camera(Vec3::X),
        KeyCode::KeyD => translate_camera(-Vec3::X),
        KeyCode::Space => translate_camera(Vec3::Y),
        KeyCode::ShiftLeft => translate_camera(-Vec3::Y),
        _ => {}
    });
}
