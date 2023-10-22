use bevy::prelude::*;

#[derive(Default)]
pub struct FpsPlugin {}

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, update);
    }
}

#[derive(Component, Default)]
struct FPSTextBox {}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Monocraft.ttf");
    commands.spawn((FPSTextBox::default(), TextBundle {
        text: Text::from_section("", TextStyle { font, font_size: 16.0, color: Color::WHITE }),
        transform: Transform::from_translation(Vec3::ZERO),
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.),
            left: Val::Px(10.),
            ..default()
        },
        ..default()
    }));
}

fn update(mut query: Query<&mut Text, With<FPSTextBox>>, time: Res<Time>) {
    let mut text = query.single_mut();
    text.sections[0].value = ((1.0 / time.delta_seconds()).round() as i32).to_string()
}
