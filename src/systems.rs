use bevy::prelude::*;
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};

use crate::components::*;

pub fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut PlayerAnimationManagement, &mut TextureAtlas)>,
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





pub fn gamepad_connections(
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

#[derive(Component)]
pub struct HealthBar(pub Gamepad);

pub fn health_bar(
    mut commands: Commands,
    query: Query<(&Name, &InputController, &Health), With<Player>>
) {
    let player_number = query.iter().count();
    let division = 100 / player_number;
    for (name, input_controller, health) in &query {
        commands.spawn(NodeBundle {
            style: Style {
                justify_self: JustifySelf::Start,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                HealthBar(input_controller.0),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(division as f32),
                        height: Val::Px(100.0),
                        position_type: PositionType::Relative,
                        top: Val::Px(10.),
                        left: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                }
                )
            );
            parent.spawn(
                TextBundle::from_section(name, TextStyle::default())
            );
        });
    }
}
