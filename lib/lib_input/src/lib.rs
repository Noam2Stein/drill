use std::fmt::Debug;

use lib_math::{Vec2, f32::FVec2, vec2};
use lib_window::{ButtonCode, ButtonEvent, DeviceEvent};

mod axis;
mod button;
mod value;
pub use axis::*;
pub use button::*;
pub use value::*;

pub use lib_input_proc_macros::InputMapped;

pub trait InputMapped: Debug + Clone + Copy + PartialEq + Default {
    type Bindings: Debug + Clone + PartialEq;
    type MapperState: Debug + Clone;

    fn new_mapper(bindings: &Self::Bindings) -> Self::MapperState;

    fn mapper_event(handler: &mut Self::MapperState, event: DeviceEvent<'_>, ctx: &MapperContext);

    fn map(handler: &mut Self::MapperState) -> Self;
}

#[derive(Debug, Clone)]
pub struct Mapper<T: InputMapped> {
    state: T::MapperState,
    left_stick_right: f32,
    left_stick_left: f32,
    left_stick_up: f32,
    left_stick_down: f32,
    right_stick_right: f32,
    right_stick_left: f32,
    right_stick_up: f32,
    right_stick_down: f32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct MapperContext {
    pub left_stick_dir: FVec2,
    pub right_stick_dir: FVec2,
}

impl<T: InputMapped> Mapper<T> {
    pub fn new(bindings: &T::Bindings) -> Self {
        Self {
            state: T::new_mapper(bindings),
            left_stick_right: 0.0,
            left_stick_left: 0.0,
            left_stick_up: 0.0,
            left_stick_down: 0.0,
            right_stick_right: 0.0,
            right_stick_left: 0.0,
            right_stick_up: 0.0,
            right_stick_down: 0.0,
        }
    }

    pub fn event(&mut self, event: DeviceEvent<'_>) {
        match event {
            DeviceEvent::Button(ButtonEvent { button, value }) => match button {
                ButtonCode::LeftStickRight => self.left_stick_right = *value,
                ButtonCode::LeftStickLeft => self.left_stick_left = *value,
                ButtonCode::LeftStickUp => self.left_stick_up = *value,
                ButtonCode::LeftStickDown => self.left_stick_down = *value,
                ButtonCode::RightStickRight => self.right_stick_right = *value,
                ButtonCode::RightStickLeft => self.right_stick_left = *value,
                ButtonCode::RightStickUp => self.right_stick_up = *value,
                ButtonCode::RightStickDown => self.right_stick_down = *value,
                _ => (),
            },
            _ => {}
        };

        T::mapper_event(
            &mut self.state,
            event,
            &MapperContext {
                left_stick_dir: vec2!(
                    self.left_stick_right - self.left_stick_left,
                    self.left_stick_up - self.left_stick_down,
                )
                .try_normalize()
                .unwrap_or(Vec2::ZERO),

                right_stick_dir: vec2!(
                    self.right_stick_right - self.right_stick_left,
                    self.right_stick_up - self.right_stick_down,
                )
                .try_normalize()
                .unwrap_or(Vec2::ZERO),
            },
        );
    }

    pub fn map(&mut self) -> T {
        T::map(&mut self.state)
    }
}

#[doc(hidden)]
pub mod hidden {
    pub use lib_input_proc_macros;
}
