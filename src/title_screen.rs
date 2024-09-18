use bevy::prelude::*;

use crate::{GameState, spawn_camera};
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TitleScreen), (
                spawn_camera, 
                title_screen_ui,
                title_screen_music
            )
        )
        .add_systems(Update, title_screen_buttons.run_if(in_state(GameState::TitleScreen)))
        .add_systems(OnExit(GameState::TitleScreen), (
                despawn_all_with::<Text>,
                despawn_all_with::<Camera2d>,
                despawn_all_with::<IntroSound>,
            )
        );
    }
}

#[derive(Component)]
struct IntroSound;

fn title_screen_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        IntroSound,
        AudioBundle {
            source: asset_server.load("sounds/Intro.mp3"),
            ..default()
        },
    ));
}

fn title_screen_ui(
    mut commands: Commands
) {
    commands.spawn((
        TextBundle::from_section("Super Studio Fighter X", 
                                 TextStyle { font_size: 50., ..default() }
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            align_content: AlignContent::Center,
            ..default()
        })
    ));

}

fn despawn_all_with<C: Component>(
    query: Query<Entity, With<C>>,
    mut commands: Commands,
)
{
    query.iter().for_each(|x| commands.entity(x).despawn());

}

fn title_screen_buttons(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) { 
        next_state.set(GameState::CharacterSelection)
    }
}
