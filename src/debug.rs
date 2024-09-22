use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::components::{Hitbox, Hurtbox};


fn in_debug(debug: bool) -> impl Condition<()> {
    IntoSystem::into_system(move || { debug })
}

#[derive(Default)]
pub struct DebugPlugin {
    pub inspector: bool,
    pub hitbox: bool
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                    debug_hitbox,
                    debug_hurtbox,
                ).run_if(in_debug(self.hitbox))
           );
        if self.inspector {
            app.add_plugins(WorldInspectorPlugin::new());
        }
    }
}

fn debug_hitbox(
    mut gizmos: Gizmos,
    query: Query<(&Hitbox, &Transform)>
) {
    for (hitbox, transform) in &query {
        let pos = Vec2::new(transform.translation.x, transform.translation.y) + Vec2::new(hitbox.x, hitbox.y);
        gizmos.rect_2d(pos, 0., Vec2::new(hitbox.length, hitbox.height), Color::RED);
    }
}

fn debug_hurtbox(
    mut gizmos: Gizmos,
    query: Query<(&Hurtbox, &Transform)>
) {
    for (hitbox, transform) in &query {
        let pos = Vec2::new(transform.translation.x, transform.translation.y) + Vec2::new(hitbox.x, hitbox.y);
        gizmos.rect_2d(pos, 0., Vec2::new(hitbox.length, hitbox.height), Color::GREEN);
    }
}
