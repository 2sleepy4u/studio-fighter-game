use bevy::prelude::*;
use bevy::reflect::TypePath;


pub mod animation;
use super::animation::*;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub sprite_sheet: String,
    pub moveset: Moveset
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Component)]
struct Hitbox {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub height: f32
}



#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Attack {
    pub state: AnimationState,
    pub damage: u32,
    pub hit_stun_frames: u32,
    pub animation: AnimationData,
    pub hitbox: Hitbox,
}

#[derive(serde::Deserialize, Asset, TypePath, Component, Debug, Clone)]
pub struct Moveset {
    pub light: Attack,
    pub heavy: Attack 
}

///Handle used to load asset from file using bevy system
#[derive(Resource, Default)]
pub struct CharacterHandle(pub Handle<Character>);



#[derive(Component, Default)]
pub struct AnimationLink(Option<Attack>, Option<Attack>);

impl AnimationLink {
    pub fn try_link(&mut self, attack: Attack) {
        if self.0.is_none() {
            self.0 = Some(attack);
        } else if self.1.is_none() {
            self.1 = Some(attack);
        }
    }

    pub fn shift(&mut self) {
        self.0 = self.1.clone();
        self.1 = None;
    }

    pub fn get(&self) -> Option<Attack> {
        self.0.clone()
    }

    pub fn consume(&mut self) -> Option<Attack> {
        let result = self.get();
        self.shift();
        result
    }
}




