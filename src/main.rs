use bevy::{prelude::*, input::mouse::MouseMotion, app::AppExit, window::CursorGrabMode};
use terrain_plane::TerrainPlanePlugin;

use crate::terrain_plane::TerrainPlane;

mod terrain_plane;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerrainPlanePlugin::default()))
        .add_systems(Startup, startup)
        .add_systems(Update, (update_move, update_look, exit_game))
        .run();
}

fn startup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, meshes: ResMut<Assets<Mesh>>, mut window: Query<&mut Window>) {
    println!("Hello, world!");

    let mut window = window.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    let material = materials.add(StandardMaterial {
        base_color: Color::CYAN,
        ..default()
    });

    let plane = TerrainPlane::new(meshes);
    let mesh = plane.mesh.clone();

    commands.spawn((plane, MaterialMeshBundle {
        mesh,
        material,
        ..default()
    }));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8., 16., 8.),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 6., 0.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

fn update_move(mut camera: Query<&mut Transform, With<Camera>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
    let mut camera = camera.single_mut();
    let forward = camera.forward();
    let right = camera.right();
    if keys.pressed(KeyCode::W) {
        camera.translation += forward * 10. * time.delta_seconds();
    }
    if keys.pressed(KeyCode::S) {
        camera.translation -= forward * 10. * time.delta_seconds();
    }
    if keys.pressed(KeyCode::A) {
        camera.translation -= right * 10. * time.delta_seconds();
    }
    if keys.pressed(KeyCode::D) {
        camera.translation += right * 10. * time.delta_seconds();
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
