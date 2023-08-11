use std::string;

use bevy::{prelude::{App, Commands, AssetServer, Res, Transform, Component}, sprite::{SpriteBundle, Sprite}};
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
                            texture: assets.load("wall.png"),
                            transform: Transform::from_xyz((x as f32) * 100.0, (y as f32) * 100.0, 0.0),
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
    commands.spawn((
        SpriteBundle {
            texture: assets.load("player"),
            ..Default::default()
        },
        Position {
            x: 0.0,
            y: 0.0
        },
        Velocity {
            x: 0.0,
            y: 0.0
        },
    ));
}

fn main() {
    App::new()
        .add_startup_system(render_map)
        .add_startup_system(setup_player)
        .run();
}
