//! Input event accumulator for [winit].
//!
//! Used to accumulate events and distribute them throughout an application. This implementation
//! uses a 2-window for keyboard and mouse buttons so it can capture an inter-frame toggle while
//! enforcing a single action per frame.
//! ```
//! use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};
//! use winput::Input;
//!
//! let mut input = Input::default();
//!
//! input.register_key(&KeyboardInput {
//!     scancode: 0,
//!     state: ElementState::Pressed,
//!     virtual_keycode: Some(VirtualKeyCode::A),
//!     modifiers: ModifiersState::default(),
//! });
//!
//! assert!(input.is_key_toggled_down(VirtualKeyCode::A));
//!
//! input.register_mouse_position(1f32, 2f32);
//! ```
#![deny(
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
use std::fmt;
use winit::event::*;

#[cfg(test)]
mod test;

const NUM_KEYS: usize = 163;
const NUM_MOUSE_BUTTONS: usize = 256 + 3;

// ---

#[derive(Clone)]
struct Keys([KeyInput; NUM_KEYS]);

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for idx in 0..self.0.len() - 1 {
            write!(f, "{:?}", self.0[idx])?;
        }
        write!(f, "{:?}", self.0.last())
    }
}

impl Default for Keys {
    fn default() -> Self {
        let default = KeyInput {
            state: ElementState::Released,
            modifiers: ModifiersState::empty(),
        };
        Keys([default; NUM_KEYS])
    }
}

// ---

#[derive(Clone)]
struct MouseButtons([MouseInput; NUM_MOUSE_BUTTONS]);

impl fmt::Debug for MouseButtons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for idx in 0..self.0.len() - 1 {
            write!(f, "{:?}", self.0[idx])?;
        }
        write!(f, "{:?}", self.0.last())
    }
}

impl Default for MouseButtons {
    fn default() -> Self {
        let default = MouseInput {
            state: ElementState::Released,
            modifiers: ModifiersState::empty(),
        };
        MouseButtons([default; NUM_MOUSE_BUTTONS])
    }
}

// ---

/// Position of the mouse
#[derive(Clone, Copy)]
pub struct MousePosition(i32, i32);

/// Keyboard input as a buttonstate and modifier state
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    /// Modifiers pressed while this event occurred
    pub modifiers: ModifiersState,
    /// State of the button
    pub state: ElementState,
}

/// Mouse input as a buttonstate and a modifier state
#[derive(Clone, Copy, Debug)]
pub struct MouseInput {
    /// State of the button
    pub state: ElementState,
    /// Modifiers pressed while this event occurred
    pub modifiers: ModifiersState,
}

// ---

/// 2-window for accumulating [winit] input events.
///
/// This struct accumulates input events and allows them to be used throughout the program. Its
/// main purpose is to resolve issues of multiple keypresses per-frame as well as accumulating
/// mouse events such as position and mousewheel events.
#[derive(Clone, Debug, Default)]
pub struct Input {
    keys_now: Keys,
    keys_before: Keys,

    mouse_buttons_now: MouseButtons,
    mouse_buttons_before: MouseButtons,

    mouse_now: (f32, f32),
    mouse_before: (f32, f32),

    mouse_wheel: f32,

    hide_mouse: bool,
    hide_keys: bool,
    current_modifiers: ModifiersState,
}

impl Input {
    /// Clear delta-based inputs such as mouse-wheel, and overwrite the previous mouse position
    pub fn prepare_for_next_frame(&mut self) {
        self.mouse_wheel = 0.0;
        self.mouse_before.0 = self.mouse_now.0;
        self.mouse_before.1 = self.mouse_now.1;

        self.keys_before.0.copy_from_slice(&self.keys_now.0);
        self.mouse_buttons_before
            .0
            .copy_from_slice(&self.mouse_buttons_now.0);
        self.hide_mouse = false;
        self.hide_keys = false;
    }
    /// Hide any key input for the rest of the current frame.
    pub fn hide_key_state(&mut self) {
        self.hide_keys = true;
    }
    /// Hide any mouse input for the rest of the current frame.
    pub fn hide_mouse_state(&mut self) {
        self.hide_mouse = true;
    }

    // ---

    /// Register an arbitrary winit event.
    ///
    /// This function may completely ignore the event.
    pub fn register_event<'a, T>(&mut self, input: &Event<'a, T>) {
        match input {
            Event::WindowEvent { event, .. } => {
                self.handle_window_event(event);
            }
            _ => {}
        }
    }

    /// Set the current modifier state.
    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.current_modifiers = modifiers;
    }

    fn handle_window_event<'a>(&mut self, event: &WindowEvent<'a>) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                self.register_key(input);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.register_mouse_position(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.register_mouse_wheel(delta);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.register_mouse_input(state, button);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.current_modifiers = *modifiers;
            }
            _ => {}
        }
    }

    /// Register a keyboard input
    pub fn register_key(&mut self, input: &KeyboardInput) {
        if let KeyboardInput {
            virtual_keycode: Some(keycode),
            ..
        } = input
        {
            let keycode = *keycode as usize;
            self.keys_before.0[keycode] = self.keys_now.0[keycode];
            self.keys_now.0[keycode] = KeyInput {
                state: input.state,
                modifiers: self.current_modifiers,
            };
        }
    }

    /// Check if a key is pressed
    pub fn is_key_down(&self, keycode: VirtualKeyCode) -> bool {
        !self.hide_keys && self.keys_now.0[keycode as usize].state == ElementState::Pressed
    }

    /// Check if a key is up (released)
    pub fn is_key_up(&self, keycode: VirtualKeyCode) -> bool {
        self.hide_keys || self.keys_now.0[keycode as usize].state == ElementState::Released
    }

    /// Check if a key has been toggled
    pub fn is_key_toggled(&self, keycode: VirtualKeyCode) -> bool {
        !self.hide_keys
            && self.keys_before.0[keycode as usize].state != self.keys_now.0[keycode as usize].state
    }

    /// Check if a key has been toggled and is pressed
    pub fn is_key_toggled_down(&self, keycode: VirtualKeyCode) -> bool {
        self.is_key_down(keycode) && self.is_key_toggled(keycode)
    }

    /// Check if a key has been toggled and is released
    pub fn is_key_toggled_up(&self, keycode: VirtualKeyCode) -> bool {
        !self.is_key_down(keycode) && self.is_key_toggled(keycode)
    }

    /// Get a key's modifiers state
    pub fn key_modifiers_state(&self, keycode: VirtualKeyCode) -> ModifiersState {
        self.keys_now.0[keycode as usize].modifiers
    }

    // ---

    /// Register a mouse button event
    pub fn register_mouse_input(&mut self, state: &ElementState, button: &MouseButton) {
        let index = mouse_button_to_index(*button);
        self.mouse_buttons_before.0[index] = self.mouse_buttons_now.0[index];
        self.mouse_buttons_now.0[index] = MouseInput {
            state: *state,
            modifiers: self.current_modifiers,
        };
    }

    /// Check if a mouse button is pressed
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        let index = mouse_button_to_index(button);
        !self.hide_mouse && self.mouse_buttons_now.0[index].state == ElementState::Pressed
    }

    /// Check if a mouse button is released (up)
    pub fn is_mouse_button_up(&self, button: MouseButton) -> bool {
        let index = mouse_button_to_index(button);
        self.hide_mouse || self.mouse_buttons_now.0[index].state == ElementState::Released
    }

    /// Check if a mouse button is toggled
    pub fn is_mouse_button_toggled(&self, button: MouseButton) -> bool {
        let index = mouse_button_to_index(button);
        !self.hide_mouse
            && self.mouse_buttons_before.0[index].state != self.mouse_buttons_now.0[index].state
    }

    /// Check if a mouse button is toggled and is pressed
    pub fn is_mouse_button_toggled_down(&self, button: MouseButton) -> bool {
        self.is_mouse_button_toggled(button) && self.is_mouse_button_down(button)
    }

    /// Check if a mouse button is toggled and is released
    pub fn is_mouse_button_toggled_up(&self, button: MouseButton) -> bool {
        self.is_mouse_button_toggled(button) && self.is_mouse_button_up(button)
    }

    /// Get a mouse button's modifiers state
    pub fn mouse_button_modifiers_state(&self, button: MouseButton) -> ModifiersState {
        let index = mouse_button_to_index(button);
        self.mouse_buttons_now.0[index].modifiers
    }

    // ---

    /// Register the position of the mouse
    pub fn register_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_now.0 = x;
        self.mouse_now.1 = y;
    }

    /// Register a scroll wheel event
    pub fn register_mouse_wheel(&mut self, delta: &MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.mouse_wheel += y;
            }
            _ => {}
        }
    }

    /// Get the current mouse position
    pub fn get_mouse_position(&self) -> (f32, f32) {
        (self.mouse_now.0, self.mouse_now.1)
    }

    /// Get the mouse movement since last frame
    pub fn get_mouse_moved(&self) -> (f32, f32) {
        (
            (self.mouse_now.0 - self.mouse_before.0),
            (self.mouse_now.1 - self.mouse_before.1),
        )
    }

    /// Get the current mouse wheel value
    pub fn get_mouse_wheel(&self) -> f32 {
        self.mouse_wheel
    }

    // ---
}

fn mouse_button_to_index(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(value) => 3 + value as usize,
    }
}

#[cfg(test)]
mod tests {}
