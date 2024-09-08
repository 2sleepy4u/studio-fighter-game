use bevy::{prelude::*, asset::LoadedFolder};

use crate::character::Character;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Setup,
    Ready,
    Loading,
    InGame,
    Pause,
    Win
}

#[derive(Resource, Default)]
pub struct CharacterFolder(pub Handle<LoadedFolder>);

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(CharacterFolder(asset_server.load_folder("characters")));
}

pub fn check_characters_assets(
    mut next_state: ResMut<NextState<GameState>>,
    sprite_folder: Res<CharacterFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&sprite_folder.0) {
            next_state.set(GameState::Ready);
        }
    }
}

pub fn create_characters(
    folder: &LoadedFolder,
    character_assets: Res<Assets<Character>>,
) -> Vec<Character> {
    let mut characters = Vec::new();
    // Build a texture atlas using the individual sprites
    for handle in folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Character>();
        let Some(character) = character_assets.get(id) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        characters.push(character.clone());
    }

    characters
}
