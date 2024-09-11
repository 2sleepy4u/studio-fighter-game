use std::collections::HashMap;

use bevy::{prelude::*, window::{EnabledButtons, WindowResolution}, asset::LoadedFolder};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod characters;
use characters::*;

const MAX_WINDOW_HEIGHT: f32 = 300.;
const MAX_WINDOW_WIDTH: f32 = 300.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fight!".to_string(),
                    resolution: WindowResolution::new(MAX_WINDOW_HEIGHT, MAX_WINDOW_WIDTH),
                    resizable: true,
                    enabled_buttons: EnabledButtons {
                        minimize: false,
                        maximize: false,
                        close: true
                    },
                    ..default()
                }),
                ..default()
            })
            )
        .init_state::<GameState>()
        .add_plugins(WorldInspectorPlugin::new())
        //characters
        .add_plugins(RonAssetPlugin::<Character>::new(&["ron"]))
        .init_asset::<Character>()
        .init_resource::<CharacterHandle>()
        .add_systems(OnEnter(GameState::Setup), load_assets)
        .add_systems(Update, check_characters_assets.run_if(in_state(GameState::Setup)))

        .add_systems(OnEnter(GameState::Ready), (
                spawn_camera, 
                spawn_player
                )
            )
        .add_event::<HitEvent>()
        .add_systems(Update, (
                keyboard_input_system,
                execute_animations,
                execute_hitboxes,
                check_hitboxes,
                debug
            ).run_if(in_state(GameState::Ready))
        )
        .run();
}


fn debug(
    mut gizmos: Gizmos,
    query: Query<(&Hitbox, &Transform)>
) {
    for (hitbox, transform) in &query {
        let pos = Vec2::new(transform.translation.x, transform.translation.y) + Vec2::new(hitbox.x, hitbox.y);
        gizmos.rect_2d(pos, 0., Vec2::new(hitbox.length, hitbox.height), Color::RED);
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

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
pub struct CharacterHandle(pub Handle<Character>);

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

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {

    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = create_characters(&loaded_folder, loader);
    
    let character = &pgs.first().unwrap();
    let name = character.name.clone();
    let moveset = character.moveset.clone();
    let path = &character.sprite_sheet;
    
    let texture: Handle<Image> = asset_server.load(path);
    let layout = TextureAtlasLayout::from_grid(Vec2::splat(64.), 3, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animations: HashMap<AnimationState, (AnimationManager, Option<Attack>)> =
            HashMap::from([
                (AnimationState::Idle, (AnimationManager::new(character.idle.clone()), None)),
                (AnimationState::Forward, (AnimationManager::new(character.forward.clone()), None)),
                (AnimationState::Backward, (AnimationManager::new(character.backward.clone()), None)),
                (AnimationState::HeavyAttack, (AnimationManager::new(moveset.heavy.animation.clone()), Some(moveset.heavy.clone()))),
                (AnimationState::LightAttack, (AnimationManager::new(moveset.light.animation.clone()), Some(moveset.light.clone())))
            ]);
    commands.spawn(
        (PlayerAnimationManagement::new(animations),
         Speed(5.),
         Velocity::default(),
         Name::new(name),
         Player,
         InputController,
         TextureAtlas {
             layout: texture_atlas_layout.clone(),
             index: 0
         },
         SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture: texture.clone(),
            ..default()
        },
        )
    );
}

