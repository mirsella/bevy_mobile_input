use bevy::{
    color::palettes::tailwind,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

#[cfg(target_arch = "wasm32")]
mod web_input;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            canvas: Some("#bevy-canvas".into()),
            ..default()
        }),
        ..default()
    }));

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(web_input::MobileInputPlugin);

    app.add_systems(Startup, setup)
        .add_systems(Update, handle_keyboard)
        .run()
}

#[derive(Component)]
struct InputText;

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
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tap the input box to show keyboard"),
                TextFont::from_font_size(24.0),
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(50.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderColor::all(tailwind::GRAY_500),
                ))
                .observe(on_input_click)
                .with_child((InputText, Text::new(""), TextFont::from_font_size(20.0)));

            parent.spawn((
                Text::new("Press Enter to submit"),
                TextFont::from_font_size(16.0),
                TextColor(tailwind::GRAY_400.into()),
            ));

            parent.spawn((
                SubmittedText,
                Text::new(""),
                TextFont::from_font_size(20.0),
                TextColor(tailwind::GREEN_400.into()),
            ));
        });
}

fn on_input_click(
    _click: On<Pointer<Click>>,
    mut border_query: Query<&mut BorderColor, With<Node>>,
) {
    #[cfg(target_arch = "wasm32")]
    web_input::show_keyboard();

    for mut border in &mut border_query {
        *border = BorderColor::all(tailwind::BLUE_500);
    }
}

fn handle_keyboard(
    mut events: MessageReader<KeyboardInput>,
    mut input_query: Query<&mut Text, (With<InputText>, Without<SubmittedText>)>,
    mut submitted_query: Query<&mut Text, (With<SubmittedText>, Without<InputText>)>,
) {
    let Ok(mut input) = input_query.single_mut() else {
        return;
    };

    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Enter => {
                let submitted_text = std::mem::take(&mut input.0);
                if let Ok(mut submitted) = submitted_query.single_mut() {
                    submitted.0 = format!("Submitted: {}", submitted_text);
                }
            }
            Key::Backspace => {
                input.0.pop();
            }
            Key::Character(ch) => {
                input.0.push_str(ch);
            }
            _ => {}
        }
    }
}
