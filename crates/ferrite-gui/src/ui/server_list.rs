use bevy::prelude::*;

use crate::{DiscoveredServer, LanDiscoveryState, SelectedServer, ServerListUI, UiFont};

#[derive(Component)]
pub struct ServerEntryContainer;

#[derive(Component)]
pub struct LanServerButton(pub String);

#[derive(Component)]
pub struct JoinServerButton;

pub fn spawn_server_list(commands: &mut Commands, font: &Handle<Font>) {
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
        .insert(ServerListUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Multiplayer",
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

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .insert(ServerEntryContainer);

            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(20.0),
                    ..default()
                },
                ..default()
            });

            join_btn(parent, font);
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

pub fn update_server_list(
    ui: Query<Entity, With<ServerListUI>>,
    container_query: Query<Entity, With<ServerEntryContainer>>,
    lan: Res<LanDiscoveryState>,
    children_query: Query<&Children>,
    fonts: Res<UiFont>,
    mut commands: Commands,
    mut last_gen: Local<u64>,
) {
    if ui.is_empty() {
        return;
    }

    if lan.generation == *last_gen {
        return;
    }
    *last_gen = lan.generation;

    let Ok(container_entity) = container_query.get_single() else {
        return;
    };

    if let Ok(children) = children_query.get(container_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }

    if !lan.lan.is_available() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "LAN discovery unavailable",
                TextStyle {
                    font: fonts.0.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.8, 0.4, 0.4),
                },
            ));
        });
    } else if lan.servers.is_empty() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Scanning for LAN games...",
                TextStyle {
                    font: fonts.0.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                },
            ));
        });
    } else {
        for server in lan.servers.iter() {
            commands.entity(container_entity).with_children(|parent| {
                server_entry(parent, server, &fonts.0);
            });
        }
    }
}

pub fn update_server_list_highlight(
    selected: Res<SelectedServer>,
    server_entries: Query<(Entity, &LanServerButton, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (_entity, btn, children) in server_entries.iter() {
        let is_selected = selected.0.as_deref() == Some(btn.0.as_str());
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

pub fn update_join_button_visual(
    selected: Res<SelectedServer>,
    mut join_btn: Query<&mut BackgroundColor, With<JoinServerButton>>,
) {
    for mut color in join_btn.iter_mut() {
        if selected.0.is_some() {
            *color = Color::srgb(0.25, 0.55, 0.25).into();
        } else {
            *color = Color::srgb(0.15, 0.15, 0.15).into();
        }
    }
}

fn server_entry(parent: &mut ChildBuilder, server: &DiscoveredServer, font: &Handle<Font>) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(380.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                margin: UiRect::vertical(Val::Px(3.0)),
                ..default()
            },
            background_color: Color::srgb(0.25, 0.25, 0.25).into(),
            ..default()
        })
        .insert(LanServerButton(server.address.clone()))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "LAN World",
                TextStyle {
                    font: font.clone(),
                    font_size: 14.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(TextBundle::from_section(
                &server.motd,
                TextStyle {
                    font: font.clone(),
                    font_size: 12.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                },
            ));
            parent.spawn(TextBundle::from_section(
                &server.address,
                TextStyle {
                    font: font.clone(),
                    font_size: 11.0,
                    color: Color::srgb(0.5, 0.5, 0.5),
                },
            ));
        });
}

fn join_btn(parent: &mut ChildBuilder, font: &Handle<Font>) {
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
        .insert(JoinServerButton)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Join Server",
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
