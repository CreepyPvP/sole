use std::{default, string};

use bevy::{
    input::mouse::MouseButtonInput,
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
    sprite::{Anchor, MaterialMesh2dBundle, Sprite, SpriteBundle},
    DefaultPlugins,
};
use picking::{PickCamera, PickState, Pickable, PickingPlugin, Triangle};
use serde::Deserialize;

mod picking;

const TILE_SIZE: f32 = 32.0;
const LEVEL_SIZE_X: f32 = 16.0;
const LEVEL_SIZE_Y: f32 = 16.0;
const PLAYER_SPEED: f32 = 2.;

static mut light_ray_texture_handle: Option<Handle<TextureAtlas>> = None; 

const RAY_COLORS: [Color; 4] = [
    Color::rgb(255. / 255., 206. / 255., 92. / 255.),
    Color::rgb(235. / 255., 171. / 255., 52. / 255.),
    Color::rgb(165. / 255., 224. / 255., 47. / 255.),
    Color::rgb(69. / 255., 97. / 255., 237. / 255.),
];

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
struct RayCaster {
    dir: Dir,
    pos_x: i32,
    pos_y: i32,
}

#[derive(Component)]
struct AnimationIndex {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Ray {
    src_x: i32,
    src_y: i32,
    dest_x: i32,
    dest_y: i32,
    reversed: bool,
    horizontal: bool,
    // smaller prio value means its above rays with higher value
    prio: i32,
}

#[derive(Resource, Default, Clone)]
struct GameState {
    ray_count: i32,
}

#[derive(Eq, PartialEq, Clone)]
enum Dir {
    Upwards,
    Downwards,
    Leftwards,
    Rightwards,
}

#[derive(Component)]
struct Player {
    x: f32,
    y: f32,
    direction: Option<Dir>,
    last_direction: Option<Dir>,
}

fn setup_textures(assets: Res<AssetServer>, mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    let light_ray_texture_atlas = TextureAtlas::from_grid(
        // assets.load("lightray.png"),
        assets.load("photon_ray_6spd_white_40alpha.png"),
        Vec2::new(32.0, 32.0),
        6,
        1,
        None,
        None,
    );
    unsafe {
        light_ray_texture_handle = Some(texture_atlases.add(light_ray_texture_atlas));
    }
}

fn render_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    let level_raw = include_str!("../assets/level/Level_0.ldtkl");
    let level: Level = serde_json::from_str(level_raw).unwrap();
    for layer in level.layerInstances {
        match layer.__type.as_str() {
            "IntGrid" => {
                let pickable = Pickable {
                    triangles: vec![
                        Triangle::new(
                            Vec2::new(-16., 16.),
                            Vec2::new(16., 16.),
                            Vec2::new(-16., -16.),
                        ),
                        Triangle::new(
                            Vec2::new(16., 16.),
                            Vec2::new(-16., -16.),
                            Vec2::new(16., -16.),
                        ),
                    ],
                };
                for x in 0..layer.__cWid {
                    for y in 0..layer.__cHei {
                        let index: usize = (x + layer.__cWid * y) as usize;
                        let value = layer.intGridCsv[index];
                        match value {
                            1 => {
                                commands.spawn(SpriteBundle {
                                    texture: assets.load("tiles_middle.png"),
                                    transform: Transform::from_xyz(
                                        (x as f32) * TILE_SIZE,
                                        -(y as f32) * TILE_SIZE,
                                        0.0,
                                    ),
                                    ..Default::default()
                                });
                            }
                            2..=5 => {
                                let dir: Dir = match value {
                                    2 => Dir::Downwards,
                                    3 => Dir::Upwards,
                                    4 => Dir::Rightwards,
                                    5 => Dir::Leftwards,
                                    // wtf are we doing?
                                    _ => Dir::Leftwards,
                                };
                                // pickable tiles
                                commands.spawn((
                                    SpriteBundle {
                                        texture: assets.load("tiles_middle.png"),
                                        transform: Transform::from_xyz(
                                            (x as f32) * TILE_SIZE,
                                            -(y as f32) * TILE_SIZE,
                                            0.0,
                                        ),
                                        ..Default::default()
                                    },
                                    pickable.clone(),
                                    RayCaster {
                                        dir,
                                        pos_x: x,
                                        pos_y: y,
                                    },
                                ));
                            }
                            _ => (),
                        }
                    }
                }
            }
            "Entities" => {
                for entity in layer.entityInstances {
                    match entity.__identifier.as_str() {
                        "Lightray" => {
                            let mut dest_x: i32 = 0;
                            let mut dest_y: i32 = 0;
                            let mut src_x = entity.__grid[0];
                            let mut src_y = entity.__grid[1];
                            let mut prio = 0;
                            for field in entity.fieldInstances {
                                match field.__identifier.as_str() {
                                    "destination" => {
                                        let obj = field.__value.as_object().unwrap();
                                        dest_x = obj.get("cx").unwrap().as_i64().unwrap() as i32;
                                        dest_y = obj.get("cy").unwrap().as_i64().unwrap() as i32;
                                    }
                                    "priority" => {
                                        prio = field.__value.as_i64().unwrap() as i32;
                                    }
                                    _ => (),
                                }
                            }
                            spawn_ray(src_x, src_y, dest_x, dest_y, prio, &mut commands);
                            game_state.ray_count += 1;
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

fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: assets.load("asymmetric_spaceship_64.png"),
            sprite: Sprite {
                anchor: Anchor::Center,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., -100., 200.)
                .with_scale(bevy::prelude::Vec3::new(0.5, 0.5, 1.)),
            ..Default::default()
        },
        Player {
            x: 3.,
            y: 8.,
            direction: None,
            last_direction: None,
        },
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(
                LEVEL_SIZE_X / 2. * TILE_SIZE,
                -LEVEL_SIZE_Y / 2. * TILE_SIZE,
                1000.,
            ),
            ..Default::default()
        },
        PickCamera,
    ));
}

fn move_player(
    time: Res<Time>,
    mut q_player: Query<(&mut Player, &mut Transform)>,
    q_ray: Query<&Ray>,
) {
    for (mut player, mut transform) in &mut q_player {
        let mut heighest_prio = 999999;
        let mut x_diff = 0.;
        let mut y_diff = 0.;

        let player_x = player.x as i32;
        let player_y = player.y as i32;
        let mut new_direction: Option<Dir> = None;
        for ray in &q_ray {
            let mut y_offset = 0;
            let mut x_offset = 0;

            if (matches!(player.direction, Some(Dir::Upwards))
                || (matches!(player.direction, Some(Dir::Rightwards))
                    && matches!(player.last_direction, Some(Dir::Upwards))))
                && ray.reversed
                && ray.horizontal
            {
                y_offset = -1;
            }
            if (matches!(player.direction, Some(Dir::Upwards))
                || (matches!(player.direction, Some(Dir::Leftwards))
                    && matches!(player.last_direction, Some(Dir::Upwards))))
                && !ray.reversed
                && ray.horizontal
            {
                y_offset = -1;
            }
            if (matches!(player.direction, Some(Dir::Leftwards))
                || (matches!(player.direction, Some(Dir::Downwards))
                    && matches!(player.last_direction, Some(Dir::Leftwards))))
                && !ray.reversed
                && !ray.horizontal
            {
                x_offset = -1;
            }
            if (matches!(player.direction, Some(Dir::Leftwards))
                || (matches!(player.direction, Some(Dir::Upwards))
                    && matches!(player.last_direction, Some(Dir::Leftwards))))
                && ray.reversed
                && !ray.horizontal
            {
                x_offset = -1;
            }

            if player_x >= ray.src_x + x_offset
                && player_x <= ray.dest_x + x_offset
                && player_y >= ray.src_y + y_offset
                && player_y <= ray.dest_y + y_offset
            {
                if ray.prio > heighest_prio {
                    continue;
                }
                heighest_prio = ray.prio;
                if ray.horizontal {
                    y_diff = 0.;
                    if ray.reversed {
                        x_diff = PLAYER_SPEED;
                        new_direction = Some(Dir::Rightwards);
                    } else {
                        x_diff = -PLAYER_SPEED;
                        new_direction = Some(Dir::Leftwards);
                    }
                } else {
                    x_diff = 0.;
                    if ray.reversed {
                        y_diff = -PLAYER_SPEED;
                        new_direction = Some(Dir::Upwards);
                    } else {
                        y_diff = PLAYER_SPEED;
                        new_direction = Some(Dir::Downwards);
                    }
                }
            }
        }

        player.x += time.delta().as_secs_f32() * x_diff;
        player.y += time.delta().as_secs_f32() * y_diff;
        // if (player.x as i32) != player_x || (player.y as i32) != player_y {
        // }
        if player.direction != new_direction {
            player.last_direction = player.direction.clone();
            player.direction = new_direction;
        }
        transform.translation = Vec3::new(player.x * TILE_SIZE, -player.y * TILE_SIZE, 200.);
    }
}

fn spawn_ray(mut src_x: i32, mut src_y: i32, mut dest_x: i32, mut dest_y: i32, prio: i32, commands: &mut Commands) {
    let horizontal = dest_y == src_y;
    let mut reversed = true;

    if src_x > dest_x {
        let tmp = src_x;
        src_x = dest_x;
        dest_x = tmp;
        reversed = false;
    }

    if src_y >= dest_y {
        let tmp = src_y;
        src_y = dest_y;
        dest_y = tmp;
    } else {
        reversed = false;
    }

    let mut rot = 3.14;
    if reversed {
        rot = 0.;
    }
    if !horizontal {
        rot += 3.14 / 2.;
    }

    // let prio = game_state.ray_count;
    commands.spawn(Ray {
        src_x,
        src_y,
        dest_x,
        dest_y,
        horizontal,
        reversed,
        prio,
    });

    let mut light_ray_handle: Option<Handle<TextureAtlas>> = None;
    unsafe {
        light_ray_handle = light_ray_texture_handle.clone();
    }

    for x in src_x..=dest_x {
        for y in src_y..=dest_y {
            let mut transform = Transform::from_xyz(
                x as f32 * TILE_SIZE,
                -y as f32 * TILE_SIZE,
                99. - prio as f32,
            );
            transform.rotate_z(rot);
            let index = AnimationIndex { first: 0, last: 5 };
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: light_ray_handle.clone().unwrap(),
                    transform,
                    sprite: TextureAtlasSprite {
                        index: index.first,
                        // TODO: fix tint
                        color: RAY_COLORS[prio as usize % RAY_COLORS.len()],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                index,
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ));
        }
    }
}

fn update_hover_tint(
    pick_state: Res<PickState>,
    mut q_sprite: Query<(&mut Sprite, Entity, &RayCaster)>,
    mouse: Res<Input<MouseButton>>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    for (mut sprite, entity, ray_caster) in &mut q_sprite {
        if pick_state.selected.is_some() && pick_state.selected.unwrap() == entity {
            sprite.color = Color::rgb(1.2, 1.2, 1.2);

            if mouse.just_pressed(MouseButton::Left) {
                match ray_caster.dir {
                    Dir::Upwards => {
                        let prio = game_state.ray_count;
                        spawn_ray(
                            ray_caster.pos_x,
                            ray_caster.pos_y,
                            ray_caster.pos_x,
                            -LEVEL_SIZE_Y as i32,
                            prio,
                            &mut commands
                        );
                    }
                    Dir::Downwards => {}
                    Dir::Leftwards => {}
                    Dir::Rightwards => {}
                }
                game_state.ray_count += 1;
            }
        } else {
            sprite.color = Color::rgb(1., 1., 1.);
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, SystemSet)]
pub enum GameSystemSets {
    Input,
    Logic,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(PickingPlugin)
        .insert_resource(GameState::default())

        .add_startup_system(render_map.after(setup_textures))
        .add_startup_system(setup_player)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_textures)

        .configure_set(GameSystemSets::Input)
        .configure_set(GameSystemSets::Logic.after(GameSystemSets::Input))
        .add_systems(
            (update_animations, move_player, update_hover_tint).in_set(GameSystemSets::Logic),
        )
        .run();
}
