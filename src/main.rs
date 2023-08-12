use std::{default, string};

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, FilterMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
    sprite::{MaterialMesh2dBundle, Sprite, SpriteBundle},
    DefaultPlugins,
};
use serde::Deserialize;

const TILE_SIZE: f32 = 32.0;
const LEVEL_SIZE_X: f32 = 16.0;
const LEVEL_SIZE_Y: f32 = 16.0;

#[derive(Deserialize)]
struct LayerInstance {
    __cHei: i32,
    __cWid: i32,
    intGridCsv: Vec<i32>,
    __type: String,
    entityInstances: Vec<EntityInstance>,
}

#[derive(Deserialize)]
struct FieldInstance {
    __identifier: String,
    __type: String,
    __value: serde_json::Value,
}

#[derive(Deserialize)]
struct EntityInstance {
    __grid: Vec<i32>,
    __identifier: String,
    fieldInstances: Vec<FieldInstance>,
}

#[derive(Deserialize)]
struct Level {
    layerInstances: Vec<LayerInstance>,
}

#[derive(Component)]
struct AnimationIndex {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn render_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let level_raw = include_str!("../assets/level/Level_0.ldtkl");
    let level: Level = serde_json::from_str(level_raw).unwrap();
    let light_ray_texture_atlas = TextureAtlas::from_grid(
        assets.load("lightray.png"),
        Vec2::new(32.0, 32.0),
        5,
        1,
        None,
        None,
    );
    let light_ray_texture_handle = texture_atlases.add(light_ray_texture_atlas);

    for layer in level.layerInstances {
        match layer.__type.as_str() {
            "IntGrid" => {
                for x in 0..layer.__cWid {
                    for y in 0..layer.__cHei {
                        let index: usize = (x + layer.__cWid * y) as usize;
                        if layer.intGridCsv[index] == 0 {
                            continue;
                        }
                        commands.spawn(SpriteBundle {
                            texture: assets.load("tiles_middle.png"),
                            transform: Transform::from_xyz(
                                (x as f32) * TILE_SIZE,
                                (y as f32) * TILE_SIZE,
                                0.0,
                            ),
                            ..Default::default()
                        });
                    }
                }
            }
            "Entities" => {
                for entity in layer.entityInstances {
                    match entity.__identifier.as_str() {
                        "Lightray" => {
                            let mut color: String = String::default();
                            let mut dest_x: i32 = 0;
                            let mut dest_y: i32 = 0;
                            let src_x = entity.__grid[0];
                            let src_y = entity.__grid[1];
                            for field in entity.fieldInstances {
                                match field.__identifier.as_str() {
                                    "destination" => {
                                        let obj = field.__value.as_object().unwrap();
                                        dest_x = obj.get("cx").unwrap().as_i64().unwrap() as i32;
                                        dest_y = obj.get("cy").unwrap().as_i64().unwrap() as i32;
                                    }
                                    "color" => {
                                        color = field.__value.to_string();
                                    }
                                    "strength" => {}
                                    _ => (),
                                }
                            }

                            for x in dest_x..=src_x {
                                for y in dest_y..=src_y {
                                    let index = AnimationIndex { first: 0, last: 4 };
                                    commands.spawn((
                                        SpriteSheetBundle {
                                            texture_atlas: light_ray_texture_handle.clone(),
                                            transform: Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 50.),
                                            sprite: TextureAtlasSprite::new(index.first),
                                            ..Default::default()
                                        },
                                        index,
                                        AnimationTimer(Timer::from_seconds(
                                            0.1,
                                            TimerMode::Repeating,
                                        )),
                                    ));
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}

fn update_animations(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndex,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (index, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == index.last {
                index.first
            } else {
                sprite.index + 1
            }
        }
    }
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((SpriteBundle {
        texture: assets.load("player_v1.png"),
        transform: Transform::from_xyz(0., 0., 100.)
            .with_scale(bevy::prelude::Vec3::new(0.125, 0.125, 1.)),
        ..Default::default()
    },));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        transform: Transform::from_xyz(
            LEVEL_SIZE_X / 2. * TILE_SIZE,
            LEVEL_SIZE_Y / 2. * TILE_SIZE,
            1000.,
        ),
        ..Default::default()
    },));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(render_map)
        .add_startup_system(setup_player)
        .add_startup_system(setup_camera)
        .add_system(update_animations)
        .run();
}
