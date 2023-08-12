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

fn render_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LightRayMaterial>>,
) {
    let level_raw = include_str!("../assets/level/Level_0.ldtkl");
    let level: Level = serde_json::from_str(level_raw).unwrap();

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
                            for field in entity.fieldInstances {
                                match field.__identifier.as_str() {
                                    "direction" => {}
                                    "color" => {
                                        color = field.__value.to_string();
                                    }
                                    "strength" => {}
                                    _ => (),
                                }
                            }

                            commands.spawn((
                                SpriteBundle {
                                    texture: assets.load("tiles_middle.png"),
                                    transform: Transform::from_xyz(100., 100., 200.),
                                    ..Default::default()
                                },
                                materials.add(LightRayMaterial {
                                    color: Color::rgb(255., 100., 0.),
                                }),
                            ));
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
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

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct LightRayMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LightRayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lightray.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/lightray.vert".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(MaterialPlugin::<LightRayMaterial>::default())
        .add_startup_system(render_map)
        .add_startup_system(setup_player)
        .add_startup_system(setup_camera)
        .run();
}
