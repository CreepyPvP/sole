use std::string;

use bevy::{prelude::{App, Commands, AssetServer, Res, Transform, Component, Camera2dBundle}, sprite::{SpriteBundle, Sprite}, DefaultPlugins};
use serde::Deserialize;

#[derive(Deserialize)]
struct LayerInstance {
    __cHei: i32,
    __cWid: i32,
    intGridCsv: Vec<i32>,
    __type: String,
}

#[derive(Deserialize)]
struct Level {
    layerInstances: Vec<LayerInstance>
}

fn render_map(mut commands: Commands, assets: Res<AssetServer>) {
    let level_raw = include_str!("../assets/level/Level_0.ldtkl");
    let level: Level = serde_json::from_str(level_raw).unwrap();

    for layer in level.layerInstances {
        if layer.__type == "IntGrid" {
            for x in 0..layer.__cWid {
                for y in 0..layer.__cHei {
                    let index: usize = (x + layer.__cWid * y) as usize;
                    if layer.intGridCsv[index] == 0 {
                        continue;
                    }
                    commands.spawn(
                        SpriteBundle {
                            texture: assets.load("tiles_middle.png"),
                            transform: Transform::from_xyz((x as f32) * 32.0, (y as f32) * 32.0, 0.0),
                            ..Default::default()
                        }
                    );
                }
            }
        }
    }

}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    // commands.spawn((
    //     SpriteBundle {
    //         texture: assets.load("player"),
    //         ..Default::default()
    //     },
    //     Position {
    //         x: 0.0,
    //         y: 0.0
    //     },
    //     Velocity {
    //         x: 0.0,
    //         y: 0.0
    //     },
    // ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 1000.),
            ..Default::default()
        },
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(render_map)
        .add_startup_system(setup_player)
        .add_startup_system(setup_camera)
        .run();
}
