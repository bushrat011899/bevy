//! Simple example demonstrating overflow behavior.

use bevy::{color::palettes::css::*, prelude::*, winit::WinitSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, update_outlines)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let text_style = TextFont::default();

    let image = asset_server.load("branding/icon.png");

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Percent(100.),
                height: Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: ANTIQUE_WHITE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            for overflow in [
                Overflow::visible(),
                Overflow::clip_x(),
                Overflow::clip_y(),
                Overflow::clip(),
            ] {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: UiRect::horizontal(Px(25.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        let label = format!("{overflow:#?}");
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    padding: UiRect::all(Px(10.)),
                                    margin: UiRect::bottom(Px(25.)),
                                    ..Default::default()
                                },
                                background_color: Color::srgb(0.25, 0.25, 0.25).into(),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn((Text::new(label), text_style.clone()));
                            });
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Px(100.),
                                    height: Px(100.),
                                    padding: UiRect {
                                        left: Px(25.),
                                        top: Px(25.),
                                        ..Default::default()
                                    },
                                    border: UiRect::all(Px(5.)),
                                    overflow,
                                    ..Default::default()
                                },
                                border_color: Color::BLACK.into(),
                                background_color: GRAY.into(),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    ImageBundle {
                                        image: UiImage::new(image.clone()),
                                        style: Style {
                                            min_width: Px(100.),
                                            min_height: Px(100.),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    Interaction::default(),
                                    Outline {
                                        width: Px(2.),
                                        offset: Px(2.),
                                        color: Color::NONE,
                                    },
                                ));
                            });
                    });
            }
        });
}

fn update_outlines(mut outlines_query: Query<(&mut Outline, Ref<Interaction>)>) {
    for (mut outline, interaction) in outlines_query.iter_mut() {
        if interaction.is_changed() {
            outline.color = match *interaction {
                Interaction::Pressed => RED.into(),
                Interaction::Hovered => WHITE.into(),
                Interaction::None => Color::NONE,
            };
        }
    }
}
