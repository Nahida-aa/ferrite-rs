use bevy::prelude::*;

use crate::player::{PlayerInfoRes, PlayerRes};

use super::{CoordText, HUDUI};

pub(super) fn spawn_hud(commands: &mut Commands, info: &PlayerInfoRes, font: &Handle<Font>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .insert(HUDUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ferrite",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(TextBundle::from_section(
                "  |  ",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.5, 0.5, 0.5),
                },
            ));
            parent
                .spawn(TextBundle::from_section(
                    "XYZ: 0.0 / 0.0 / 0.0",
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ))
                .insert(CoordText);
            parent
                .spawn(TextBundle::from_section(
                    format!(
                        "  |  E:{} GM:{}",
                        info.entity_id.unwrap_or(0),
                        match info.game_mode {
                            Some(0) => "S",
                            Some(1) => "C",
                            Some(2) => "A",
                            Some(3) => "Sp",
                            _ => "?",
                        }
                    ),
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.0,
                        color: Color::srgb(0.7, 0.7, 0.7),
                    },
                ))
                .insert(super::InfoText);
            parent.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    margin: UiRect::right(Val::Px(6.0)),
                    ..default()
                },
                background_color: Color::srgb(0.0, 1.0, 0.0).into(),
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "Connected",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.0, 1.0, 0.0),
                },
            ));
        });
}

pub(super) fn hud_update_system(
    player: Res<PlayerRes>,
    mut query: Query<&mut Text, With<CoordText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        if let Some((x, y, z)) = player.position {
            text.sections[0].value = format!("XYZ: {:.1} / {:.1} / {:.1}", x, y, z);
        } else {
            text.sections[0].value = "XYZ: ---".to_string();
        }
    }
}
