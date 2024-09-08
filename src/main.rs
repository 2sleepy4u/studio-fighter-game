use bevy::{asset::LoadedFolder, gizmos::aabb::AabbGizmoPlugin, prelude::*, window::{EnabledButtons, WindowResolution}};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod gamestate;
use gamestate::*;

mod character;
use character::{*, animation::*};

mod collisions;
use collisions::*;

const MAX_WINDOW_HEIGHT: f32 = 1920.;
const MAX_WINDOW_WIDTH: f32 = 1080.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fight!".to_string(),
                    resolution: WindowResolution::new(MAX_WINDOW_HEIGHT, MAX_WINDOW_WIDTH),
                    resizable: false,
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
        .add_systems(Update, (
                keyboard_input_system,
                execute_animations,
                execute_hitboxes,
                debug
            ).run_if(in_state(GameState::Ready))
        )
        .run();
}


fn debug(
    mut gizmos: Gizmos,
    query: Query<&Trigger>
) {
    for hitbox in &query {
        gizmos.rect_2d(Vec2::new(hitbox.x, hitbox.y), 0., Vec2::new(hitbox.length, hitbox.height), Color::RED);
    }

}


#[derive(Component)]
pub struct Player;



#[derive(Component)]
pub struct InputController;

pub fn spawn_camera(
    mut commands: Commands,
  ) {
    commands.spawn(Camera2dBundle::default());
}

pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut Transform, &mut PlayerAnimationManagement        ), With<InputController>>,
) {
    //when pressing a button
    //it needs to ask/check if move is legal
    //it is legal if no move is active 
    //or if it falls withing cancel and linking rules
    for (mut transform, mut animation) in query.iter_mut() {
        let mut movement = transform.translation;
        if keyboard_input.pressed(KeyCode::KeyW) {
            //jump
        }

        if keyboard_input.pressed(KeyCode::KeyA) && animation.request_animation(AnimationState::Backward) {
            movement.x -= 1.;           
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            //crouch
        }

        if keyboard_input.pressed(KeyCode::KeyD) && animation.request_animation(AnimationState::Forward) {
            movement.x += 1.;           
        }

        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            animation.request_animation(AnimationState::LightAttack);
        }

        if keyboard_input.just_pressed(KeyCode::KeyX) {
            animation.request_animation(AnimationState::HeavyAttack);
        }

        transform.translation = movement;
    }
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


    commands.spawn(
        (PlayerAnimationManagement::new(moveset),
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
