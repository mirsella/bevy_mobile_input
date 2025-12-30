use bevy::{color::palettes::tailwind, prelude::*};

mod web_input;
use web_input::{WebInputPlugin, WebInputState, WebTextInput, WebTextSubmit};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                canvas: Some("#bevy-canvas".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WebInputPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_click, handle_input, handle_submit))
        .run()
}

#[derive(Component)]
struct TextInputDisplay;

#[derive(Component)]
struct InputBox;

#[derive(Component)]
struct SubmittedText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            padding: UiRect::horizontal(Val::Px(16.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tap the input box below to type"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    InputBox,
                    Node {
                        width: Val::Percent(90.0),
                        max_width: Val::Px(400.0),
                        height: Val::Px(50.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderColor::all(tailwind::GRAY_500),
                    Interaction::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextInputDisplay,
                        Text::new(""),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn((
                Text::new("Press Enter to submit"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(tailwind::GRAY_400.into()),
            ));

            parent.spawn((
                SubmittedText,
                Text::new(""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(tailwind::GREEN_400.into()),
            ));
        });
}

fn handle_click(
    query: Query<&Interaction, (Changed<Interaction>, With<InputBox>)>,
    mut border_query: Query<&mut BorderColor, With<InputBox>>,
    mut state: ResMut<WebInputState>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            state.focused = true;
            web_input::focus_input();
            for mut border in &mut border_query {
                *border = BorderColor::all(tailwind::BLUE_500);
            }
        }
    }
}

fn handle_input(
    mut events: MessageReader<WebTextInput>,
    mut query: Query<&mut Text, With<TextInputDisplay>>,
) {
    for WebTextInput(text) in events.read() {
        for mut display in &mut query {
            display.0 = text.clone();
        }
    }
}

fn handle_submit(
    mut events: MessageReader<WebTextSubmit>,
    mut display_query: Query<&mut Text, (With<TextInputDisplay>, Without<SubmittedText>)>,
    mut submitted_query: Query<&mut Text, (With<SubmittedText>, Without<TextInputDisplay>)>,
) {
    for WebTextSubmit(text) in events.read() {
        for mut display in &mut display_query {
            display.0.clear();
        }
        for mut submitted in &mut submitted_query {
            submitted.0 = format!("Submitted: {}", text);
        }
    }
}
