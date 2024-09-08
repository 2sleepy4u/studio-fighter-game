use bevy::prelude::*;
use bevy::{math::bounding::{IntersectsVolume, Aabb2d}, prelude::{Vec2, Query}};

//this is used for objects that needs to block, like player collider so they do not overlap 
#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub x: f32,
    pub y: f32
}

//this is used for hitboxes and hurtboxes
#[derive(Component, Debug)]
pub struct Trigger {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub height: f32
}

#[derive(Event, Debug)]
pub struct CollisionEvent {
    pub target: Entity,
    pub source: Entity,
    pub is_trigger: bool,
    //pub vector: Vec2,
}

pub fn check_triggers (
    mut target: Query<(Entity, &Trigger, &GlobalTransform)>,
    source: Query<(Entity, &Trigger, &Transform)>,
    mut ev_collision: EventWriter<CollisionEvent>,
) {
    for (source_entity, source_trigger, source_transform) in source.iter() {
        let position = 
            Vec2::new(source_transform.translation.x, source_transform.translation.y)
            +
            Vec2::new(source_trigger.x, source_trigger.y);
        let size = Vec2::new(source_trigger.x, source_trigger.y);
        let first_collider = Aabb2d::new(position, size);

        for (target_entity, target_trigger, target_transform) in target.iter_mut() {
            let position = 
                Vec2::new(target_transform.compute_transform().translation.x, target_transform.compute_transform().translation.y)
                +
                Vec2::new(source_trigger.x, source_trigger.y);
 
            let size = Vec2::new(target_trigger.x, target_trigger.y);
            let second_collider = Aabb2d::new(position, size);

            if source_entity != target_entity && first_collider.intersects(&second_collider) {
                ev_collision.send(CollisionEvent { 
                    target: target_entity,
                    source: source_entity,
                    is_trigger: true
                });
            }
        }
    }
}


