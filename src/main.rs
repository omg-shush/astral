use std::f32::consts::PI;
use rand::random;

use bevy::{prelude::*, input::mouse::MouseMotion, app::AppExit, window::CursorGrabMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};

use crate::terrain_plane::{TerrainPlane, TerrainPlanePlugin};

mod terrain_plane;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba(0.7, 0.7, 1.0, 1.0)))
        .add_plugins((DefaultPlugins, TerrainPlanePlugin::default(), FramepacePlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, (update_move, update_look, exit_game))
        .run();
}

fn startup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, mut meshes: ResMut<Assets<Mesh>>, mut window: Query<&mut Window>, mut frames: ResMut<FramepaceSettings>) {
    println!("Hello, world!");

    let mut window = window.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    frames.limiter = Limiter::from_framerate(60.);

    // Terrain
    let material_terrain = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        reflectance: 0.1,
        ..default()
    });
    let perlin_1 = perlin(100);
    let perlin_2 = perlin(100);
    let perlin_3 = perlin(100);
    let perlin_4 = perlin(100);
    let terrain_heightmap = |x: f32, y: f32| {
        [
            perlin_1(x / 3., y / 3.),
            perlin_2(x / 13., y / 13.) * 4.,
            perlin_3(x / 43., y / 43.) * 16.,
            perlin_4(x / 197., y / 197.) * 64.
        ].iter().sum()
    };
    let terrain = TerrainPlane::new(&mut meshes, terrain_heightmap);
    let terrain_handle = terrain.mesh.clone();
    commands.spawn((terrain, MaterialMeshBundle {
        mesh: terrain_handle,
        material: material_terrain,
        ..default()
    }));

    // Rocks
    let material_rocks = materials.add(StandardMaterial {
        base_color: Color::DARK_GRAY,
        reflectance: 0.001,
        ..default()
    });
    let perlin_1 = perlin(100);
    let perlin_2 = perlin(100);
    let perlin_3 = perlin(100);
    let perlin_4 = perlin(100);
    let rocks_heightmap = |x: f32, y: f32| {
        let t = terrain_heightmap(x, y);
        if t < 8. || t > 26. {
            return -32.;
        }
        [
            t,
            perlin_1(x / 3., y / 3.) * 1.,
            perlin_2(x / 13., y / 13.) * 2.,
            perlin_3(x / 23., y / 23.) * 4.,
            perlin_4(x / 197., y / 197.) * 0.5,
            -2.
        ].iter().sum()
    };
    let rocks = TerrainPlane::new(&mut meshes, rocks_heightmap);
    let rocks_handle = rocks.mesh.clone();
    commands.spawn((rocks, MaterialMeshBundle {
        mesh: rocks_handle,
        material: material_rocks,
        ..default()
    }));

    // Water
    let material_water = materials.add(StandardMaterial {
        base_color: Color::rgba(0.2, 0.2, 0.9, 0.45),
        reflectance: 0.4,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let perlin_1 = perlin(100);
    let perlin_2 = perlin(100);
    let heightmap = |x: f32, y: f32| {
        [
            perlin_1(x / 1.5, y / 1.5) * 0.3,
            perlin_2(x / 7., y / 7.) * 0.8,
        ].iter().sum()
    };
    let water = TerrainPlane::new(&mut meshes, heightmap);
    let water2 = TerrainPlane::new(&mut meshes, |x, y| heightmap(x + 34., y - 12.));
    let water3 = TerrainPlane::new(&mut meshes, |x, y| heightmap(x + 11., y + 64.));
    let water4 = TerrainPlane::new(&mut meshes, |x, y| heightmap(x - 22., y - 36.));
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
        transform: Transform::from_xyz(0., 64., 12.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
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

fn update_look(mut camera: Query<&mut Transform, With<Camera>>, mut mouse: EventReader<MouseMotion>, time: Res<Time>) {
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

fn exit_game(keys: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit::default());
    }
}

fn perlin(size: usize) -> impl Fn(f32, f32) -> f32 {
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
