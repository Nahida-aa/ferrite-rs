use bevy::prelude::*;

use crate::{CoordText, HUDUI, PlayerInfoRes};

pub fn spawn_hud(commands: &mut Commands, _info: &PlayerInfoRes, font: &Handle<Font>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        })
        .insert(HUDUI)
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Entity: ---  Gamemode: ---",
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                ),
                CoordText,
            ));
        });
}

pub fn hud_update_system(
    mut query: Query<&mut Text, With<CoordText>>,
    info: Res<PlayerInfoRes>,
) {
    for mut text in query.iter_mut() {
        for section in text.sections.iter_mut() {
            let entity = info.entity_id.map(|e| format!("{e}")).unwrap_or_default();
            let mode = info.game_mode.map(|m| format!("{m}")).unwrap_or_default();
            section.value = format!("Entity: {entity}  Gamemode: {mode}");
        }
    }
}
