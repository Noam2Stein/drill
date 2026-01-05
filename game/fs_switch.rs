use winit::{keyboard::KeyCode, window::Fullscreen};

use crate::game::{GameContext, GameEvent};

#[derive(Debug, Default)]
pub struct FsSwitch {
    alt_left_is_held: bool,
    alt_right_is_held: bool,
    enter_is_held: bool,
    numpad_enter_is_held: bool,
}

impl FsSwitch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event(&mut self, event: &GameEvent, ctx: GameContext) {
        let GameEvent::Key { code, is_held } = event else {
            return;
        };

        let is_held = *is_held;

        match code {
            KeyCode::AltLeft => self.alt_left_is_held = is_held,
            KeyCode::AltRight => self.alt_right_is_held = is_held,

            KeyCode::Enter => {
                if is_held && !self.enter_is_held {
                    Self::switch(ctx);
                }

                self.enter_is_held = is_held;
            }

            KeyCode::NumpadEnter => {
                if is_held && !self.numpad_enter_is_held {
                    Self::switch(ctx);
                }

                self.numpad_enter_is_held = is_held;
            }

            _ => {}
        }
    }

    fn switch(ctx: GameContext) {
        if ctx.window.fullscreen().is_some() {
            ctx.window.set_fullscreen(None);
        } else {
            ctx.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }
}
