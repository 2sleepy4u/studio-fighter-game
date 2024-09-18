use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::TypePath;

pub const MAX_FRAME_RATE: u8 = 60;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub sprite_sheet: String,
    pub speed: f32,
    pub health: u32,
    pub hurtbox: Hurtbox,
    pub moveset: Moveset,
    pub idle: Animation,
    pub block: Animation,
    pub jump: Animation,
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
    Block,
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
                if state != self.state {
                    self.state = state;
                    self.get_current_animation_mut().0.start_timer(MAX_FRAME_RATE);
                }
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
                self.index <= (options.active_frames + options.startup_frames + options.recovery_frames).into()
        }
        return false  
    }


    pub fn is_just_finished(&self) -> bool {
        self.timer.just_finished()
    }

    pub fn is_finished(&self) -> bool {
        self.timer.finished()
    }

    pub fn is_last_frame(&self) -> bool {
        self.animation.indexes.len() - 1 == self.index
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


#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Component)]
pub struct Hurtbox {
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
    pub fn new(value: u32) -> Self {
        Self(value)
    }

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

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct IsBlocking;

#[derive(Component)]
pub struct InputController(pub Gamepad);

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
