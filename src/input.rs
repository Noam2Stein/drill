use lib_input::{Axis, Button, InputMapped, Value, button_bindings};
use lib_window::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Default, InputMapped)]
pub struct Input {
    pub x: Axis<Value>,
    pub y: Axis<Value>,
    pub jump: Button,
    pub drill: Button,

    pub menu_x: Axis<Button>,
    pub menu_y: Axis<Button>,
    pub menu_accept: Button,
    pub menu_cancel: Button,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            x: (
                button_bindings!(KeyCode::ArrowRight),
                button_bindings!(KeyCode::ArrowLeft),
            ),
            y: (
                button_bindings!(KeyCode::ArrowUp),
                button_bindings!(KeyCode::ArrowDown),
            ),
            jump: button_bindings!(KeyCode::Space),
            drill: button_bindings!(KeyCode::KeyC),

            menu_x: (
                button_bindings!(KeyCode::ArrowRight),
                button_bindings!(KeyCode::ArrowLeft),
            ),
            menu_y: (
                button_bindings!(KeyCode::ArrowUp),
                button_bindings!(KeyCode::ArrowDown),
            ),
            menu_accept: button_bindings!(KeyCode::Space),
            menu_cancel: button_bindings!(KeyCode::KeyC),
        }
    }
}
