use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct WebTextInput(pub String);

#[derive(Message, Debug, Clone)]
pub struct WebTextSubmit(pub String);

pub struct WebInputPlugin;

impl Plugin for WebInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WebTextInput>()
            .add_message::<WebTextSubmit>();

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

        let _ =
            input.add_event_listener_with_callback("input", input_handler.as_ref().unchecked_ref());
        let _ = input
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref());

        input_handler.forget();
        keydown_handler.forget();
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = bevyFocusInput)]
        fn js_focus_input();
    }

    pub fn focus_input() {
        js_focus_input();
    }

    pub fn poll_web_input(
        mut input_events: MessageWriter<WebTextInput>,
        mut submit_events: MessageWriter<WebTextSubmit>,
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
                    input_events.write(WebTextInput(text));
                }
                WebEvent::Submit(text) => {
                    submit_events.write(WebTextSubmit(text));
                }
            }
        }
    }
}
