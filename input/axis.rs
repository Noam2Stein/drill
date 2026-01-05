use crate::{
    game::GameEvent,
    input::{
        Button, ButtonBindings, ButtonHandler, Value, ValueHandler, stick_handler::StickHandler,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Axis<T> {
    pub positive: T,
    pub negative: T,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AxisBindings {
    pub positive: ButtonBindings,
    pub negative: ButtonBindings,
}

#[derive(Debug, Default)]
pub(in crate::input) struct AxisHandler<T> {
    positive: T,
    negative: T,
}

impl Axis<Value> {
    pub fn value(&self) -> f32 {
        self.positive.0 - self.negative.0
    }
}

impl AxisHandler<ButtonHandler> {
    pub fn new(bindings: &AxisBindings) -> Self {
        Self {
            positive: ButtonHandler::new(&bindings.positive),
            negative: ButtonHandler::new(&bindings.negative),
        }
    }

    pub fn event(&mut self, event: &GameEvent, stick_handler: &StickHandler) {
        self.positive.event(event, stick_handler);
        self.negative.event(event, stick_handler);
    }

    pub fn next_state(&mut self) -> Axis<Button> {
        Axis {
            positive: self.positive.next_state(),
            negative: self.negative.next_state(),
        }
    }
}

impl AxisHandler<ValueHandler> {
    pub fn new(bindings: &AxisBindings) -> Self {
        Self {
            positive: ValueHandler::new(&bindings.positive),
            negative: ValueHandler::new(&bindings.negative),
        }
    }

    pub fn event(&mut self, event: &GameEvent) {
        self.positive.event(event);
        self.negative.event(event);
    }

    pub fn next_state(&mut self) -> Axis<Value> {
        Axis {
            positive: self.positive.next_state(),
            negative: self.negative.next_state(),
        }
    }
}
