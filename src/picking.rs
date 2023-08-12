use bevy::{
    prelude::{
        Camera, Component, Entity, GlobalTransform, Plugin, Query,
        ResMut, Resource, Vec2, With,
    },
    render::camera::RenderTarget,
    window::{PrimaryWindow, Window},
};

// Components

#[derive(Component, Clone)]
pub struct Pickable {
    pub triangles: Vec<Triangle>,
}

#[derive(Component, Default)]
pub struct PickCamera;

// Resources

#[derive(Resource, Default)]
pub struct PickState {
    pub selected: Option<Entity>,
}

// Plugin

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(PickState::default());
        app.add_system(pick_input);
    }
}

fn pick_input(
    camera: Query<(&Camera, &GlobalTransform)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    pickables: Query<(&Pickable, &GlobalTransform, Entity)>,
    mut pick_state: ResMut<PickState>,
) {
    let (camera, camera_transform) = camera.single();
    // fuck off bevy docs
    let window = match camera.target {
        RenderTarget::Window(bevy::window::WindowRef::Primary) => primary_window.single(),
        // Ignore this
        // RenderTarget::Window(bevy::window::WindowRef::Entity(entity)) => windows.get(entity),
        _ => return,
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            pick_state.selected = pick_nearst(&pickables, &world_pos);
        }
    }
}

fn pick_nearst(
    pickables: &Query<(&Pickable, &GlobalTransform, Entity)>,
    world_pos: &Vec2,
) -> Option<Entity> {
    let mut nearest: Option<Entity> = None;
    let mut distance = -1.;
    for (pickable, transform, entity) in &(*pickables) {
        let obj_translation = transform.translation();
        let corrected_pos = Vec2::new(
            world_pos.x - obj_translation.x,
            world_pos.y - obj_translation.y,
        );

        if distance >= 0. && obj_translation.z < distance {
            continue;
        }

        for triangle in &pickable.triangles {
            if triangle.contains(&corrected_pos) {
                nearest = Some(entity);
                distance = obj_translation.z;
            }
        }
    }
    nearest
}

#[derive(Clone)]
pub struct Triangle {
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
}

impl Triangle {
    pub fn new(p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        Triangle { p1, p2, p3 }
    }
}

impl Triangle {
    fn sign(p1: &Vec2, p2: &Vec2, p3: &Vec2) -> f32 {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    }

    // see: https://stackoverflow.com/questions/2049582/how-to-determine-if-a-point-is-in-a-2d-triangle
    pub fn contains(&self, pt: &Vec2) -> bool {
        let (d1, d2, d3) = (
            Self::sign(pt, &self.p1, &self.p2),
            Self::sign(pt, &self.p2, &self.p3),
            Self::sign(pt, &self.p3, &self.p1),
        );

        let has_neg = (d1 < 0.) || (d2 < 0.) || (d3 < 0.);
        let has_pos = (d1 > 0.) || (d2 > 0.) || (d3 > 0.);

        !(has_neg && has_pos)
    }
}
