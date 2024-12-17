use bevy::prelude::*;
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((HttpPlugin, DefaultPlugins))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(BackgroundColor(Color::WHITE))
        .insert(ImageNode::new(
            asset_server.load("https://bevyengine.org/news/bevy-0-11/with_ssao.png"),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Displaying image from web"));
        });
}
