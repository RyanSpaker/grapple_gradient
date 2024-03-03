use bevy::prelude::*;
mod grapple;
use grapple::grapple_plugin::GrapplePlugin;

#[derive(Component)]
pub struct MainCamera{}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins((
            DefaultPlugins, 
            GrapplePlugin{}
        ))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
) {
    //spawn camera
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera{}
    ));
}