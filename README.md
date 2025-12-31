# Bevy Mobile Input

A minimal demo showing how to trigger the virtual keyboard on mobile browsers in a Bevy WASM application.

## The Problem

WebGL canvas elements cannot receive text input directly. On mobile devices, tapping a canvas won't show the virtual keyboard. This is a fundamental browser limitation.

## The Solution

Use a hidden HTML `<input>` element as a bridge:

```
User taps Bevy UI  -->  Focus hidden <input>  -->  Virtual keyboard appears
                                                          |
Bevy receives KeyboardInput  <--  JS captures keydown  <--+
```

The hidden input is positioned off-screen but remains focusable. When focused, the browser shows the virtual keyboard. Keystrokes are captured and injected directly into Bevy's native `KeyboardInput` event queue, so your game code doesn't need any special handling.

## How It Works

1. **HTML**: A hidden `<input>` element exists in the page (`web/index.html`)
2. **Click detection**: Bevy's picking system detects clicks on the input UI via `Pointer<Click>` observer
3. **Focus**: `show_keyboard()` is called, which focuses the hidden input element
4. **Keyboard appears**: The browser shows the virtual keyboard
5. **Events flow**: `keydown` events are captured in JS and sent to Bevy as native `KeyboardInput` events

## Usage

```rust
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
mod web_input;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(web_input::MobileInputPlugin);
    
    app.run();
}

// Trigger keyboard on click using an observer
commands.spawn(Button).observe(|_: Trigger<Pointer<Click>>| {
    #[cfg(target_arch = "wasm32")]
    web_input::show_keyboard();
});

// Handle input like any physical keyboard
fn handle_input(mut events: EventReader<KeyboardInput>) {
    for event in events.read() {
        // Works with both physical and virtual keyboards
    }
}
```

## Files

| File | Purpose |
|------|---------|
| `src/web_input.rs` | WASM-only plugin that sets up the JS-to-Bevy event bridge |
| `web/index.html` | Hidden input element and viewport fixes for mobile |
| `src/main.rs` | Demo app with a clickable input field |

## Running

Requires [bevy_cli](https://github.com/TheBevyFlock/bevy_cli):

```bash
bevy run web
```

Then open on a mobile device or use browser dev tools to emulate a touch device.
