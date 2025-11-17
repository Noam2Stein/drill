use std::collections::{HashMap, HashSet};

use lib_math::{NegativeDownExt, NegativeLeftExt, PositiveRightExt, PositiveUpExt, Vec2};
use lib_window::{ButtonCode, ButtonEvent, DeviceEvent, KeyCode, PhysicalKey, event::KeyEvent};

use crate::{InputMapped, MapperContext};

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

#[macro_export]
macro_rules! button_bindings {
    ($($e:expr),* $(,)*) => {{
        let mut result = $crate::ButtonBindings::default();
        $(
            result.extend(std::iter::once($e));
        )*
        result
    }};
}

impl InputMapped for Button {
    type Bindings = ButtonBindings;
    type MapperState = ButtonHandlerState;

    fn new_mapper(bindings: &Self::Bindings) -> Self::MapperState {
        Self::MapperState {
            key_indices: bindings
                .keys
                .iter()
                .copied()
                .enumerate()
                .map(|(i, key)| (key, i as u8))
                .collect(),

            button_indices: bindings
                .buttons
                .iter()
                .copied()
                .enumerate()
                .map(|(i, button)| (button, (i + bindings.keys.len()) as u8))
                .collect(),

            held_bindings: 0,
            is_pressed: false,
            was_held: false,
        }
    }

    fn mapper_event(handler: &mut Self::MapperState, event: DeviceEvent<'_>, ctx: &MapperContext) {
        let (binding_idx, is_held) = match event {
            DeviceEvent::Connected => return,
            DeviceEvent::Disconnected => {
                handler.held_bindings = 0;
                return;
            }

            DeviceEvent::Key(KeyEvent {
                physical_key,
                state,
                ..
            }) => {
                let PhysicalKey::Code(keycode) = physical_key else {
                    return;
                };

                match handler.key_indices.get(&keycode) {
                    Some(&idx) => (idx, state.is_pressed()),
                    None => return,
                }
            }

            DeviceEvent::Button(ButtonEvent { button, value }) => {
                match handler.button_indices.get(&button) {
                    Some(&idx) => {
                        const MIN_DOT: f32 = 0.3827;

                        let is_held = match button {
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
                                *value >= 0.5 && ctx.left_stick_dir.dot(Vec2::RIGHT) >= MIN_DOT
                            }
                            ButtonCode::LeftStickLeft => {
                                *value >= 0.5 && ctx.left_stick_dir.dot(Vec2::LEFT) >= MIN_DOT
                            }
                            ButtonCode::LeftStickUp => {
                                *value >= 0.5 && ctx.left_stick_dir.dot(Vec2::UP) >= MIN_DOT
                            }
                            ButtonCode::LeftStickDown => {
                                *value >= 0.5 && ctx.left_stick_dir.dot(Vec2::DOWN) >= MIN_DOT
                            }

                            ButtonCode::RightStickRight => {
                                *value >= 0.5 && ctx.right_stick_dir.dot(Vec2::RIGHT) >= MIN_DOT
                            }
                            ButtonCode::RightStickLeft => {
                                *value >= 0.5 && ctx.right_stick_dir.dot(Vec2::LEFT) >= MIN_DOT
                            }
                            ButtonCode::RightStickUp => {
                                *value >= 0.5 && ctx.right_stick_dir.dot(Vec2::UP) >= MIN_DOT
                            }
                            ButtonCode::RightStickDown => {
                                *value >= 0.5 && ctx.right_stick_dir.dot(Vec2::DOWN) >= MIN_DOT
                            }
                        };

                        (idx, is_held)
                    }
                    None => return,
                }
            }

            _ => return,
        };

        let binding_mask = 1 << binding_idx;
        let binding_was_held = handler.held_bindings & binding_mask != 0;

        if is_held && !binding_was_held {
            handler.is_pressed = true;
        }

        handler.held_bindings =
            (handler.held_bindings & !binding_mask) | (binding_mask * is_held as u32);
    }

    fn map(handler: &mut Self::MapperState) -> Self {
        let result = Self {
            is_held: handler.held_bindings != 0,
            is_pressed: handler.is_pressed,
            is_released: handler.was_held && handler.held_bindings == 0,
        };

        handler.was_held = result.is_held;
        handler.is_pressed = false;

        result
    }
}

impl Extend<KeyCode> for ButtonBindings {
    fn extend<T: IntoIterator<Item = KeyCode>>(&mut self, iter: T) {
        self.keys.extend(iter);
    }
}

impl Extend<ButtonCode> for ButtonBindings {
    fn extend<T: IntoIterator<Item = ButtonCode>>(&mut self, iter: T) {
        self.buttons.extend(iter);
    }
}

mod private {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct ButtonHandlerState {
        pub(super) key_indices: HashMap<KeyCode, u8>,
        pub(super) button_indices: HashMap<ButtonCode, u8>,
        pub(super) held_bindings: u32,
        pub(super) is_pressed: bool,
        pub(super) was_held: bool,
    }
}
use private::*;
