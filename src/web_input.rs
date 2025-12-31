use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

static EVENT_QUEUE: Mutex<Vec<KeyboardInput>> = Mutex::new(Vec::new());

pub struct MobileInputPlugin;

impl Plugin for MobileInputPlugin {
    fn build(&self, app: &mut App) {
        setup_event_listeners();
        app.add_systems(First, poll_events);
    }
}

pub fn show_keyboard() {
    if let Some(input) = get_input_element() {
        let _ = input.focus();
    }
}

#[allow(dead_code)]
pub fn hide_keyboard() {
    if let Some(input) = get_input_element() {
        let _ = input.blur();
    }
}

fn get_input_element() -> Option<HtmlInputElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id("mobile-keyboard-input")?
        .dyn_into::<HtmlInputElement>()
        .ok()
}

fn setup_event_listeners() {
    let Some(input) = get_input_element() else {
        warn!("mobile-keyboard-input element not found");
        return;
    };

    let input_handler = Closure::wrap(Box::new(move |e: web_sys::InputEvent| {
        let Some(data) = e.data() else { return };
        for ch in data.chars() {
            if let Ok(mut queue) = EVENT_QUEUE.lock() {
                queue.push(KeyboardInput {
                    key_code: KeyCode::Unidentified(
                        bevy::input::keyboard::NativeKeyCode::Unidentified,
                    ),
                    logical_key: Key::Character(ch.to_string().into()),
                    state: ButtonState::Pressed,
                    repeat: false,
                    text: Some(ch.to_string().into()),
                    window: Entity::PLACEHOLDER,
                });
            }
        }
    }) as Box<dyn FnMut(_)>);

    let keydown_handler = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        let (key_code, logical_key) = match key.as_str() {
            "Enter" => (KeyCode::Enter, Key::Enter),
            "Backspace" => (KeyCode::Backspace, Key::Backspace),
            "Escape" => (KeyCode::Escape, Key::Escape),
            _ => return,
        };

        if let Ok(mut queue) = EVENT_QUEUE.lock() {
            queue.push(KeyboardInput {
                key_code,
                logical_key,
                state: ButtonState::Pressed,
                repeat: false,
                text: None,
                window: Entity::PLACEHOLDER,
            });
        }

        if key == "Enter" {
            if let Some(el) = get_input_element() {
                el.set_value("");
            }
        }
    }) as Box<dyn FnMut(_)>);

    let _ = input.add_event_listener_with_callback("input", input_handler.as_ref().unchecked_ref());
    let _ =
        input.add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref());

    input_handler.forget();
    keydown_handler.forget();
}

fn poll_events(mut keyboard_events: MessageWriter<KeyboardInput>) {
    let Ok(mut queue) = EVENT_QUEUE.lock() else {
        return;
    };
    for event in queue.drain(..) {
        keyboard_events.write(event);
    }
}
