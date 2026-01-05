use std::collections::HashSet;

use winit::keyboard::KeyCode;

use crate::{
    game::GameEvent,
    input::{
        Axis, AxisBindings, AxisHandler, Button, ButtonBindings, ButtonHandler, Value,
        ValueHandler, stick_handler::StickHandler,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct InputBindings {
    pub x: AxisBindings,
    pub y: AxisBindings,
    pub jump: ButtonBindings,
    pub drill: ButtonBindings,

    pub menu_x: AxisBindings,
    pub menu_y: AxisBindings,
    pub menu_accept: ButtonBindings,
    pub menu_cancel: ButtonBindings,
}

#[derive(Debug, Default)]
pub struct InputHandler {
    x: AxisHandler<ValueHandler>,
    y: AxisHandler<ValueHandler>,
    jump: ButtonHandler,
    drill: ButtonHandler,

    menu_x: AxisHandler<ButtonHandler>,
    menu_y: AxisHandler<ButtonHandler>,
    menu_accept: ButtonHandler,
    menu_cancel: ButtonHandler,

    stick_handler: StickHandler,
}

impl InputHandler {
    pub fn new(bindings: &InputBindings) -> Self {
        Self {
            x: AxisHandler::<ValueHandler>::new(&bindings.x),
            y: AxisHandler::<ValueHandler>::new(&bindings.y),
            jump: ButtonHandler::new(&bindings.jump),
            drill: ButtonHandler::new(&bindings.drill),

            menu_x: AxisHandler::<ButtonHandler>::new(&bindings.menu_x),
            menu_y: AxisHandler::<ButtonHandler>::new(&bindings.menu_y),
            menu_accept: ButtonHandler::new(&bindings.menu_accept),
            menu_cancel: ButtonHandler::new(&bindings.menu_cancel),

            stick_handler: StickHandler::new(),
        }
    }

    pub fn event(&mut self, event: &GameEvent) {
        self.stick_handler.event(event);

        self.x.event(event);
        self.y.event(event);
        self.jump.event(event, &self.stick_handler);
        self.drill.event(event, &self.stick_handler);

        self.menu_x.event(event, &self.stick_handler);
        self.menu_y.event(event, &self.stick_handler);
        self.menu_accept.event(event, &self.stick_handler);
        self.menu_cancel.event(event, &self.stick_handler);
    }

    pub fn next_state(&mut self) -> Input {
        Input {
            x: self.x.next_state(),
            y: self.y.next_state(),
            jump: self.jump.next_state(),
            drill: self.drill.next_state(),

            menu_x: self.menu_x.next_state(),
            menu_y: self.menu_y.next_state(),
            menu_accept: self.menu_accept.next_state(),
            menu_cancel: self.menu_cancel.next_state(),
        }
    }
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            x: AxisBindings {
                positive: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowRight]),
                    buttons: HashSet::from_iter([]),
                },
                negative: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowLeft]),
                    buttons: HashSet::from_iter([]),
                },
            },
            y: AxisBindings {
                positive: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowUp]),
                    buttons: HashSet::from_iter([]),
                },
                negative: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowDown]),
                    buttons: HashSet::from_iter([]),
                },
            },
            jump: ButtonBindings {
                keys: HashSet::from_iter([KeyCode::Space]),
                buttons: HashSet::from_iter([]),
            },
            drill: ButtonBindings {
                keys: HashSet::from_iter([KeyCode::KeyC]),
                buttons: HashSet::from_iter([]),
            },

            menu_x: AxisBindings {
                positive: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowRight]),
                    buttons: HashSet::from_iter([]),
                },
                negative: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowLeft]),
                    buttons: HashSet::from_iter([]),
                },
            },
            menu_y: AxisBindings {
                positive: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowUp]),
                    buttons: HashSet::from_iter([]),
                },
                negative: ButtonBindings {
                    keys: HashSet::from_iter([KeyCode::ArrowDown]),
                    buttons: HashSet::from_iter([]),
                },
            },
            menu_accept: ButtonBindings {
                keys: HashSet::from_iter([KeyCode::Space]),
                buttons: HashSet::from_iter([]),
            },
            menu_cancel: ButtonBindings {
                keys: HashSet::from_iter([KeyCode::KeyC]),
                buttons: HashSet::from_iter([]),
            },
        }
    }
}
