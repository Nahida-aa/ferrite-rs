use bevy::prelude::*;

use crate::PauseMenuUI;

pub fn spawn_pause_menu(commands: &mut Commands, font: &Handle<Font>) {
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
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .insert(PauseMenuUI)
        .with_children(|parent| {
            btn(parent, "Back to Game", font);
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            btn(parent, "Disconnect", font);
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
