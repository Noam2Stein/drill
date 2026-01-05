use std::collections::HashMap;

use winit::keyboard::KeyCode;

use crate::{
    game::{ButtonCode, GameEvent},
    input::ButtonBindings,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Value(pub f32);

#[derive(Debug, Default)]
pub(in crate::input) struct ValueHandler {
    key_indices: HashMap<KeyCode, u8>,
    button_indices: HashMap<ButtonCode, u8>,
    binding_values: [u8; 32],
}

impl ValueHandler {
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
            binding_values: [0; 32],
        }
    }

    pub fn event(&mut self, event: &GameEvent) {
        let binding_index;
        let binding_value;

        match event {
            GameEvent::Key { code, is_held } => {
                if let Some(index) = self.key_indices.get(code) {
                    binding_index = *index;
                    binding_value = if *is_held { !0 } else { 0 };
                } else {
                    return;
                }
            }

            GameEvent::Button { code, value } => {
                if let Some(index) = self.button_indices.get(code) {
                    binding_index = *index;
                    binding_value = (*value * 255.0) as u8;
                } else {
                    return;
                }
            }

            _ => return,
        };

        self.binding_values[binding_index as usize] = binding_value;
    }

    pub fn next_state(&mut self) -> Value {
        Value(
            self.binding_values
                .into_iter()
                .map(|x| x as f32 / 255.0)
                .sum::<f32>()
                .min(1.0),
        )
    }
}
