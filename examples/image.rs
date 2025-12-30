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

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (Text::new("Displaying image from web")),
            (
                ImageNode::new(
                    asset_server.load("https://bevyengine.org/news/bevy-0-11/with_ssao.png")
                ),
                Node {
                    height: Val::Px(300.0),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                }
            )
        ],
    ));
}
