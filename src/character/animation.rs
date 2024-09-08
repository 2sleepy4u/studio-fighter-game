use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::collisions::Trigger;

use super::{Attack, Moveset};

#[derive(Component, Default)]
pub struct PlayerAnimationManagement {
    pub state: AnimationState,
    pub next_state: Option<AnimationState>,
    pub animations: HashMap<AnimationState, (AnimationDataTimer, Option<Attack>)>
}

impl PlayerAnimationManagement {
    pub fn new(moveset: Moveset) -> Self {
        let mut animations: HashMap<AnimationState, (AnimationDataTimer, Option<Attack>)> = HashMap::new();

        //to add to teh config file
        animations.insert(AnimationState::Idle, (AnimationDataTimer::new(AnimationData::new(vec![FrameData::new(1, 1), FrameData::new(2, 1), FrameData::new(2, 1)], AnimationConfig::default())), None)); 

        let light_data = moveset.light.animation.clone();
        let heavy_data = moveset.heavy.animation.clone();
        animations.insert(AnimationState::LightAttack, (AnimationDataTimer::new(light_data), Some(moveset.light))); 
        animations.insert(AnimationState::HeavyAttack, (AnimationDataTimer::new(heavy_data), Some(moveset.heavy))); 
        Self {
            animations,
            ..default()
        }
    }
    
    //it returns wweather the animation request was accepted or not
    pub fn request_animation(&mut self, state: AnimationState) -> bool {
        println!("Animation requested {:?}", state);
        match self.state {
            //if the current animation is an attack and is running...
            AnimationState::LightAttack | AnimationState::HeavyAttack if self.next_state.is_none() => {
                match state {
                    //...and i want to link an attack after it
                    AnimationState::LightAttack | AnimationState::HeavyAttack => {
                        //..and it falls withing the recovery frames
                        if self.get_current_animation().0.is_withing_recovery(0) {
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
            //if current state is not critical
            //therefore is not an attack
            //the prev animation get canceled and the new one become imediatly active
            _ => {
                self.state = state;
                return true;
            }
        }

        return false;
   }

    //start next animation buffered if any
    //else it just uses default (in this case is idle)
    pub fn next_state(&mut self) {
        if let Some(next) = &self.next_state {
            self.state = next.clone();
        } else {
            self.state = AnimationState::default();
        }
    }
    pub fn reset(&mut self) {
        self.get_current_animation_mut().0.reset();
    }
    pub fn get_current_animation_mut(&mut self) -> &mut (AnimationDataTimer, Option<Attack>) {
        let state = self.state.clone();
        let Some(animation) = self.animations.get_mut(&state) else {
            panic!("Animation not found!")
        };
        animation
    }
    pub fn get_current_animation(&self) -> &(AnimationDataTimer, Option<Attack>) {
        let state = self.state.clone();
        let Some(animation) = self.animations.get(&state) else {
            panic!("Animation not found!")
        };
        animation
    }
}

#[derive(Component)]
pub struct AnimationDataTimer {
    pub index: u8,
    pub animation_data: AnimationData,
    pub frame_timer: Timer
}

impl AnimationDataTimer {
   pub fn new(animation_data: AnimationData) -> Self {
       //first animation index in Atlas
       let index = animation_data.frame_data.get(0).unwrap().index;
       Self {
           animation_data,
           index,
           frame_timer: Timer::new(Duration::from_secs(0), TimerMode::Once)
       }
   } 

   pub fn reset(&mut self) {
       //first animation index in Atlas
       self.index = self.animation_data.frame_data.get(0).unwrap().index;
       self.frame_timer = Timer::new(Duration::from_secs(0), TimerMode::Once);
   } 

   pub fn is_withing_active(&self) -> bool {
       let current = self.get_current_frame_count().unwrap();
       self.get_config().is_withing_active(current)
   }

   pub fn is_withing_startup(&self, window: u8) -> bool {
       let current = self.get_current_frame_count().unwrap();
       self.get_config().is_withing_startup(current, window)
   }

   pub fn is_withing_recovery(&self, window: u8) -> bool {
       let config = self.get_config();
       let current = self.get_current_frame_count().unwrap();
       config.is_withing_recovery(current, window)
   }

   pub fn is_just_finished(&self) -> bool {
       self.frame_timer.just_finished()
   }

   pub fn get_config(&self) -> AnimationConfig {
       self.animation_data.config.clone()
   }

   pub fn get_current_frame_count(&self) -> Result<u8, String> {
       self.get_frame_from_index(self.index.into())
   }

   pub fn get_current_frame_data(&self) -> Option<FrameData> {
       let index = self.index as usize;
       self.animation_data.frame_data.get(index).cloned()
   }

   pub fn get_frame_from_index(&self, index: usize) -> Result<u8, String> {
       if self.animation_data.frame_data.len() <= index {
           return Err(format!("Out of index len: {} index: {}", self.animation_data.frame_data.len(), index));
       }

       Ok(self.animation_data.frame_data[0..index].iter().fold(0, |acc, x| x.frames + acc))
   }
}

pub fn execute_animations(
    time: Res<Time>,
    mut query: Query<(
        &mut PlayerAnimationManagement, &mut TextureAtlas
        )>,
) {
    for (mut player_animation, mut atlas) in &mut query {
        //scope to drop automatically anim so the mutable reference stop existing
        {
            //get current animation data
            let anim = &mut player_animation.get_current_animation_mut().0;
            // we track how long the current sprite has been displayed for
            anim.frame_timer.tick(time.delta());
        }


        let anim = &player_animation.get_current_animation().0;
        // If it has been displayed for the user-defined amount of time (fps)...
        if anim.frame_timer.just_finished() {
            if atlas.index == anim.animation_data.frame_data.last().unwrap().index as usize {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = anim.animation_data.frame_data.first().unwrap().index as usize;
                player_animation.next_state();
                player_animation.reset();
            } else {
                let anim = &mut player_animation.get_current_animation_mut().0;
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                anim.index += 1;
                // ...and reset the frame timer to start counting all over again
                anim.frame_timer = anim.get_current_frame_data().unwrap().get_timer();
            }
        }
    }
}

pub fn execute_hitboxes (
    mut commands: Commands,
    mut query: Query<(Entity,
        &mut PlayerAnimationManagement, &mut TextureAtlas
        )>,
) {
    for (entity, mut player_animation, mut atlas) in &mut query {
        let state = player_animation.state.clone();
        let anim = &player_animation.animations.get_mut(&state).unwrap().0;

        //if animation has finished remove from queue
        if anim.frame_timer.just_finished() {
            //shift next
        } else {
            //if is still runing then check for hitboxes
            if anim.is_withing_active() {
                //spawn hitbox
                if let Some(attack) = player_animation.get_current_animation().1.clone() {
                    //add attack to trigger or just check what type of attack active in other
                    //player?
                    commands.entity(entity).insert(Trigger {
                        x: 2.,
                        y: 2.,
                        height: 2.,
                        length: 2.
                    });
                }
            }
        }
    }
}


#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Default)]
pub struct AnimationConfig {
    pub startup_frames: u8,
    pub active_frames: u8,
    pub recovery_frames: u8
}

impl AnimationConfig {
    pub fn is_withing_active(&self, frames: u8) -> bool {
        frames > self.startup_frames && frames <= self.active_frames
    }


    pub fn is_withing_startup(&self, frames: u8, window: u8) -> bool {
        frames < self.startup_frames + window
    }

    pub fn is_withing_recovery(&self, frames: u8, window: u8) -> bool {
        frames > self.startup_frames + self.active_frames + window
            &&
        frames < self.startup_frames + self.active_frames + self.recovery_frames + window
    }
}

#[derive(Default, serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum AnimationState {
    #[default]
    Idle,
    Forward,
    Backward,
    Jump,
    LightAttack,
    HeavyAttack,
}


#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct FrameData {
    pub index: u8,
    pub frames: u8
}

impl FrameData {
    pub fn new(index: u8, frames: u8) -> Self {
        Self { index, frames }
    }
    pub fn get_timer(&self) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (self.frames as f32)), TimerMode::Once)
    }
}

#[derive(Component)]
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct AnimationData {
    pub frame_data: Vec<FrameData>,
    pub config: AnimationConfig
}

impl AnimationData {
    pub fn new(frame_data: Vec<FrameData>, config: AnimationConfig) -> Self {
        Self { frame_data, config }
    }
}

#[test]
fn test_recovery_frames() { }
