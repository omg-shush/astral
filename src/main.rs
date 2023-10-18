use std::f32::consts::PI;
use bevy_framepace::{Limiter, FramepaceSettings, FramepacePlugin};
use rand::random;

use bevy::{prelude::*, input::mouse::MouseMotion, app::AppExit, window::CursorGrabMode, log::{LogPlugin, Level}};
use sky_plane::{SkyPlaneMaterial, SkyPlanePlugin};
use terrain_plane::TerrainPlaneMaterial;

use crate::{terrain_plane::{TerrainPlane, TerrainPlanePlugin}, sky_plane::SkyPlane};

mod terrain_plane;
mod sky_plane;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.94, 0.97, 1.0) * 0.8))
        .add_plugins((DefaultPlugins.set(LogPlugin {filter: "warn,wgpu_hal=off".to_string(), level: Level::WARN}), FramepacePlugin {}))
        .add_plugins((TerrainPlanePlugin::default(), SkyPlanePlugin::default()))
        .add_systems(Startup, startup)
        .add_systems(Update, (update_move, update_look, exit_game, use_mouse))
        .run();
}

fn startup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_materials: ResMut<Assets<TerrainPlaneMaterial>>,
    mut sky_materials: ResMut<Assets<SkyPlaneMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut window: Query<&mut Window>,
    mut frames: ResMut<FramepaceSettings>
) {
    println!("Hello, world!");

    let mut window = window.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    frames.limiter = Limiter::from_framerate(60.);

    // Terrain
    let perlin_1 = perlin_2d(100);
    let perlin_2 = perlin_2d(100);
    let perlin_3 = perlin_2d(100);
    let perlin_4 = perlin_2d(100);
    let terrain_heightmap = |x: f32, y: f32| {
        [
            perlin_1(x / 3., y / 3.),
            perlin_2(x / 13., y / 13.) * 4.,
            perlin_3(x / 43., y / 43.) * 16.,
            perlin_4(x / 197., y / 197.) * 64.
        ].iter().sum()
    };
    let terrain = TerrainPlane::new(&mut meshes, &mut terrain_materials, &mut images, terrain_heightmap);
    let terrain_handle = terrain.mesh.clone();
    let terrain_material_handle = terrain.material.clone();
    commands.spawn((terrain, MaterialMeshBundle {
        mesh: terrain_handle,
        material: terrain_material_handle.clone(),
        ..default()
    }));

    // Sky
    let sky = SkyPlane::new(&mut meshes, &mut sky_materials, &mut images);
    let sky_handle = sky.mesh.clone();
    let sky_material_handle = sky.material.clone();
    commands.spawn((sky, MaterialMeshBundle {
        mesh: sky_handle.clone(),
        material: sky_material_handle.clone(),
        transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI)).with_translation(Vec3::Y * 512.),
        ..default()
    }));

    // Water
    let material_water = materials.add(StandardMaterial {
        base_color: Color::rgba(0.2, 0.2, 0.9, 0.45),
        reflectance: 0.4,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let perlin_1 = perlin_2d(100);
    let perlin_2 = perlin_2d(100);
    let heightmap = |x: f32, y: f32| {
        [
            perlin_1(x / 1.5, y / 1.5) * 0.3,
            perlin_2(x / 7., y / 7.) * 0.8,
        ].iter().sum()
    };
    let water = TerrainPlane::new(&mut meshes, &mut terrain_materials, &mut images, heightmap);
    let water2 = TerrainPlane::new(&mut meshes, &mut terrain_materials, &mut images, |x, y| heightmap(x + 34., y - 12.));
    let water3 = TerrainPlane::new(&mut meshes, &mut terrain_materials, &mut images, |x, y| heightmap(x + 11., y + 64.));
    let water4 = TerrainPlane::new(&mut meshes, &mut terrain_materials, &mut images, |x, y| heightmap(x - 22., y - 36.));
    commands.spawn(MaterialMeshBundle {
        mesh: water.mesh.clone(),
        material: material_water.clone(),
        transform: Transform::from_translation(-16. * Vec3::Y),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: water2.mesh.clone(),
        material: material_water.clone(),
        transform: Transform::from_translation(-17. * Vec3::Y),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: water3.mesh.clone(),
        material: material_water.clone(),
        transform: Transform::from_translation(-18. * Vec3::Y),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: water4.mesh.clone(),
        material: material_water.clone(),
        transform: Transform::from_translation(-19. * Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 8000.,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_translation(Vec3::ZERO).looking_at(Vec3::new(-5., -3., -8.), Vec3::Y),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(1024., 1024., 1024.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

fn update_move(mut camera: Query<&mut Transform, With<Camera>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
    let mut camera = camera.single_mut();
    let forward = camera.forward();
    let right = camera.right();
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 100. } else { 10. };
    if keys.pressed(KeyCode::W) {
        camera.translation += forward * speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::S) {
        camera.translation -= forward * speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::A) {
        camera.translation -= right * speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::D) {
        camera.translation += right * speed * time.delta_seconds();
    }
}

fn update_look(mut camera: Query<&mut Transform, With<Camera>>, mut mouse: EventReader<MouseMotion>, time: Res<Time>, window: Query<&Window>) {
    let window = window.single();
    if window.cursor.visible {
        return; // disable look while cursor is released
    }

    let mut camera = camera.single_mut();
    let (mut delta_x, mut delta_y) = (0., 0.);
    let (up, right) = (camera.up(), camera.right());
    for evt in mouse.iter() {
        delta_x += evt.delta.x;
        delta_y += evt.delta.y;
    }
    camera.rotate_axis(up, delta_x * -0.05 * time.delta_seconds());
    camera.rotate_axis(right, delta_y * -0.05 * time.delta_seconds());
}

fn exit_game(keys: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>, assets: Res<Assets<TerrainPlaneMaterial>>, assets2: Res<Assets<SkyPlaneMaterial>>) {
    if keys.pressed(KeyCode::Escape) {
        assets.iter().for_each(|(_, mat)| println!("{:#?}", mat));
        assets2.iter().for_each(|(_, mat)| println!("{:#?}", mat));
        exit.send(AppExit::default());
    }
}

fn use_mouse(keys: Res<Input<KeyCode>>, mut window: Query<&mut Window>) {
    if keys.just_pressed(KeyCode::ControlLeft) {
        let mut window = window.single_mut();
        if window.cursor.visible {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        } else {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn perlin_2d(size: usize) -> impl Fn(f32, f32) -> f32 {
    let mut map = vec![vec![Vec2::ZERO; size]; size];
    for xi in 0..size {
        for yi in 0..size {
            map[xi][yi] = Vec2::from_angle(2. * PI * random::<f32>());
        }
    }
    move |x, y| {
        let (xi, yi) = (x.rem_euclid(size as f32), y.rem_euclid(size as f32));
        let (xi_f, yi_f) = (xi.floor(), yi.floor());
        let points = vec![
            Vec2::new(xi_f, yi_f),
            Vec2::new(xi_f, yi_f+1.),
            Vec2::new(xi_f+1., yi_f),
            Vec2::new(xi_f+1., yi_f+1.),
        ];
        let gradients = points.iter().map(|p| map[p.x as usize % size][p.y as usize % size]); // % size in case floating point error
        let offsets = points.iter().map(|p| Vec2::new(xi, yi) - *p);
        let dot_offsets = gradients.zip(offsets).map(|(g, o)| (g.dot(o), o)).collect::<Vec<_>>();
        let smoothstep = |x: f32| 6. * x.powi(5) - 15. * x.powi(4) + 10. * x.powi(3);
        let interp = |a, b, p| a + (b - a) * smoothstep(p);
        let interp_x = |(d1, p1): (f32, Vec2), (d2, _)| (interp(d1, d2, p1.x), Vec2::new(0., p1.y));
        let interp_y = |(d1, p1): (f32, Vec2), (d2, _)| interp(d1, d2, p1.y);
        interp_y(interp_x(dot_offsets[0b00], dot_offsets[0b10]), interp_x(dot_offsets[0b01], dot_offsets[0b11]))
    }
}

fn perlin_3d(size: usize) -> impl Fn(f32, f32, f32) -> f32 {
    let mut map = vec![vec![vec![Vec3::ZERO; size]; size]; size];
    for xi in 0..size {
        for yi in 0..size {
            for zi in 0..size {
                let (dx, dy, dz) = random();
                map[xi][yi][zi] = Vec3::new(dx, dy, dz).normalize();
            }
        }
    }
    move |x, y, z| {
        let (xi, yi, zi) = (x.rem_euclid(size as f32), y.rem_euclid(size as f32), z.rem_euclid(size as f32));
        let (xi_f, yi_f, zi_f) = (xi.floor(), yi.floor(), zi.floor());
        let mut points = Vec::new();
        for dx in 0..=1 {
            for dy in 0..=1 {
                for dz in 0..=1 {
                    points.push(Vec3::new(xi_f + dx as f32, yi_f + dy as f32, zi_f + dz as f32));
                }
            }
        }
        // modulo size again in case floating point error
        let gradients = points.iter().map(|p| map[p.x as usize % size][p.y as usize % size][p.z as usize % size]);
        let offsets = points.iter().map(|p| Vec3::new(xi, yi, zi) - *p);
        let dot_offsets = gradients.zip(offsets).map(|(g, o)| (g.dot(o), o)).collect::<Vec<_>>();
        let smoothstep = |x: f32| 6. * x.powi(5) - 15. * x.powi(4) + 10. * x.powi(3);
        let interp = |a, b, p| a + (b - a) * smoothstep(p);
        let interp_x = |(d1, p1): (f32, Vec3), (d2, _)| (interp(d1, d2, p1.x), Vec3::new(0., p1.y, p1.z));
        let interp_y = |(d1, p1): (f32, Vec3), (d2, _)| (interp(d1, d2, p1.y), Vec3::new(0., 0., p1.z));
        let interp_z = |(d1, p1): (f32, Vec3), (d2, _)| interp(d1, d2, p1.z);
        interp_z(
            interp_y(
                interp_x(dot_offsets[0b000], dot_offsets[0b100]),
                interp_x(dot_offsets[0b010], dot_offsets[0b110]),
            ),
            interp_y(
                interp_x(dot_offsets[0b001], dot_offsets[0b101]),
                interp_x(dot_offsets[0b011], dot_offsets[0b111])
            )
        )
    }
}
