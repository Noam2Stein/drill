use glam::{Vec2, vec2};

use crate::game::{ButtonCode, GameEvent};

#[derive(Debug, Default)]
pub struct StickHandler {
    left_stick_dir: Vec2,
    right_stick_dir: Vec2,
    left_stick_right: f32,
    left_stick_left: f32,
    left_stick_up: f32,
    left_stick_down: f32,
    right_stick_right: f32,
    right_stick_left: f32,
    right_stick_up: f32,
    right_stick_down: f32,
}

impl StickHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event(&mut self, event: &GameEvent) {
        let GameEvent::Button { code, value } = event else {
            return;
        };

        match code {
            ButtonCode::LeftStickRight => self.left_stick_right = *value,
            ButtonCode::LeftStickLeft => self.left_stick_left = *value,
            ButtonCode::LeftStickUp => self.left_stick_up = *value,
            ButtonCode::LeftStickDown => self.left_stick_down = *value,
            ButtonCode::RightStickRight => self.right_stick_right = *value,
            ButtonCode::RightStickLeft => self.right_stick_left = *value,
            ButtonCode::RightStickUp => self.right_stick_up = *value,
            ButtonCode::RightStickDown => self.right_stick_down = *value,
            _ => {}
        }

        self.left_stick_dir = vec2(
            self.left_stick_right - self.left_stick_left,
            self.left_stick_up - self.left_stick_down,
        );
        self.right_stick_dir = vec2(
            self.right_stick_right - self.right_stick_left,
            self.right_stick_up - self.right_stick_down,
        );
    }

    pub fn left_stick_dir(&self) -> Vec2 {
        self.left_stick_dir
    }

    pub fn right_stick_dir(&self) -> Vec2 {
        self.right_stick_dir
    }
}
