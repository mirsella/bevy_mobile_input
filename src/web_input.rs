use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct WebTextInput(pub String);

#[derive(Message, Debug, Clone)]
pub struct WebTextSubmit(pub String);

#[derive(Message, Debug, Clone)]
pub struct WebImeComposition {
    pub text: String,
    pub is_composing: bool,
}

#[derive(Resource, Default)]
pub struct WebInputState {
    pub focused: bool,
    pub current_value: String,
}

pub struct WebInputPlugin;

impl Plugin for WebInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WebTextInput>()
            .add_message::<WebTextSubmit>()
            .add_message::<WebImeComposition>()
            .init_resource::<WebInputState>();

        #[cfg(target_arch = "wasm32")]
        {
            platform::setup_web_input();
            app.add_systems(Update, platform::poll_web_input);
        }
    }
}

pub fn focus_input() {
    #[cfg(target_arch = "wasm32")]
    platform::focus_input();
}

pub fn blur_input() {
    #[cfg(target_arch = "wasm32")]
    platform::blur_input();
}

pub fn set_input_value(value: &str) {
    #[cfg(target_arch = "wasm32")]
    platform::set_input_value(value);
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::*;
    use std::sync::Mutex;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlInputElement;

    #[derive(Debug, Clone)]
    enum WebEvent {
        Input(String),
        Submit(String),
        Composition { text: String, is_composing: bool },
    }

    static EVENT_QUEUE: Mutex<Vec<WebEvent>> = Mutex::new(Vec::new());

    fn get_input_element() -> Option<HtmlInputElement> {
        web_sys::window()?
            .document()?
            .get_element_by_id("bevy-hidden-input")?
            .dyn_into::<HtmlInputElement>()
            .ok()
    }

    pub fn setup_web_input() {
        let Some(input) = get_input_element() else {
            warn!("bevy-hidden-input element not found in DOM");
            return;
        };

        let input_handler = Closure::wrap(Box::new(move |_: web_sys::InputEvent| {
            if let Some(el) = get_input_element() {
                let value = el.value();
                if let Ok(mut queue) = EVENT_QUEUE.lock() {
                    queue.push(WebEvent::Input(value));
                }
            }
        }) as Box<dyn FnMut(_)>);

        let keydown_handler = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" {
                if let Some(el) = get_input_element() {
                    let value = el.value();
                    if let Ok(mut queue) = EVENT_QUEUE.lock() {
                        queue.push(WebEvent::Submit(value));
                    }
                    el.set_value("");
                }
            }
        }) as Box<dyn FnMut(_)>);

        let composition_start = Closure::wrap(Box::new(move |e: web_sys::CompositionEvent| {
            let text = e.data().unwrap_or_default();
            if let Ok(mut queue) = EVENT_QUEUE.lock() {
                queue.push(WebEvent::Composition {
                    text,
                    is_composing: true,
                });
            }
        }) as Box<dyn FnMut(_)>);

        let composition_update = Closure::wrap(Box::new(move |e: web_sys::CompositionEvent| {
            let text = e.data().unwrap_or_default();
            if let Ok(mut queue) = EVENT_QUEUE.lock() {
                queue.push(WebEvent::Composition {
                    text,
                    is_composing: true,
                });
            }
        }) as Box<dyn FnMut(_)>);

        let composition_end = Closure::wrap(Box::new(move |e: web_sys::CompositionEvent| {
            let text = e.data().unwrap_or_default();
            if let Ok(mut queue) = EVENT_QUEUE.lock() {
                queue.push(WebEvent::Composition {
                    text,
                    is_composing: false,
                });
            }
        }) as Box<dyn FnMut(_)>);

        let _ =
            input.add_event_listener_with_callback("input", input_handler.as_ref().unchecked_ref());
        let _ = input
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref());
        let _ = input.add_event_listener_with_callback(
            "compositionstart",
            composition_start.as_ref().unchecked_ref(),
        );
        let _ = input.add_event_listener_with_callback(
            "compositionupdate",
            composition_update.as_ref().unchecked_ref(),
        );
        let _ = input.add_event_listener_with_callback(
            "compositionend",
            composition_end.as_ref().unchecked_ref(),
        );

        input_handler.forget();
        keydown_handler.forget();
        composition_start.forget();
        composition_update.forget();
        composition_end.forget();
    }

    pub fn focus_input() {
        if let Some(input) = get_input_element() {
            let _ = input.focus();
        }
    }

    pub fn blur_input() {
        if let Some(input) = get_input_element() {
            let _ = input.blur();
        }
    }

    pub fn set_input_value(value: &str) {
        if let Some(input) = get_input_element() {
            input.set_value(value);
        }
    }

    pub fn poll_web_input(
        mut input_events: MessageWriter<WebTextInput>,
        mut submit_events: MessageWriter<WebTextSubmit>,
        mut composition_events: MessageWriter<WebImeComposition>,
        mut state: ResMut<WebInputState>,
    ) {
        let events: Vec<WebEvent> = {
            let Ok(mut queue) = EVENT_QUEUE.lock() else {
                return;
            };
            std::mem::take(&mut *queue)
        };

        for event in events {
            match event {
                WebEvent::Input(text) => {
                    state.current_value = text.clone();
                    input_events.write(WebTextInput(text));
                }
                WebEvent::Submit(text) => {
                    state.current_value.clear();
                    submit_events.write(WebTextSubmit(text));
                }
                WebEvent::Composition { text, is_composing } => {
                    composition_events.write(WebImeComposition { text, is_composing });
                }
            }
        }
    }
}
