use lib_window::DeviceEvent;

use crate::{Button, InputMapped, Value};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Axis<T> {
    pub positive: T,
    pub negative: T,
}

impl<T: InputMapped> InputMapped for Axis<T> {
    type Bindings = (T::Bindings, T::Bindings);
    type MapperState = (T::MapperState, T::MapperState);

    fn new_mapper(bindings: &Self::Bindings) -> Self::MapperState {
        (T::new_mapper(&bindings.0), T::new_mapper(&bindings.1))
    }

    fn mapper_event(
        handler: &mut Self::MapperState,
        event: DeviceEvent<'_>,
        ctx: &super::MapperContext,
    ) {
        T::mapper_event(&mut handler.0, event, ctx);
        T::mapper_event(&mut handler.1, event, ctx);
    }

    fn map(handler: &mut Self::MapperState) -> Self {
        Self {
            positive: T::map(&mut handler.0),
            negative: T::map(&mut handler.1),
        }
    }
}

impl Axis<Button> {
    pub fn value(&self) -> i8 {
        self.positive.is_held as i8 - self.negative.is_held as i8
    }
}

impl Axis<Value> {
    pub fn value(&self) -> f32 {
        self.positive.0 - self.negative.0
    }
}
