use std::collections::{HashMap, HashSet};

use winit::keyboard::KeyCode;

use crate::{
    game::{ButtonCode, GameEvent},
    input::stick_handler::StickHandler,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Button {
    pub is_held: bool,
    pub is_pressed: bool,
    pub is_released: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ButtonBindings {
    pub keys: HashSet<KeyCode>,
    pub buttons: HashSet<ButtonCode>,
}

#[derive(Debug, Default)]
pub(in crate::input) struct ButtonHandler {
    key_indices: HashMap<KeyCode, u8>,
    button_indices: HashMap<ButtonCode, u8>,
    held_bindings: u32,
    is_pressed: bool,
    was_held: bool,
}

impl ButtonHandler {
    pub fn new(bindings: &ButtonBindings) -> Self {
        let key_indices = bindings
            .keys
            .iter()
            .copied()
            .enumerate()
            .map(|(index, code)| (code, index as u8))
            .collect();

        let button_indices = bindings
            .buttons
            .iter()
            .copied()
            .enumerate()
            .map(|(index, code)| (code, index as u8))
            .collect();

        Self {
            key_indices,
            button_indices,
            held_bindings: 0,
            is_pressed: false,
            was_held: false,
        }
    }

    pub fn event(&mut self, event: &GameEvent, stick_handler: &StickHandler) {
        let binding_index;
        let binding_is_held;

        match event {
            GameEvent::Key { code, is_held } => {
                if let Some(index) = self.key_indices.get(code) {
                    binding_index = *index;
                    binding_is_held = *is_held;
                } else {
                    return;
                }
            }

            GameEvent::Button { code, value } => {
                if let Some(index) = self.button_indices.get(code) {
                    const STICK_DIR_DOT: f32 = 0.3827;

                    let is_held = match code {
                        ButtonCode::DPadRight
                        | ButtonCode::DPadLeft
                        | ButtonCode::DPadUp
                        | ButtonCode::DPadDown
                        | ButtonCode::South
                        | ButtonCode::North
                        | ButtonCode::East
                        | ButtonCode::West
                        | ButtonCode::C
                        | ButtonCode::Z
                        | ButtonCode::LeftThumb
                        | ButtonCode::RightThumb
                        | ButtonCode::Start
                        | ButtonCode::Select
                        | ButtonCode::Mode
                        | ButtonCode::LeftTrigger
                        | ButtonCode::LeftTrigger2
                        | ButtonCode::RightTrigger
                        | ButtonCode::RightTrigger2
                        | ButtonCode::Unknown => *value >= 0.5,

                        ButtonCode::LeftStickRight => {
                            *value >= 0.5 && stick_handler.left_stick_dir().x >= STICK_DIR_DOT
                        }
                        ButtonCode::LeftStickLeft => {
                            *value >= 0.5 && -stick_handler.left_stick_dir().x >= STICK_DIR_DOT
                        }
                        ButtonCode::LeftStickUp => {
                            *value >= 0.5 && stick_handler.left_stick_dir().y >= STICK_DIR_DOT
                        }
                        ButtonCode::LeftStickDown => {
                            *value >= 0.5 && -stick_handler.left_stick_dir().y >= STICK_DIR_DOT
                        }

                        ButtonCode::RightStickRight => {
                            *value >= 0.5 && stick_handler.right_stick_dir().x >= STICK_DIR_DOT
                        }
                        ButtonCode::RightStickLeft => {
                            *value >= 0.5 && -stick_handler.right_stick_dir().x >= STICK_DIR_DOT
                        }
                        ButtonCode::RightStickUp => {
                            *value >= 0.5 && stick_handler.right_stick_dir().y >= STICK_DIR_DOT
                        }
                        ButtonCode::RightStickDown => {
                            *value >= 0.5 && -stick_handler.right_stick_dir().y >= STICK_DIR_DOT
                        }
                    };

                    binding_index = *index;
                    binding_is_held = is_held;
                } else {
                    return;
                }
            }

            _ => return,
        };

        let binding_mask = 1 << binding_index;
        let binding_was_held = self.held_bindings & binding_mask != 0;

        if binding_is_held && !binding_was_held {
            self.is_pressed = true;
        }

        self.held_bindings =
            (self.held_bindings & !binding_mask) | (binding_mask * binding_is_held as u32);
    }

    pub fn next_state(&mut self) -> Button {
        let state = Button {
            is_held: self.held_bindings != 0,
            is_pressed: self.is_pressed,
            is_released: self.was_held && self.held_bindings == 0,
        };

        self.was_held = state.is_held;
        self.is_pressed = false;

        state
    }
}
