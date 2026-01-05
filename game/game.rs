use glam::{Vec2, vec2};
use wgpu::TextureView;
use winit::window::{Window, WindowAttributes};

use crate::{
    game::{FsSwitch, GameContext, GameEvent, Time},
    input::{InputBindings, InputHandler},
    renderer::{Quad, Renderer, Sprite},
};

#[derive(Debug)]
pub struct Game {
    time: Time,
    fs_switch: FsSwitch,
    renderer: Renderer,
    input: InputHandler,
    t: f32,
    pos: Vec2,
}

impl Game {
    pub fn window_attributes() -> WindowAttributes {
        Window::default_attributes()
            .with_title("Drill Game")
            .with_maximized(true)
    }

    pub fn new(ctx: GameContext) -> Self {
        Self {
            time: Time::new(),
            fs_switch: FsSwitch::new(),
            renderer: Renderer::new(ctx.into()),
            input: InputHandler::new(&InputBindings::default()),
            t: 0.0,
            pos: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, _: GameContext) {
        let dt = self.time.tick();
        self.t += dt;

        let input = self.input.next_state();

        self.pos += vec2(input.x.value(), input.y.value()) * 10.0 * dt;
    }

    pub fn render(&mut self, output: &TextureView, ctx: GameContext) {
        self.renderer.render_frame(
            |r| {
                r.render_layer(
                    |r| {
                        r.render_quad(Quad {
                            center: self.pos,
                            layer: 0.0,
                            sprite: Sprite {
                                center: Vec2::splat(1.0 / 40.0),
                                extents: Vec2::splat(1.0 / 40.0),
                            },
                        })
                    },
                    vec2(3.0, self.t.sin()),
                );

                r.render_layer(
                    |r| {
                        r.render_quad(Quad {
                            center: Vec2::ZERO,
                            layer: 0.0,
                            sprite: Sprite {
                                center: Vec2::splat(3.0 / 40.0),
                                extents: Vec2::splat(1.0 / 40.0),
                            },
                        })
                    },
                    vec2(0.0, 0.0),
                );
            },
            output,
            ctx.into(),
        );
    }

    pub fn event(&mut self, event: &GameEvent, ctx: GameContext) {
        self.fs_switch.event(event, ctx);
        self.input.event(event);

        match event {
            GameEvent::CloseRequested => ctx.exit(),
            _ => {}
        }
    }

    pub fn end(&mut self, _: GameContext) {}
}
