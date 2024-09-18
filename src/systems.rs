use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;

use crate::components::*;


pub fn check_hits (
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


pub fn execute_animations(
    time: Res<Time>,
    mut query: Query<( &mut PlayerAnimationManagement, &mut TextureAtlas)>,
) {
    for (mut player_animation, mut atlas) in &mut query {
        //scope to drop automatically anim so the mutable reference stop existing
        {
            //get current animation data
            let (animation_manager, _) = &mut player_animation.get_current_animation_mut();
            // we track how long the current sprite has been displayed for
            animation_manager.tick(time.delta());
        }


        let (animation_manager, _) = player_animation.get_current_animation().clone();
        // If it has been displayed for the user-defined amount of time (fps)...
        if animation_manager.is_just_finished() {
            if animation_manager.is_last_frame() {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                {
                    let (animation_manager, _) = &mut player_animation.get_current_animation_mut();
                    animation_manager.reset(MAX_FRAME_RATE);
                }                
                player_animation.shift();
                let (animation_manager, _) = &mut player_animation.get_current_animation_mut();
                animation_manager.reset(MAX_FRAME_RATE);
                atlas.index = animation_manager.current_sprite_index().unwrap().into();
            } else {
                let (animation_manager, _) = &mut player_animation.get_current_animation_mut();
                // ...and it is NOT the last frame, then we move to the next frame...
                animation_manager.next_frame(MAX_FRAME_RATE);
                atlas.index = animation_manager.current_sprite_index().unwrap().into();
                // ...and reset the frame timer to start counting all over again
                
                animation_manager.start_timer(MAX_FRAME_RATE);
            }
        }
    }
}


pub fn execute_hitboxes (
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

pub fn keyboard_input_system(
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity,
        &mut Transform, &mut PlayerAnimationManagement, &Speed,
        &mut Velocity,
        ), Without<HitStun>>,
) {
    //when pressing a button
    //it needs to ask/check if move is legal
    //it is legal if no move is active 
    //or if it falls withing cancel and linking rules
    for (entity, mut transform, mut animation, speed, mut velocity) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) { 
            //implement jump
            //check if is grounded
            if transform.translation.y == 0. && animation.request_animation(AnimationState::Jump) {
                //if it is then it can jump
                velocity.y = 7.
            }
        }

        //checks for button pressed and request response
        if keyboard_input.pressed(KeyCode::KeyA) && animation.request_animation(AnimationState::Backward) {
            velocity.x = -speed.0;           
        } else if keyboard_input.pressed(KeyCode::KeyD) && animation.request_animation(AnimationState::Forward) {
            velocity.x = speed.0;           
        } else {
            velocity.x = 0.;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            animation.request_animation(AnimationState::Crouch);
        }

        if keyboard_input.pressed(KeyCode::KeyQ) && animation.request_animation(AnimationState::Block) {
            commands.entity(entity).insert(IsBlocking);
        } else {
            commands.entity(entity).remove::<IsBlocking>();
        }

        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            animation.request_animation(AnimationState::LightAttack);
        }

        if keyboard_input.just_pressed(KeyCode::KeyX) {
            animation.request_animation(AnimationState::HeavyAttack);
        }

        let delta_time = time.delta_seconds();
        let gravity_coef = 15.;
        velocity.y -= gravity_coef * delta_time; 

        // Update position based on velocity
        transform.translation.x += velocity.x * delta_time * 100.;
        transform.translation.y += velocity.y * delta_time * 100.;

        // Ensure player doesn't fall through the ground
        if transform.translation.y < 0. {
            transform.translation.y = 0.;
            velocity.y = 0.;
        }
    }
}

use bevy::input::gamepad::{GamepadConnection, GamepadEvent};



fn gamepad_connections(
    mut commands: Commands,
    mut evr_gamepad: EventReader<GamepadEvent>,
    query: Query<(Entity, &InputController)>
) {
    for ev in evr_gamepad.read() {
        // we only care about connection events
        let GamepadEvent::Connection(ev_conn) = ev else {
            continue;
        };
        match &ev_conn.connection {
            GamepadConnection::Connected(info) => {
                debug!(
                    "New gamepad connected: {:?}, name: {}",
                    ev_conn.gamepad, info.name,
                );
                // if we don't have any gamepad yet, use this one
                commands.spawn(InputController(ev_conn.gamepad));
            }
            GamepadConnection::Disconnected => {
                debug!("Lost connection with gamepad: {:?}", ev_conn.gamepad);
                // if it's the one we previously used for the player, remove it:
                if let Some((e, _)) = query.iter().find(|(_, x)| x.0 == ev_conn.gamepad) {
                    commands.entity(e).despawn()
                }
            }
        }
    }
}
/*

fn gamepad_input(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
) {
    let Some(&MyGamepad(gamepad)) = my_gamepad.as_deref() else {
        // no gamepad is connected
        return;
    };

    // The joysticks are represented using a separate axis for X and Y
    let axis_lx = GamepadAxis {
        gamepad, axis_type: GamepadAxisType::LeftStickX
    };
    let axis_ly = GamepadAxis {
        gamepad, axis_type: GamepadAxisType::LeftStickY
    };

    if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
        // combine X and Y into one vector
        let left_stick = Vec2::new(x, y);

        // Example: check if the stick is pushed up
        if left_stick.length() > 0.9 && left_stick.y > 0.5 {
            // do something
        }
    }

    // In a real game, the buttons would be configurable, but here we hardcode them
    let jump_button = GamepadButton {
        gamepad, button_type: GamepadButtonType::South
    };
    let heal_button = GamepadButton {
        gamepad, button_type: GamepadButtonType::East
    };

    if buttons.just_pressed(jump_button) {
        // button just pressed: make the player jump
    }

    if buttons.pressed(heal_button) {
        // button being held down: heal the player
    }
}
*/
