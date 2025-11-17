use std::collections::HashMap;

use lib_window::{
    ButtonCode, ButtonEvent, DeviceEvent,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{ButtonBindings, InputMapped, MapperContext};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Value(pub f32);

impl InputMapped for Value {
    type Bindings = ButtonBindings;
    type MapperState = ValueHandlerState;

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

            bindings_values: [0; 32],
        }
    }

    fn mapper_event(handler: &mut Self::MapperState, event: DeviceEvent<'_>, _ctx: &MapperContext) {
        let (binding_idx, value) = match event {
            DeviceEvent::Connected => return,
            DeviceEvent::Disconnected => {
                handler.bindings_values = [0; 32];
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
                    Some(&idx) => (idx, if state.is_pressed() { !0 } else { 0 }),
                    None => return,
                }
            }

            DeviceEvent::Button(ButtonEvent { button, value }) => {
                match handler.button_indices.get(&button) {
                    Some(&idx) => (idx, (*value * 255.0) as u8),
                    None => return,
                }
            }

            _ => return,
        };

        handler.bindings_values[binding_idx as usize] = value;
    }

    fn map(handler: &mut Self::MapperState) -> Self {
        Value(
            handler
                .bindings_values
                .into_iter()
                .map(|x| x as f32 / 255.0)
                .sum(),
        )
    }
}

mod private {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct ValueHandlerState {
        pub(super) key_indices: HashMap<KeyCode, u8>,
        pub(super) button_indices: HashMap<ButtonCode, u8>,
        pub(super) bindings_values: [u8; 32],
    }
}
use private::*;
