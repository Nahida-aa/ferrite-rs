use bevy::prelude::*;

use super::{MainMenuUI, UiRes};

pub(super) fn spawn_menu(commands: &mut Commands, ui: &UiRes) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(MainMenuUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ferrite",
                TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(30.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Single Player");
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Multi Player");
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Quit");

            if let Some(ref err) = ui.last_error {
                parent.spawn(NodeBundle {
                    style: Style {
                        height: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });
                parent.spawn(TextBundle::from_section(
                    format!("Error: {err}"),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(1.0, 0.0, 0.0),
                        ..default()
                    },
                ));
            }
        });
}

fn btn(parent: &mut ChildBuilder, label: &str) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgb(0.3, 0.3, 0.3).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}
