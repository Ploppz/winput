use super::*;

#[test]
fn tri_state_switch_pressed_released_pressed() {
    let mut input = Input::default();

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Released,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    assert_eq!(true, input.is_key_toggled_down(VirtualKeyCode::A));
    assert_eq!(false, input.is_key_toggled_up(VirtualKeyCode::A));
    assert_eq!(true, input.is_key_down(VirtualKeyCode::A));
}

#[test]
fn tri_state_switch_released_pressed_released() {
    let mut input = Input::default();

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Released,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Released,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    assert_eq!(false, input.is_key_toggled_down(VirtualKeyCode::A));
    assert_eq!(true, input.is_key_toggled_up(VirtualKeyCode::A));
    assert_eq!(false, input.is_key_down(VirtualKeyCode::A));
}

#[test]
fn tri_state_modifiers() {
    let mut input = Input::default();

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.set_modifiers(ModifiersState::CTRL);

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Released,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    input.set_modifiers(ModifiersState::SHIFT);

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::A),
        modifiers: ModifiersState::default(),
    });

    assert_eq!(true, input.is_key_toggled_down(VirtualKeyCode::A));
    assert_eq!(false, input.is_key_toggled_up(VirtualKeyCode::A));
    assert_eq!(true, input.is_key_down(VirtualKeyCode::A));
    assert_eq!(false, input.key_modifiers_state(VirtualKeyCode::A).ctrl());
    assert_eq!(true, input.key_modifiers_state(VirtualKeyCode::A).shift());
}

#[test]
fn tri_state_mouse_input() {
    let mut input = Input::default();

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);

    input.register_mouse_input(&ElementState::Released, &MouseButton::Left);

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);

    assert_eq!(true, input.is_mouse_button_toggled(MouseButton::Left));
    assert_eq!(true, input.is_mouse_button_down(MouseButton::Left));
    assert_eq!(false, input.is_mouse_button_up(MouseButton::Left));
    assert_eq!(true, input.is_mouse_button_toggled_down(MouseButton::Left));
    assert_eq!(false, input.is_mouse_button_toggled_up(MouseButton::Left));
    assert_eq!(
        ModifiersState::default(),
        input.mouse_button_modifiers_state(MouseButton::Left)
    );
}

#[test]
fn tri_state_mouse_modifiers() {
    let mut input = Input::default();

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);

    input.set_modifiers(ModifiersState::ALT);

    input.register_mouse_input(&ElementState::Released, &MouseButton::Left);

    input.set_modifiers(ModifiersState::LOGO);

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);

    assert_eq!(
        true,
        input.mouse_button_modifiers_state(MouseButton::Left).logo()
    );
    assert_eq!(
        false,
        input.mouse_button_modifiers_state(MouseButton::Left).alt()
    );
}

#[test]
fn mouse_toggled_is_reset_on_next_frame() {
    let mut input = Input::default();

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);

    assert_eq!(true, input.is_mouse_button_toggled(MouseButton::Left));

    input.prepare_for_next_frame();

    assert_eq!(false, input.is_mouse_button_toggled(MouseButton::Left));
}

#[test]
fn only_consider_last_mouse_pos() {
    let mut input = Input::default();
    input.register_mouse_position(1f32, 1f32);
    input.register_mouse_position(8f32, 9f32);
    input.register_mouse_position(123f32, 456f32);
    input.register_mouse_position(3f32, 6f32);

    assert_eq!((3.0, 6.0), input.get_mouse_position());
    assert_eq!((3.0, 6.0), input.get_mouse_moved());

    input.prepare_for_next_frame();

    assert_eq!((3.0, 6.0), input.get_mouse_position());
    assert_eq!((0.0, 0.0), input.get_mouse_moved());
}

#[test]
fn accumulate_mouse_wheel_deltas() {
    let mut input = Input::default();
    input.register_mouse_wheel(&MouseScrollDelta::LineDelta(0.0, 0.1));
    input.register_mouse_wheel(&MouseScrollDelta::LineDelta(0.0, 0.8));
    input.register_mouse_wheel(&MouseScrollDelta::LineDelta(0.0, 0.3));
    assert_eq!(1.2, input.get_mouse_wheel());

    input.prepare_for_next_frame();

    assert_eq!(0.0, input.get_mouse_wheel());
}

#[test]
fn ensure_boundaries_ok() {
    let mut input = Input::default();
    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::Cut),
        modifiers: ModifiersState::default(),
    });

    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: None,
        modifiers: ModifiersState::default(),
    });

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Other(255));
}

#[test]
fn state_advances_after_preparation() {
    let mut input = Input::default();
    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::F),
        modifiers: ModifiersState::default(),
    });

    assert!(input.is_key_toggled_down(VirtualKeyCode::F));
    input.prepare_for_next_frame();
    assert!(!input.is_key_toggled_down(VirtualKeyCode::F));
}

#[test]
fn hide_mouse_and_keys() {
    let mut input = Input::default();
    input.register_key(&KeyboardInput {
        scancode: 0,
        state: ElementState::Pressed,
        virtual_keycode: Some(VirtualKeyCode::F),
        modifiers: ModifiersState::default(),
    });
    input.hide_key_state();
    assert!(!input.is_key_down(VirtualKeyCode::F));
    assert!(input.is_key_up(VirtualKeyCode::F));
    assert!(!input.is_key_toggled_down(VirtualKeyCode::F));
    assert!(!input.is_key_toggled_up(VirtualKeyCode::F));

    input.register_mouse_input(&ElementState::Pressed, &MouseButton::Left);
    input.hide_mouse_state();
    assert!(!input.is_mouse_button_down(MouseButton::Left));
    assert!(input.is_mouse_button_up(MouseButton::Left));
    assert!(!input.is_mouse_button_toggled(MouseButton::Left));
}
