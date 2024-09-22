use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;

use crate::{components::*, GameState};


pub struct HitManagementPlugin;

impl Plugin for HitManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                manage_hitboxes,
                check_hitboxes,
                check_hits,
                check_hitstun
            ).run_if(in_state(GameState::InGame))
        ).add_event::<HitEvent>();
    }
}

pub fn check_hits(
    mut target: Query<(Entity, &Hurtbox, &GlobalTransform)>,
    source: Query<(Entity, &Hitbox, &Transform)>,
    mut ev_collision: EventWriter<HitEvent>,
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
                ev_collision.send(HitEvent { 
                    target: target_entity,
                    source: source_entity,
                });
            }
        }
    }
}

pub fn manage_hitboxes (
    mut commands: Commands,
    query: Query<(Entity, &PlayerAnimationManagement)>,
) {
    for (entity, player_animation) in &query {
        let (anim, attack) = &player_animation.get_current_animation();
        if anim.is_within_active() {
            //if is still runing then check for hitboxes
            //spawn hitbox
            if let Some(attack) = attack.clone() {
                //add attack to trigger or just check what type of attack active in other
                //player?
                commands.entity(entity).insert(attack.hitbox);
            }
        } else {
            commands.entity(entity).remove::<Hitbox>();
        }
    }
}


pub fn check_hitboxes (
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, &PlayerAnimationManagement)>,
    mut ev_collision: EventReader<HitEvent>,
) {
    for ev in ev_collision.read() {
        if let Some((entity, mut hp, player_aniation)) = query.iter_mut().find(|(entity, _, _)| &ev.target == entity) {
            let attack = player_aniation.get_current_animation().1.clone().unwrap();

            hp.damage(attack.damage);

            commands.entity(entity)
                .remove::<Hitbox>()
                .insert(HitStun::new(attack.hit_stun_frames, MAX_FRAME_RATE));
        }
    }
}

//if the hitstun timer is finished then it removes it
pub fn check_hitstun(
    mut commands: Commands,
    query: Query<(Entity, &HitStun)>
) {
    for (entity, hitstun) in &query {
        if hitstun.is_finished() {
            commands.entity(entity).remove::<HitStun>();
        }
    }
}
