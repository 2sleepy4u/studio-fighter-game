use std::collections::HashMap;
use std::time::Duration;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use bevy::reflect::TypePath;

pub const MAX_FRAME_RATE: u8 = 60;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub sprite_sheet: String,
    pub moveset: Moveset,
    pub idle: Animation,
    pub forward: Animation,
    pub backward: Animation,
}


#[derive(serde::Deserialize, Asset, TypePath, Component, Debug, Clone)]
pub struct Moveset {
    pub light: Attack,
    pub heavy: Attack 
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Attack {
    pub damage: u32,
    pub hit_stun_frames: u8,
    pub animation: Animation,
    pub hitbox: Hitbox,
}

//Animations ----
#[derive(Default, serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum AnimationState {
    #[default]
    Idle,
    Forward,
    Backward,
    Jump,
    Crouch,
    LightAttack,
    HeavyAttack,
}

#[derive(Component, Default)]
pub struct PlayerAnimationManagement {
    pub state: AnimationState,
    pub next_state: Option<AnimationState>,
    pub animations: HashMap<AnimationState, (AnimationManager, Option<Attack>)>
}

impl PlayerAnimationManagement {
    pub fn new(animations: HashMap<AnimationState, (AnimationManager, Option<Attack>)>) -> Self { 
        Self { animations, ..default() }
    }

    pub fn request_animation(&mut self, state: AnimationState) -> bool {
        match self.state {
            //if the current animation is an attack and is running...
            AnimationState::LightAttack | AnimationState::HeavyAttack if self.next_state.is_none() => {
                match state {
                    //...and i want to link an attack after it
                    AnimationState::LightAttack | AnimationState::HeavyAttack => {
                        //..and it falls within the recovery frames
                        let (animation_manager, _) = self.get_current_animation();
                        if animation_manager.is_within_recovery() {
                            //..it gets buffered
                            self.next_state = Some(state);
                            return true
                        }
                    },
                    //..and is something with low priority
                    //like movement or idle (jump, crouch,..)
                    //then it just ignores the move and the animation
                    _ => { } 
                }
            },

            AnimationState::LightAttack | AnimationState::HeavyAttack if self.next_state.is_some() => { },
            //if current state is not critical
            //therefore is not an attack
            //the prev animation get canceled and the new one become imediatly active
            _ => {
                self.state = state;
                self.get_current_animation_mut().0.start_timer(MAX_FRAME_RATE);
                return true;
            }
        }

        return false;
   }

    //start next animation buffered if any
    //else it just uses default (in this case is idle)
    pub fn shift(&mut self) {
        if let Some(next) = &self.next_state {
            self.state = next.clone();
            self.next_state = None;
        } else {
            self.state = AnimationState::default();
        }
        let (animation, _) = &mut self.get_current_animation_mut();
        animation.reset(MAX_FRAME_RATE);
        animation.start_timer(MAX_FRAME_RATE);

    }

    pub fn get_current_animation(&self) -> &(AnimationManager, Option<Attack>) {
        let state = self.state.clone();
        let Some(animation) = self.animations.get(&state) else {
            panic!("Animation not found!")
        };
        animation
    }

    pub fn get_current_animation_mut(&mut self) -> &mut (AnimationManager, Option<Attack>) {
        let state = self.state.clone();
        let Some(animation) = self.animations.get_mut(&state) else {
            panic!("Animation not found!")
        };
        animation
    }
 
}



#[derive(Debug, Clone)]
pub struct AnimationManager {
    index: usize,
    animation: Animation,
    timer: Timer
}

impl AnimationManager {
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            index: 0,
            timer: Timer::from_seconds(0., TimerMode::Once)
        }
    }

    pub fn reset(&mut self, max_fps: u8) {
        self.index = 0;
        self.timer = Timer::from_seconds(
            self.animation.fps as f32 / max_fps as f32, 
            TimerMode::Once
        )
    }



    pub fn start_timer(&mut self, max_fps: u8) {
        self.timer = Timer::from_seconds(
            self.animation.fps as f32 / max_fps as f32, 
            TimerMode::Once
        )
    }


    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }


    pub fn is_within_active(&self) -> bool {
        if let Some(options) = &self.animation.options {
            return 
                self.index > options.startup_frames.into()
                &&
                self.index <= (options.active_frames + options.startup_frames).into()
        }
        return false
    }

    pub fn is_within_startup(&self) -> bool {
        if let Some(options) = &self.animation.options {
            return self.index < options.startup_frames.into()
        }
        return false   
    }

    pub fn is_within_recovery(&self) -> bool {
        if let Some(options) = &self.animation.options {
            return 
                self.index > (options.active_frames + options.startup_frames).into()
                &&
                self.index <= options.recovery_frames.into()
        }
        return false  
    }


    pub fn is_just_finished(&self) -> bool {
        self.timer.just_finished()
    }

    pub fn is_finished(&self) -> bool {
        self.timer.finished()
    }

    pub fn next_frame(&mut self, max_fps: u8) {
        if self.animation.indexes.len() > self.index + 1 {
            self.index += 1;
            self.timer = Timer::from_seconds(
                self.animation.fps as f32 / max_fps as f32, 
                TimerMode::Once
            )
        } 
    }

    pub fn current_frame_index(&self) -> u8 {
        self.index as u8
    }

    
    pub fn last_sprite_index(&self) -> Option<u8> {
        self.animation.indexes.last().cloned()
    }

    pub fn current_sprite_index(&self) -> Option<u8> {
        self.animation.indexes.get(self.index).cloned()
    }
}


#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Animation {
    pub indexes: Vec<u8>,
    pub fps: u8,
    pub options: Option<AnimationOptions>
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct AnimationOptions {
    pub startup_frames: u8,
    pub active_frames: u8,
    pub recovery_frames: u8
}


#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Component)]
pub struct Hitbox {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub height: f32
}

#[derive(Event, Debug)]
pub struct HitEvent {
    pub target: Entity,
    pub source: Entity,
}

pub fn check_hits (
    mut target: Query<(Entity, &Hitbox, &GlobalTransform)>,
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


//systems



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
            if atlas.index == animation_manager.last_sprite_index().unwrap() as usize {
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

#[derive(Component, Clone, Debug)]
pub struct HitStun(Timer);
impl HitStun {
   pub fn new(fps: u8, max_fps: u8) -> Self {
        Self(Timer::from_seconds( fps as f32 / max_fps as f32, TimerMode::Once))
   } 

   pub fn is_finished(&self) -> bool {
       self.0.finished()
   }
}

#[derive(Component, Clone, Debug)]
pub struct Health(u32);

impl Health {
    pub fn heal(&mut self, value: u32, max_health: u32) {
        if self.0 + value > max_health {
            self.0 = max_health;
        } else {
            self.0 += value;
        }
    }

    pub fn damage(&mut self, value: u32) {
        if self.0 - value < 0 {
            self.0 = 0;
        } else {
            self.0 -= value;
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


#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Speed(pub f32);



#[derive(Component)]
pub struct InputController;

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut Transform, &mut PlayerAnimationManagement, &Speed,
        &mut Velocity,
        ), (With<InputController>, Without<HitStun>)>,
) {
    //when pressing a button
    //it needs to ask/check if move is legal
    //it is legal if no move is active 
    //or if it falls withing cancel and linking rules
    for (mut transform, mut animation, speed, mut velocity) in query.iter_mut() {
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

        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            animation.request_animation(AnimationState::LightAttack);
        }

        if keyboard_input.just_pressed(KeyCode::KeyX) {
            animation.request_animation(AnimationState::HeavyAttack);
        }

        let delta_time = time.delta_seconds();
        velocity.y -= 15. * delta_time; // Adjust gravity value as needed

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

/// Simple resource to store the ID of the first connected gamepad.
/// We can use it to know which gamepad to use for player input.
#[derive(Resource)]
struct MyGamepad(Gamepad);

fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<MyGamepad>>,
    mut evr_gamepad: EventReader<GamepadEvent>,
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
                if my_gamepad.is_none() {
                    commands.insert_resource(MyGamepad(ev_conn.gamepad));
                }
            }
            GamepadConnection::Disconnected => {
                debug!("Lost connection with gamepad: {:?}", ev_conn.gamepad);
                // if it's the one we previously used for the player, remove it:
                if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                    if *old_id == ev_conn.gamepad {
                        commands.remove_resource::<MyGamepad>();
                    }
                }
            }
        }
    }
}


fn gamepad_input(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    my_gamepad: Option<Res<MyGamepad>>,
    mut query: Query<(&mut Transform, &mut PlayerAnimationManagement, &Speed), (With<InputController>, Without<HitStun>)>,
) {
    for (mut transform, mut animation, speed) in query.iter_mut() {
        let mut movement = transform.translation;
        let Some(&MyGamepad(gamepad)) = my_gamepad.as_deref() else {
            // no gamepad is connected
            return;
        };

        // The joysticks are represented using a separate axis for X and Y
        let axis_lx = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickX
        };
        let _axis_ly = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickY
        };

        if let Some(x) = axes.get(axis_lx) {
            if x > 0. && animation.request_animation(AnimationState::Forward) {
                movement.x += 1. * speed.0;
            } else if x < 0. && animation.request_animation(AnimationState::Backward) {
                movement.x -= 1. * speed.0;
            }
        }

        // In a real game, the buttons would be configurable, but here we hardcode them
        let light_attack = GamepadButton {
            gamepad, button_type: GamepadButtonType::North
        };
        let heavy_attack = GamepadButton {
            gamepad, button_type: GamepadButtonType::East
        };

        if buttons.just_pressed(light_attack) {
            animation.request_animation(AnimationState::LightAttack);
        }

        if buttons.pressed(heavy_attack) {
            animation.request_animation(AnimationState::HeavyAttack);
        }

        transform.translation = movement;
    }
}
