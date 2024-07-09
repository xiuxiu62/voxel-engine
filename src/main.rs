use bevy::{
    app::{AppExit, Update},
    asset::{AssetServer, Assets, Handle},
    color::Color,
    core_pipeline::core_3d::{Camera3d, Camera3dBundle},
    ecs::{
        component::Component,
        event::Events,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{vec3, Vec3},
    pbr::{light_consts, DirectionalLight, DirectionalLightBundle, PbrBundle, StandardMaterial},
    prelude::{default, App, PluginGroup, Startup},
    render::{
        camera::ClearColor,
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        settings::{Backends, RenderCreation, WgpuSettings},
        texture::Image,
        RenderPlugin,
    },
    time::Time,
    transform::components::Transform,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};

const TITLE: &str = "Voxel";

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
    // chunks: Query<&Chunk>,
) {
    (0..3)
        .flat_map(|x| (0..3).map(move |z| Chunk::new(vec3(x as f32, 0.0, z as f32), 8)))
        .for_each(|mut chunk| {
            for x in 0..8 {
                for y in 0..8 {
                    for z in 0..8 {
                        chunk.set(x, y, z, Voxel { id: 1 });
                    }
                }
            }

            commands.spawn(chunk);
        });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(vec3(0.0, 0.0, -10.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let texture: Handle<Image> = asset_server.load("array_texture.png");
    let mesh = meshes.add(generate_cube_mesh());
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
        for x in 0..chunk.size {
            for y in 0..chunk.size {
                for z in 0..chunk.size {
                    if chunk.get(x, y, z).is_none() {
                        return;
                    }

                    let transform = Transform::from_xyz(
                        (Voxel::SIZE * x as f32)
                            + (Voxel::SIZE * chunk.size as f32 * chunk.position.x),
                        Voxel::SIZE * y as f32,
                        (Voxel::SIZE * z as f32)
                            + (Voxel::SIZE * chunk.size as f32 * chunk.position.z),
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

#[derive(Debug, Resource)]
struct ExampleAsset {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Voxel {
    id: u8,
}

impl Voxel {
    const SIZE: f32 = 1.0;
}

#[derive(Debug, Component)]
struct Chunk {
    voxels: Vec<Voxel>,
    position: Vec3,
    size: usize,
}

impl Chunk {
    #[inline]
    pub fn new(position: Vec3, size: usize) -> Self {
        Self {
            voxels: vec![Voxel { id: 0 }; size * size * size],
            position,
            size,
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        self.voxels.get(self.flatten_cartesian(x, y, z))
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: Voxel) {
        if x < self.size && y < self.size && z < self.size {
            let i = self.flatten_cartesian(x, y, z);
            self.voxels[i] = value;
        }
    }

    #[inline]
    const fn flatten_cartesian(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.size * self.size) + (y * self.size) + x
    }
}

fn handle_input(
    timer: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    // let (mut velocity, transform) = query.single_mut();

    // if keys.just_pressed(KeyCode::Escape) {
    //     app_exit_events.send(AppExit);
    // }

    // if *cursor_state.as_ref() == CursorState::Ungrabbed {
    //     return;
    // }

    // mouse_motion.read().for_each(|event| {
    //     let force = ACCELERATION * timer.delta_seconds();

    //     // mouse position deltas
    //     let Vec2 {
    //         x: delta_x,
    //         y: delta_y,
    //     } = event.delta;

    //     // transform deltas
    //     let delta_x = up(transform.rotation) * (-delta_x * force / 20.0);
    //     let delta_y = right(transform.rotation) * (-delta_y * force / 20.0);
    //     velocity.angvel += delta_x + delta_y;
    // });

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

pub fn generate_cube_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5],  // vertex with index 1
            [0.5, 0.5, 0.5],   // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    )
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.2],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.2],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.45],
            [0.0, 0.25],
            [1.0, 0.25],
            [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
        ],
    )
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    )
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    .with_inserted_indices(Indices::U32(vec![
        0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
        4, 5, 7, 5, 6, 7, // bottom (-y)
        8, 11, 9, 9, 11, 10, // right (+x)
        12, 13, 15, 13, 14, 15, // left (-x)
        16, 19, 17, 17, 19, 18, // back (+z)
        20, 21, 23, 21, 22, 23, // forward (-z)
    ]))
}
