use bevy::prelude::*;

use super::{MainMenuUI, UiRes, WorldSelectUI};
use crate::worlds::WorldManager;

pub(super) fn spawn_menu(commands: &mut Commands, ui: &UiRes, font: &Handle<Font>) {
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
                    font: font.clone(),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(30.0),
                    ..default()
                },
                ..default()
            });

            btn(parent, "Single Player", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Demo World", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Multi Player", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Quit", font);

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
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::srgb(1.0, 0.0, 0.0),
                    },
                ));
            }
        });
}

pub(super) fn spawn_world_select(commands: &mut Commands, worlds: &WorldManager, font: &Handle<Font>) {
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
        .insert(WorldSelectUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Select World",
                TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(20.0),
                    ..default()
                },
                ..default()
            });

            if worlds.worlds.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "No worlds found",
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                    },
                ));
                parent.spawn(NodeBundle {
                    style: Style {
                        height: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                });
            } else {
                for world in &worlds.worlds {
                    btn(parent, &world.name, font);
                    parent.spawn(NodeBundle {
                        style: Style {
                            height: Val::Px(6.0),
                            ..default()
                        },
                        ..default()
                    });
                }
            }

            btn(parent, "Create New World", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Back", font);
        });
}

fn btn(parent: &mut ChildBuilder, label: &str, font: &Handle<Font>) {
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
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
        });
}
