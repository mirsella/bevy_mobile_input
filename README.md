# Bevy WASM IME Input

A Bevy plugin for handling text input on web/mobile platforms, including IME (Input Method Editor) support for CJK languages.

## Problem

Bevy doesn't natively support virtual keyboards on mobile browsers. This plugin solves that by using a hidden HTML input element to capture keyboard events and forward them to Bevy.

## How It Works

1. A hidden `<input>` element is positioned off-screen in the HTML
2. When the user taps the in-game input field, the hidden input is focused
3. This triggers the mobile virtual keyboard
4. Input events (keystrokes, IME composition) are captured and sent to Bevy as messages

## Usage

```rust
use bevy::prelude::*;
mod web_input;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(web_input::WebInputPlugin)
        // your systems here
        .run();
}

fn handle_input(mut events: EventReader<web_input::WebTextInput>) {
    for web_input::WebTextInput(text) in events.read() {
        println!("Input: {}", text);
    }
}

fn on_click_input_field() {
    web_input::focus_input();
}
```

## Messages

| Message | Description |
|---------|-------------|
| `WebTextInput(String)` | Fired on each keystroke with current input value |
| `WebTextSubmit(String)` | Fired when Enter is pressed |
| `WebImeComposition { text, is_composing }` | Fired during IME composition (CJK input) |

## Running

Requires [bevy_cli](https://github.com/TheBevyFlock/bevy_cli):

```bash
bevy run web
```

## Files

- `web/index.html` - Custom HTML with hidden input and focus management
- `src/web_input.rs` - Bevy plugin with event handling
- `src/main.rs` - Demo application
