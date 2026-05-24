use bevy::prelude::*;

use crate::{LanDiscoveryState, ServerListUI, UiFont};

#[derive(Component)]
pub struct ServerEntryContainer;

#[derive(Component)]
pub struct LanServerButton(pub String);

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
                    height: Val::Px(30.0),
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

fn server_entry(parent: &mut ChildBuilder, server: &crate::DiscoveredServer, font: &Handle<Font>) {
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
