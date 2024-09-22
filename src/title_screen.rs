use bevy::prelude::*;

use crate::{GameState, spawn_camera};
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
            .add_systems(OnEnter(GameState::TitleScreen), (
                spawn_camera, 
                title_screen_ui,
                title_screen_music,
            )
        )
        .add_systems(Update, gamepad_input.run_if(in_state(GameState::TitleScreen)))
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
            source: asset_server.load("sounds/SUPA_STUDIO_FIGHTER_X_2.ogg"),
            ..default()
        },
    ));
}

fn title_screen_ui(
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn((
        TextBundle::from_section("Supa Studio Fighter X", 
                                 TextStyle { 
                                     font_size: 150., 
                                     font: asset_server.load("fonts/Act_Of_Rejection.ttf"),
                                     ..default() 
                                 }
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
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


fn gamepad_input(
    mut commands: Commands,
    gamepads: Res<Gamepads>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<GamepadButton>>,
) {
    for gamepad in gamepads.iter() {
        let valid_keys = [
            GamepadButton { gamepad, button_type: GamepadButtonType::East },
            GamepadButton { gamepad, button_type: GamepadButtonType::West },
            GamepadButton { gamepad, button_type: GamepadButtonType::South },
            GamepadButton { gamepad, button_type: GamepadButtonType::North },
        ];
        if buttons.any_just_pressed(valid_keys) {
            commands.remove_resource::<ClearColor>();
            next_state.set(GameState::CharacterSelection)
        }
    }
}


