use bevy::prelude::*;

use crate::worlds::{SelectedWorld, WorldManager};
use crate::{
    MainMenuUI, PlayWorldButton, UiRes, WorldEntryButton, WorldSelectUI,
};

pub fn spawn_menu(commands: &mut Commands, ui: &UiRes, font: &Handle<Font>) {
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

pub fn spawn_world_select(commands: &mut Commands, worlds: &WorldManager, font: &Handle<Font>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::top(Val::Px(40.0)),
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
                    world_entry(parent, world, font);
                    parent.spawn(NodeBundle {
                        style: Style {
                            height: Val::Px(6.0),
                            ..default()
                        },
                        ..default()
                    });
                }
            }

            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Create New World", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(6.0),
                    ..default()
                },
                ..default()
            });
            play_btn(parent, font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(6.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Back", font);
        });
}

pub fn update_world_select_highlight(
    selected: Res<SelectedWorld>,
    world_entries: Query<(Entity, &WorldEntryButton, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let selected_name: Option<&str> = selected.0.as_ref().and_then(|p| {
        std::path::Path::new(p).file_name().and_then(|n| n.to_str())
    });

    for (_entity, entry, children) in world_entries.iter() {
        let is_selected = selected_name == Some(entry.0.as_str());
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                for section in text.sections.iter_mut() {
                    if is_selected {
                        section.style.color = Color::srgb(1.0, 1.0, 0.6);
                    } else {
                        section.style.color = Color::WHITE;
                    }
                }
            }
        }
    }
}

pub fn update_play_button_visual(
    selected: Res<SelectedWorld>,
    mut play_btn: Query<&mut BackgroundColor, (With<PlayWorldButton>, Without<WorldEntryButton>)>,
) {
    for mut color in play_btn.iter_mut() {
        if selected.0.is_some() {
            *color = Color::srgb(0.25, 0.55, 0.25).into();
        } else {
            *color = Color::srgb(0.15, 0.15, 0.15).into();
        }
    }
}

fn world_entry(parent: &mut ChildBuilder, world: &crate::worlds::WorldEntry, font: &Handle<Font>) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(380.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                ..default()
            },
            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
            ..default()
        })
        .insert(WorldEntryButton(world.name.clone()))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                &world.name,
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(TextBundle::from_section(
                world.path.to_string_lossy().to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 11.0,
                    color: Color::srgb(0.5, 0.5, 0.5),
                },
            ));
        });
}

fn play_btn(parent: &mut ChildBuilder, font: &Handle<Font>) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgb(0.15, 0.15, 0.15).into(),
            ..default()
        })
        .insert(PlayWorldButton)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play World",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
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
